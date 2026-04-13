use crate::model::ResolvedSession;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const SSH_BINARY: &str = "ssh";

/// A fully built SSH command ready for execution.
#[derive(Debug, Clone)]
pub struct CommandSpec {
    /// The executable name (e.g. `ssh`).
    pub executable: String,
    /// Arguments to pass to the executable.
    pub args: Vec<String>,
    /// Human-readable display string (e.g. `ssh -p 2222 -i /key admin@host`).
    pub display: String,
}

/// Builds a [`CommandSpec`] from a [`ResolvedSession`].
///
/// Constructs the SSH argument list: `-p port`, `-i key` (if configured),
/// and `user@host` as the final positional argument.
pub fn build_command(session: &ResolvedSession) -> CommandSpec {
    let mut args = Vec::new();

    // Jump host (ProxyJump) — must come before other args
    if let Some(ref jump) = session.jump_target {
        args.push("-J".into());
        args.push(jump.clone());
    }

    args.push("-p".into());
    args.push(session.port.to_string());

    if let Some(ref key) = session.ssh_key {
        args.push("-i".into());
        args.push(key.clone());
    }

    let destination = format!("{}@{}", session.username, session.host);
    args.push(destination);

    let display = format!("{} {}", SSH_BINARY, args.join(" "));

    CommandSpec {
        executable: SSH_BINARY.into(),
        args,
        display,
    }
}

/// Builds a [`CommandSpec`] for running a procedure (list of shell commands) on a remote host.
///
/// Like [`build_command`] but:
/// - Adds `-T` to disable pseudo-terminal allocation when `no_tty` is true
/// - Appends `shell_command` (the joined commands) as the final SSH argument after destination
pub fn build_procedure_command(
    session: &ResolvedSession,
    shell_command: &str,
    no_tty: bool,
) -> CommandSpec {
    let mut args = Vec::new();

    if let Some(ref jump) = session.jump_target {
        args.push("-J".into());
        args.push(jump.clone());
    }

    if no_tty {
        args.push("-T".into());
    }

    args.push("-p".into());
    args.push(session.port.to_string());

    if let Some(ref key) = session.ssh_key {
        args.push("-i".into());
        args.push(key.clone());
    }

    let destination = format!("{}@{}", session.username, session.host);
    args.push(destination);

    args.push(shell_command.into());

    let display = format!("{} {}", SSH_BINARY, args.join(" "));

    CommandSpec {
        executable: SSH_BINARY.into(),
        args,
        display,
    }
}

/// Builds a [`CommandSpec`] for `--from-script` mode where stdin is piped to the remote host.
///
/// Like [`build_command`] but:
/// - Adds `-T` to disable pseudo-terminal allocation when `no_tty` is true
/// - Does **not** append a remote command (stdin will be piped instead)
pub fn build_script_command(session: &ResolvedSession, no_tty: bool) -> CommandSpec {
    let mut args = Vec::new();

    if let Some(ref jump) = session.jump_target {
        args.push("-J".into());
        args.push(jump.clone());
    }

    if no_tty {
        args.push("-T".into());
    }

    args.push("-p".into());
    args.push(session.port.to_string());

    if let Some(ref key) = session.ssh_key {
        args.push("-i".into());
        args.push(key.clone());
    }

    let destination = format!("{}@{}", session.username, session.host);
    args.push(destination);

    let display = format!("{} {}", SSH_BINARY, args.join(" "));

    CommandSpec {
        executable: SSH_BINARY.into(),
        args,
        display,
    }
}

