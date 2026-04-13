use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write as _;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Write a TOML string to a named temp file and return it (keeps file alive).
fn write_config(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "{content}").unwrap();
    f
}

fn russh() -> Command {
    Command::cargo_bin("russh").unwrap()
}

// Path to the checked-in fixture with three clean sessions.
fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

// ── list ─────────────────────────────────────────────────────────────────────

#[test]
fn list_shows_three_sessions() {
    russh()
        .args(["--config", &fixture("three_sessions.toml"), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("bravo"))
        .stdout(predicate::str::contains("charlie"));
}

#[test]
fn list_shows_table_headers() {
    russh()
        .args(["--config", &fixture("three_sessions.toml"), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("HOST"))
        .stdout(predicate::str::contains("USER"));
}

#[test]
fn list_shows_hosts_and_ports() {
    russh()
        .args(["--config", &fixture("three_sessions.toml"), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("10.0.0.1"))
        .stdout(predicate::str::contains("10.0.0.2"))
        .stdout(predicate::str::contains("10.0.0.3"));
}

#[test]
fn list_empty_config_prints_no_sessions() {
    let cfg = write_config("[sessions]\n");
    russh()
        .args(["--config", cfg.path().to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No sessions configured"));
}

#[test]
fn list_missing_config_exits_nonzero() {
    russh()
        .args(["--config", "/nonexistent/path/config.toml", "list"])
        .assert()
        .failure();
}

// ── show ─────────────────────────────────────────────────────────────────────

#[test]
fn show_known_session_displays_details() {
    russh()
        .args(["--config", &fixture("three_sessions.toml"), "show", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("10.0.0.1"))
        .stdout(predicate::str::contains("alice"));
}

#[test]
fn show_session_with_defaults_shows_resolved_values() {
    // "charlie" has no username or port; show should display resolved defaults.
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "show",
            "charlie",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("charlie"))
        .stdout(predicate::str::contains("10.0.0.3"));
}

#[test]
fn show_unknown_session_exits_nonzero() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "show",
            "nosuchsession",
        ])
        .assert()
        .failure();
}

#[test]
fn show_unknown_session_includes_name_in_error() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "show",
            "nosuchsession",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nosuchsession"));
}

// ── check ─────────────────────────────────────────────────────────────────────

#[test]
fn check_clean_config_exits_zero() {
    // All sessions use IP addresses → no warnings, no errors.
    russh()
        .args(["--config", &fixture("three_sessions.toml"), "check"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn check_warnings_only_exits_one() {
    // Sessions with hostname (not IP) produce "hostname-not-ip" warnings.
    let cfg = write_config(
        r#"
[sessions.web]
host = "example.com"
username = "admin"
"#,
    );
    russh()
        .args(["--config", cfg.path().to_str().unwrap(), "check"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Warning"));
}

#[test]
fn check_errors_exits_two() {
    // port = 0 passes config parsing but triggers a validation error.
    let cfg = write_config(
        r#"
[sessions.broken]
host = "10.0.0.9"
port = 0
"#,
    );
    russh()
        .args(["--config", cfg.path().to_str().unwrap(), "check"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("Error"));
}

#[test]
fn check_missing_config_exits_two() {
    russh()
        .args(["--config", "/nonexistent/path/config.toml", "check"])
        .assert()
        .code(2);
}

// ── connect ───────────────────────────────────────────────────────────────────

#[test]
fn connect_unknown_session_exits_nonzero() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "connect",
            "nosuchsession",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nosuchsession"));
}

#[test]
fn connect_alias_c_unknown_session_exits_nonzero() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "c",
            "nosuchsession",
        ])
        .assert()
        .failure();
}

#[test]
fn connect_session_with_validation_error_exits_nonzero() {
    // port = 0 is a launch-blocking error; connect should refuse before exec.
    let cfg = write_config(
        r#"
[sessions.badport]
host = "10.0.0.5"
port = 0
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "connect",
            "badport",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("launch-blocking"));
}

// ── export ───────────────────────────────────────────────────────────────────

#[test]
fn export_prints_config_contents() {
    let content = "[sessions.dev]\nhost = \"10.0.0.1\"\nusername = \"admin\"\n";
    let cfg = write_config(content);
    russh()
        .args(["--config", cfg.path().to_str().unwrap(), "export"])
        .assert()
        .success()
        .stdout(predicate::eq(content));
}

#[test]
fn export_fixture_matches_file() {
    let path = fixture("three_sessions.toml");
    let expected = std::fs::read_to_string(&path).unwrap();
    russh()
        .args(["--config", &path, "export"])
        .assert()
        .success()
        .stdout(predicate::eq(expected.as_str()));
}

#[test]
fn export_missing_config_exits_nonzero() {
    russh()
        .args(["--config", "/nonexistent/path/config.toml", "export"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("config file not found"));
}

// ── proc run ────────────────────────────────────────────────────────────────

#[test]
fn proc_run_unknown_procedure_exits_nonzero() {
    let cfg = write_config(
        r#"
[sessions.dev]
host = "10.0.0.1"
username = "admin"
"#,
    );
    let proc_cfg = write_config(
        r#"
[procedures.deploy]
session = "dev"
commands = ["echo hello"]
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "proc",
            "--from-config",
            proc_cfg.path().to_str().unwrap(),
            "run",
            "nonexistent",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("procedure not found"));
}

#[test]
fn proc_run_unknown_session_exits_nonzero() {
    let cfg = write_config(
        r#"
[sessions.dev]
host = "10.0.0.1"
username = "admin"
"#,
    );
    let proc_cfg = write_config(
        r#"
[procedures.deploy]
session = "ghost"
commands = ["echo hello"]
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "proc",
            "--from-config",
            proc_cfg.path().to_str().unwrap(),
            "run",
            "deploy",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown session"));
}

#[test]
fn proc_run_missing_procedures_config_exits_nonzero() {
    let cfg = write_config(
        r#"
[sessions.dev]
host = "10.0.0.1"
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "proc",
            "--from-config",
            "/nonexistent/procedures.toml",
            "run",
            "deploy",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("procedures"));
}

// ── proc list ────────────────────────────────────────────────────────────────

#[test]
fn proc_list_shows_all_procedures() {
    russh()
        .args(["proc", "--from-config", &fixture("procedures.toml"), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backup"))
        .stdout(predicate::str::contains("deploy"))
        .stdout(predicate::str::contains("healthcheck"));
}

#[test]
fn proc_list_shows_table_headers() {
    russh()
        .args(["proc", "--from-config", &fixture("procedures.toml"), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("SESSION"))
        .stdout(predicate::str::contains("DESCRIPTION"))
        .stdout(predicate::str::contains("TAGS"));
}

#[test]
fn proc_list_empty_config_prints_no_procedures() {
    let cfg = write_config("[procedures]\n");
    russh()
        .args([
            "proc",
            "--from-config",
            cfg.path().to_str().unwrap(),
            "list",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("No procedures configured"));
}

#[test]
fn proc_list_missing_config_exits_nonzero() {
    russh()
        .args([
            "proc",
            "--from-config",
            "/nonexistent/path/procedures.toml",
            "list",
        ])
        .assert()
        .failure();
}

// ── proc show ────────────────────────────────────────────────────────────────

#[test]
fn proc_show_known_procedure_displays_details() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "proc",
            "--from-config",
            &fixture("procedures.toml"),
            "show",
            "backup",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("backup"))
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("Run daily backup script"));
}

#[test]
fn proc_show_displays_commands() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "proc",
            "--from-config",
            &fixture("procedures.toml"),
            "show",
            "backup",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd /opt/backup"))
        .stdout(predicate::str::contains("./run-backup.sh"));
}

#[test]
fn proc_show_displays_resolved_session() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "proc",
            "--from-config",
            &fixture("procedures.toml"),
            "show",
            "backup",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved session"))
        .stdout(predicate::str::contains("10.0.0.1"));
}

#[test]
fn proc_show_displays_ssh_command_preview() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "proc",
            "--from-config",
            &fixture("procedures.toml"),
            "show",
            "backup",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("SSH command preview"))
        .stdout(predicate::str::contains("ssh"));
}

#[test]
fn proc_show_unknown_procedure_exits_nonzero() {
    russh()
        .args([
            "proc",
            "--from-config",
            &fixture("procedures.toml"),
            "show",
            "nosuchproc",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nosuchproc"));
}

// ── proc check ───────────────────────────────────────────────────────────────

#[test]
fn proc_check_clean_config_exits_zero() {
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "proc",
            "--from-config",
            &fixture("procedures.toml"),
            "check",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn proc_check_missing_session_ref_exits_two() {
    let cfg = write_config(
        r#"
[procedures.broken]
session = "nonexistent"
commands = ["echo hello"]
"#,
    );
    russh()
        .args([
            "--config",
            &fixture("three_sessions.toml"),
            "proc",
            "--from-config",
            cfg.path().to_str().unwrap(),
            "check",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("Error"));
}

#[test]
fn proc_check_missing_config_exits_two() {
    russh()
        .args([
            "proc",
            "--from-config",
            "/nonexistent/procedures.toml",
            "check",
        ])
        .assert()
        .code(2);
}

// ── no-subcommand (menu entry path) ──────────────────────────────────────────

#[test]
fn no_subcommand_invokes_menu_path() {
    // Without a TTY the menu prompter fails, but the process should NOT exit
    // with a clap "unrecognised subcommand" error — it should enter the menu
    // code path and fail there (non-zero exit is fine in a non-TTY environment).
    let cfg = write_config(
        r#"
[sessions.demo]
host = "10.0.0.1"
"#,
    );
    let output = russh()
        .args(["--config", cfg.path().to_str().unwrap()])
        .output()
        .unwrap();

    // Must NOT complain about an unrecognised subcommand.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unrecognized subcommand"),
        "expected menu path, got clap error: {stderr}"
    );
}

// ── edit ─────────────────────────────────────────────────────────────────────

#[test]
fn edit_set_port() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
username = "alice"
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "edit",
            "foo",
            "-p",
            "888",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated"));

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert!(content.contains("port = 888"), "port not set: {content}");
    // Original fields preserved
    assert!(
        content.contains("username = \"alice\""),
        "username lost: {content}"
    );
}

#[test]
fn edit_remove_port_with_none() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
port = 2222
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "edit",
            "foo",
            "-p",
            "NONE",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert!(!content.contains("port"), "port not removed: {content}");
    assert!(
        content.contains("host = \"10.0.0.1\""),
        "host lost: {content}"
    );
}

#[test]
fn edit_remove_ssh_key_with_none() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
ssh_key = "~/.ssh/id_rsa"
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "edit",
            "foo",
            "-i",
            "NONE",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert!(
        !content.contains("ssh_key"),
        "ssh_key not removed: {content}"
    );
}

#[test]
fn edit_nonexistent_session_fails() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "edit",
            "bar",
            "-p",
            "22",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn edit_no_flags_fails() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
"#,
    );
    russh()
        .args(["--config", cfg.path().to_str().unwrap(), "edit", "foo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no fields to edit"));
}

