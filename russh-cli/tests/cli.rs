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