/// Spawns SSH as a child process using the given [`CommandSpec`].
///
/// Unlike [`exec_ssh`] which replaces the current process, this spawns a child
/// and waits for it to complete — suitable for procedure execution where the
/// parent process needs to continue afterward.
///
/// - `log_path`: if `Some`, redirects stdout and stderr to this file
/// - `stdin_file`: if `Some`, pipes this local file into the SSH process's stdin
pub fn spawn_ssh(
    spec: &CommandSpec,
    log_path: Option<&Path>,
    stdin_file: Option<&Path>,
) -> Result<ExitStatus, ExecError> {
    let mut cmd = Command::new(&spec.executable);
    cmd.args(&spec.args);

    if let Some(path) = stdin_file {
        let file = File::open(path).map_err(ExecError::ExecFailed)?;
        cmd.stdin(Stdio::from(file));
    }

    if let Some(path) = log_path {
        let log = File::create(path).map_err(ExecError::ExecFailed)?;
        let log_err = log.try_clone().map_err(ExecError::ExecFailed)?;
        cmd.stdout(Stdio::from(log));
        cmd.stderr(Stdio::from(log_err));
    }

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            ExecError::NotFound(e)
        } else {
            ExecError::ExecFailed(e)
        }
    })?;

    child.wait().map_err(ExecError::ExecFailed)
}

/// Error returned when the ssh binary cannot be found or exec fails.
#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("ssh binary not found: {0}")]
    NotFound(io::Error),

    #[error("failed to exec ssh: {0}")]
    ExecFailed(io::Error),
}