#[test]
fn edit_preserves_other_sessions() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
port = 22

[sessions.bar]
host = "10.0.0.2"
username = "bob"
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "edit",
            "foo",
            "-p",
            "9999",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert!(
        content.contains("port = 9999"),
        "port not updated: {content}"
    );
    assert!(
        content.contains("[sessions.bar]"),
        "other session lost: {content}"
    );
    assert!(
        content.contains("username = \"bob\""),
        "other session data lost: {content}"
    );
}

#[test]
fn edit_change_host_and_user() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
username = "alice"
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "edit",
            "foo",
            "--host",
            "192.168.1.1",
            "--user",
            "deploy",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert!(
        content.contains("host = \"192.168.1.1\""),
        "host not updated: {content}"
    );
    assert!(
        content.contains("username = \"deploy\""),
        "user not updated: {content}"
    );
}

#[test]
fn edit_none_as_session_name_rejected() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
"#,
    );
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "edit",
            "NONE",
            "-p",
            "22",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("reserved keyword"));
}

// ── insert: NONE rejection ──────────────────────────────────────────────────

#[test]
fn insert_none_as_session_name_rejected() {
    let cfg = write_config("[sessions]\n");
    russh()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "insert",
            "NONE",
            "user@host",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("reserved keyword"));
}

// ── edit: no-name opens editor (error when no $EDITOR) ──────────────────────

#[test]
fn edit_no_name_no_editor_fails() {
    let cfg = write_config(
        r#"
[sessions.foo]
host = "10.0.0.1"
"#,
    );
    russh()
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .args(["--config", cfg.path().to_str().unwrap(), "edit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("EDITOR").or(predicate::str::contains("VISUAL")));
}