/// Replaces the current process with `ssh` using the given [`CommandSpec`].
///
/// On Unix, this uses `exec` and never returns on success. On non-Unix
/// platforms, it spawns the process and exits with its status code.
pub fn exec_ssh(spec: &CommandSpec) -> Result<(), ExecError> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // exec replaces the process — only returns on error
        let err = Command::new(&spec.executable).args(&spec.args).exec();
        // If we reach here, exec failed
        if err.kind() == io::ErrorKind::NotFound {
            Err(ExecError::NotFound(err))
        } else {
            Err(ExecError::ExecFailed(err))
        }
    }

    #[cfg(not(unix))]
    {
        let mut child = Command::new(&spec.executable)
            .args(&spec.args)
            .spawn()
            .map_err(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    ExecError::NotFound(e)
                } else {
                    ExecError::ExecFailed(e)
                }
            })?;
        let status = child.wait().map_err(ExecError::ExecFailed)?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{KeySource, ResolvedSession};

    fn make_resolved(overrides: impl FnOnce(&mut ResolvedSession)) -> ResolvedSession {
        let mut s = ResolvedSession {
            name: "test".into(),
            host: "10.0.0.1".into(),
            username: "admin".into(),
            port: 22,
            ssh_key: None,
            key_source: KeySource::SystemDefault,
            display_target: "admin@10.0.0.1:22".into(),
            tags: vec![],
            jump_target: None,
        };
        overrides(&mut s);
        s
    }

    #[test]
    fn basic_command_no_key() {
        let spec = build_command(&make_resolved(|_| {}));
        assert_eq!(spec.executable, "ssh");
        assert_eq!(spec.args, vec!["-p", "22", "admin@10.0.0.1"]);
    }

    #[test]
    fn command_with_custom_port() {
        let spec = build_command(&make_resolved(|s| s.port = 2222));
        assert_eq!(spec.args, vec!["-p", "2222", "admin@10.0.0.1"]);
    }

    #[test]
    fn command_with_ssh_key() {
        let spec = build_command(&make_resolved(|s| {
            s.ssh_key = Some("/home/admin/.ssh/id_rsa".into());
            s.key_source = KeySource::Explicit;
        }));
        assert_eq!(
            spec.args,
            vec![
                "-p",
                "22",
                "-i",
                "/home/admin/.ssh/id_rsa",
                "admin@10.0.0.1"
            ]
        );
    }

    #[test]
    fn command_with_key_and_custom_port() {
        let spec = build_command(&make_resolved(|s| {
            s.port = 8022;
            s.ssh_key = Some("/keys/mykey".into());
        }));
        assert_eq!(
            spec.args,
            vec!["-p", "8022", "-i", "/keys/mykey", "admin@10.0.0.1"]
        );
    }

    #[test]
    fn display_string_no_key() {
        let spec = build_command(&make_resolved(|_| {}));
        assert_eq!(spec.display, "ssh -p 22 admin@10.0.0.1");
    }

    #[test]
    fn display_string_with_key() {
        let spec = build_command(&make_resolved(|s| {
            s.ssh_key = Some("/keys/k".into());
        }));
        assert_eq!(spec.display, "ssh -p 22 -i /keys/k admin@10.0.0.1");
    }

    #[test]
    fn destination_format() {
        let spec = build_command(&make_resolved(|s| {
            s.username = "deploy".into();
            s.host = "prod.example.com".into();
        }));
        let dest = spec.args.last().unwrap();
        assert_eq!(dest, "deploy@prod.example.com");
    }

    #[test]
    fn args_order_port_before_key_before_destination() {
        let spec = build_command(&make_resolved(|s| {
            s.port = 2222;
            s.ssh_key = Some("/keys/k".into());
        }));
        let p_idx = spec.args.iter().position(|a| a == "-p").unwrap();
        let i_idx = spec.args.iter().position(|a| a == "-i").unwrap();
        let dest_idx = spec.args.len() - 1;
        assert!(p_idx < i_idx, "-p must come before -i");
        assert!(i_idx < dest_idx, "-i must come before destination");
    }

    #[test]
    fn ipv6_host_destination() {
        let spec = build_command(&make_resolved(|s| {
            s.host = "::1".into();
            s.username = "root".into();
        }));
        let dest = spec.args.last().unwrap();
        assert_eq!(dest, "root@::1");
    }

    #[test]
    fn command_spec_clone() {
        let spec = build_command(&make_resolved(|_| {}));
        let cloned = spec.clone();
        assert_eq!(cloned.executable, spec.executable);
        assert_eq!(cloned.args, spec.args);
        assert_eq!(cloned.display, spec.display);
    }

    #[test]
    fn display_contains_executable() {
        let spec = build_command(&make_resolved(|_| {}));
        assert!(
            spec.display.starts_with("ssh "),
            "display: {}",
            spec.display
        );
    }

    #[test]
    fn high_port_number_in_args() {
        let spec = build_command(&make_resolved(|s| s.port = 65535));
        assert!(spec.args.contains(&"65535".to_string()));
    }

    // --- build_procedure_command ---

    #[test]
    fn procedure_command_appends_shell_command() {
        let spec = build_procedure_command(
            &make_resolved(|_| {}),
            "echo hello && echo world",
            false,
        );
        assert_eq!(spec.args.last().unwrap(), "echo hello && echo world");
    }

    #[test]
    fn procedure_command_no_tty_adds_flag() {
        let spec = build_procedure_command(&make_resolved(|_| {}), "ls", true);
        assert!(spec.args.contains(&"-T".to_string()));
    }

    #[test]
    fn procedure_command_tty_no_flag() {
        let spec = build_procedure_command(&make_resolved(|_| {}), "ls", false);
        assert!(!spec.args.contains(&"-T".to_string()));
    }

    #[test]
    fn procedure_command_flag_ordering() {
        let spec = build_procedure_command(
            &make_resolved(|s| {
                s.ssh_key = Some("/keys/k".into());
            }),
            "cmd",
            true,
        );
        let t_idx = spec.args.iter().position(|a| a == "-T").unwrap();
        let p_idx = spec.args.iter().position(|a| a == "-p").unwrap();
        let i_idx = spec.args.iter().position(|a| a == "-i").unwrap();
        let dest_idx = spec.args.len() - 2; // destination is second-to-last
        let cmd_idx = spec.args.len() - 1; // shell command is last
        assert!(t_idx < p_idx, "-T must come before -p");
        assert!(p_idx < i_idx, "-p must come before -i");
        assert!(i_idx < dest_idx, "-i must come before destination");
        assert!(dest_idx < cmd_idx, "destination must come before shell command");
    }

    #[test]
    fn procedure_command_with_jump_host() {
        let spec = build_procedure_command(
            &make_resolved(|s| {
                s.jump_target = Some("ops@bastion:2222".into());
            }),
            "uptime",
            false,
        );
        assert_eq!(spec.args[0], "-J");
        assert_eq!(spec.args[1], "ops@bastion:2222");
        assert_eq!(spec.args.last().unwrap(), "uptime");
    }

    #[test]
    fn procedure_command_display_includes_shell_command() {
        let spec = build_procedure_command(&make_resolved(|_| {}), "echo hi", false);
        assert!(
            spec.display.contains("echo hi"),
            "display: {}",
            spec.display
        );
    }

    // --- build_script_command ---

    #[test]
    fn script_command_no_remote_command() {
        let spec = build_script_command(&make_resolved(|_| {}), false);
        // Last arg should be destination, no shell command after it
        assert_eq!(spec.args.last().unwrap(), "admin@10.0.0.1");
    }

    #[test]
    fn script_command_no_tty_adds_flag() {
        let spec = build_script_command(&make_resolved(|_| {}), true);
        assert!(spec.args.contains(&"-T".to_string()));
    }

    #[test]
    fn script_command_tty_no_flag() {
        let spec = build_script_command(&make_resolved(|_| {}), false);
        assert!(!spec.args.contains(&"-T".to_string()));
    }

    #[test]
    fn script_command_with_key_and_no_tty() {
        let spec = build_script_command(
            &make_resolved(|s| {
                s.ssh_key = Some("/keys/k".into());
                s.port = 2222;
            }),
            true,
        );
        assert_eq!(
            spec.args,
            vec!["-T", "-p", "2222", "-i", "/keys/k", "admin@10.0.0.1"]
        );
    }

    #[test]
    fn script_command_display_no_remote_command() {
        let spec = build_script_command(&make_resolved(|_| {}), true);
        assert!(
            spec.display.ends_with("admin@10.0.0.1"),
            "display should end with destination: {}",
            spec.display
        );
    }

    // --- spawn_ssh ---

    #[test]
    fn spawn_ssh_not_found_returns_error() {
        let spec = CommandSpec {
            executable: "/nonexistent/binary".into(),
            args: vec![],
            display: "/nonexistent/binary".into(),
        };
        let result = spawn_ssh(&spec, None, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            ExecError::NotFound(_) => {}
            other => panic!("expected NotFound, got: {other}"),
        }
    }

    #[test]
    fn spawn_ssh_runs_simple_command() {
        let spec = CommandSpec {
            executable: "true".into(),
            args: vec![],
            display: "true".into(),
        };
        let status = spawn_ssh(&spec, None, None).unwrap();
        assert!(status.success());
    }

    #[test]
    fn spawn_ssh_captures_exit_code() {
        let spec = CommandSpec {
            executable: "false".into(),
            args: vec![],
            display: "false".into(),
        };
        let status = spawn_ssh(&spec, None, None).unwrap();
        assert!(!status.success());
    }

    #[test]
    fn spawn_ssh_with_log_path() {
        let dir = std::env::temp_dir().join("russh_test_log");
        let log_path = dir.join("test.log");
        std::fs::create_dir_all(&dir).unwrap();

        let spec = CommandSpec {
            executable: "echo".into(),
            args: vec!["hello from ssh".into()],
            display: "echo hello from ssh".into(),
        };
        let status = spawn_ssh(&spec, Some(&log_path), None).unwrap();
        assert!(status.success());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(
            content.contains("hello from ssh"),
            "log content: {content}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn spawn_ssh_with_stdin_file() {
        let dir = std::env::temp_dir().join("russh_test_stdin");
        let input_path = dir.join("input.txt");
        let output_path = dir.join("output.txt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&input_path, "piped input\n").unwrap();

        let spec = CommandSpec {
            executable: "cat".into(),
            args: vec![],
            display: "cat".into(),
        };
        let status = spawn_ssh(&spec, Some(&output_path), Some(&input_path)).unwrap();
        assert!(status.success());

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(
            content.contains("piped input"),
            "output content: {content}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn spawn_ssh_stdin_file_not_found() {
        let spec = CommandSpec {
            executable: "cat".into(),
            args: vec![],
            display: "cat".into(),
        };
        let result = spawn_ssh(&spec, None, Some(Path::new("/nonexistent/input.txt")));
        assert!(result.is_err());
    }
}
