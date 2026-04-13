use std::path::Path;

use crate::model::{Procedure, ResolvedSession, Session, Severity, ValidationIssue};
use crate::resolve::resolve_session_with_jump;
use crate::ssh::CommandSpec;

// ---------------------------------------------------------------------------
// A procedure resolved with its full session
// ---------------------------------------------------------------------------

/// A procedure ready for execution, with its session fully resolved.
#[derive(Debug, Clone)]
pub struct ExecutableProcedure {
    pub name: String,
    pub session: ResolvedSession,
    pub commands: Vec<String>,
    pub no_tty: bool,
    pub fail_fast: bool,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

/// Resolve a procedure by looking up its session in the session list.
///
/// Returns `None` if the referenced session does not exist.
pub fn resolve_procedure(proc: &Procedure, sessions: &[Session]) -> Option<ExecutableProcedure> {
    let session = sessions.iter().find(|s| s.name == proc.session)?;
    let resolved = resolve_session_with_jump(session, sessions);
    Some(ExecutableProcedure {
        name: proc.name.clone(),
        session: resolved,
        commands: proc.commands.clone(),
        no_tty: proc.no_tty,
        fail_fast: proc.fail_fast,
        tags: proc.tags.clone(),
    })
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate a procedure for launch-blocking errors.
///
/// Checks that the procedure references an existing session.
/// (Empty commands and missing session are already caught by `proc_config`
/// at parse time, but we guard here too for defense in depth.)
pub fn validate_procedure(proc: &Procedure, sessions: &[Session]) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if proc.commands.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: None,
            procedure_name: Some(proc.name.clone()),
            field: Some("commands".into()),
            message: format!("procedure \"{}\" has no commands", proc.name),
            code: Some("empty-commands".into()),
        });
    }

    if !sessions.iter().any(|s| s.name == proc.session) {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: None,
            procedure_name: Some(proc.name.clone()),
            field: Some("session".into()),
            message: format!(
                "procedure \"{}\" references unknown session \"{}\"",
                proc.name, proc.session
            ),
            code: Some("unknown-session".into()),
        });
    }

    issues
}

// ---------------------------------------------------------------------------
// SSH command building
// ---------------------------------------------------------------------------

/// Build an SSH command spec for running a procedure on a remote host.
///
/// Produces: `ssh [-T] [-J jump] -p port [-i key] user@host 'cmd1 && cmd2'`
/// (or `cmd1 ; cmd2` if `fail_fast` is false).
pub fn build_procedure_command(proc: &ExecutableProcedure) -> CommandSpec {
    let mut args = Vec::new();

    if proc.no_tty {
        args.push("-T".into());
    }

    if let Some(ref jump) = proc.session.jump_target {
        args.push("-J".into());
        args.push(jump.clone());
    }

    args.push("-p".into());
    args.push(proc.session.port.to_string());

    if let Some(ref key) = proc.session.ssh_key {
        args.push("-i".into());
        args.push(key.clone());
    }

    let destination = format!("{}@{}", proc.session.username, proc.session.host);
    args.push(destination);

    let joiner = if proc.fail_fast { " && " } else { " ; " };
    let remote_cmd = proc.commands.join(joiner);
    args.push(remote_cmd);

    let display = format!("ssh {}", args.join(" "));

    CommandSpec {
        executable: "ssh".into(),
        args,
        display,
    }
}

/// Build an SSH command that pipes a local script to the remote host via stdin.
///
/// Produces: `ssh [-T] [-J jump] -p port [-i key] user@host 'bash -s'`
pub fn build_script_command(session: &ResolvedSession, no_tty: bool) -> CommandSpec {
    let mut args = Vec::new();

    if no_tty {
        args.push("-T".into());
    }

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
    args.push("bash -s".into());

    let display = format!("ssh {}", args.join(" "));

    CommandSpec {
        executable: "ssh".into(),
        args,
        display,
    }
}

// ---------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------

/// Error returned when spawning an SSH process fails.
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("ssh binary not found: {0}")]
    NotFound(std::io::Error),

    #[error("failed to spawn ssh: {0}")]
    SpawnFailed(std::io::Error),

    #[error("failed to open file: {0}")]
    FileError(std::io::Error),
}

/// Spawn an SSH process and wait for it to complete.
///
/// Unlike `exec_ssh` (which replaces the process), this spawns a child
/// and returns its exit code.
pub fn spawn_ssh(spec: &CommandSpec) -> Result<i32, SpawnError> {
    let status = std::process::Command::new(&spec.executable)
        .args(&spec.args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SpawnError::NotFound(e)
            } else {
                SpawnError::SpawnFailed(e)
            }
        })?;

    Ok(status.code().unwrap_or(1))
}

/// Spawn an SSH process, redirecting stdout/stderr to a log file.
pub fn spawn_ssh_with_log(spec: &CommandSpec, log_path: &Path) -> Result<i32, SpawnError> {
    let log_file = std::fs::File::create(log_path).map_err(SpawnError::FileError)?;
    let log_stderr = log_file.try_clone().map_err(SpawnError::FileError)?;

    let status = std::process::Command::new(&spec.executable)
        .args(&spec.args)
        .stdin(std::process::Stdio::inherit())
        .stdout(log_file)
        .stderr(log_stderr)
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SpawnError::NotFound(e)
            } else {
                SpawnError::SpawnFailed(e)
            }
        })?;

    Ok(status.code().unwrap_or(1))
}

/// Spawn an SSH process that reads a local script from stdin.
pub fn spawn_ssh_with_script(spec: &CommandSpec, script_path: &Path) -> Result<i32, SpawnError> {
    let script_file = std::fs::File::open(script_path).map_err(SpawnError::FileError)?;

    let status = std::process::Command::new(&spec.executable)
        .args(&spec.args)
        .stdin(script_file)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SpawnError::NotFound(e)
            } else {
                SpawnError::SpawnFailed(e)
            }
        })?;

    Ok(status.code().unwrap_or(1))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{KeySource, Procedure, ResolvedSession, Session};

    fn make_sessions() -> Vec<Session> {
        vec![
            Session {
                name: "prod".into(),
                host: "10.0.0.1".into(),
                username: Some("deploy".into()),
                ssh_key: None,
                port: Some(22),
                tags: vec![],
                jump: None,
            },
            Session {
                name: "dev".into(),
                host: "10.0.0.2".into(),
                username: Some("admin".into()),
                ssh_key: None,
                port: Some(2222),
                tags: vec![],
                jump: None,
            },
        ]
    }

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

    fn make_proc(name: &str, session: &str, commands: Vec<&str>) -> Procedure {
        Procedure {
            name: name.into(),
            session: session.into(),
            commands: commands.into_iter().map(String::from).collect(),
            description: None,
            no_tty: false,
            fail_fast: true,
            tags: vec![],
        }
    }

    // --- Resolution ---

    #[test]
    fn resolve_known_session() {
        let proc = make_proc("deploy", "prod", vec!["echo hi"]);
        let resolved = resolve_procedure(&proc, &make_sessions()).unwrap();
        assert_eq!(resolved.session.host, "10.0.0.1");
        assert_eq!(resolved.session.username, "deploy");
    }

    #[test]
    fn resolve_unknown_session_returns_none() {
        let proc = make_proc("bad", "nonexistent", vec!["echo hi"]);
        assert!(resolve_procedure(&proc, &make_sessions()).is_none());
    }

    // --- Validation ---

    #[test]
    fn validate_valid_procedure() {
        let proc = make_proc("ok", "prod", vec!["uptime"]);
        let issues = validate_procedure(&proc, &make_sessions());
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_unknown_session() {
        let proc = make_proc("bad", "ghost", vec!["echo hi"]);
        let issues = validate_procedure(&proc, &make_sessions());
        assert!(issues
            .iter()
            .any(|i| i.code.as_deref() == Some("unknown-session")));
    }

    #[test]
    fn validate_empty_commands() {
        let mut proc = make_proc("empty", "prod", vec![]);
        proc.commands.clear();
        let issues = validate_procedure(&proc, &make_sessions());
        assert!(issues
            .iter()
            .any(|i| i.code.as_deref() == Some("empty-commands")));
    }

    // --- Command building ---

    #[test]
    fn build_command_fail_fast() {
        let exec = ExecutableProcedure {
            name: "test".into(),
            session: make_resolved(|_| {}),
            commands: vec!["cd /app".into(), "make build".into()],
            no_tty: false,
            fail_fast: true,
            tags: vec![],
        };
        let spec = build_procedure_command(&exec);
        assert_eq!(spec.executable, "ssh");
        assert_eq!(spec.args.last().unwrap(), "cd /app && make build");
    }

    #[test]
    fn build_command_no_fail_fast() {
        let exec = ExecutableProcedure {
            name: "test".into(),
            session: make_resolved(|_| {}),
            commands: vec!["cmd1".into(), "cmd2".into()],
            no_tty: false,
            fail_fast: false,
            tags: vec![],
        };
        let spec = build_procedure_command(&exec);
        assert_eq!(spec.args.last().unwrap(), "cmd1 ; cmd2");
    }

    #[test]
    fn build_command_with_no_tty() {
        let exec = ExecutableProcedure {
            name: "test".into(),
            session: make_resolved(|_| {}),
            commands: vec!["uptime".into()],
            no_tty: true,
            fail_fast: true,
            tags: vec![],
        };
        let spec = build_procedure_command(&exec);
        assert_eq!(spec.args[0], "-T");
    }

    #[test]
    fn build_command_with_jump_host() {
        let exec = ExecutableProcedure {
            name: "test".into(),
            session: make_resolved(|s| {
                s.jump_target = Some("ops@bastion:22".into());
            }),
            commands: vec!["uptime".into()],
            no_tty: false,
            fail_fast: true,
            tags: vec![],
        };
        let spec = build_procedure_command(&exec);
        assert!(spec.args.contains(&"-J".to_string()));
        assert!(spec.args.contains(&"ops@bastion:22".to_string()));
    }

    #[test]
    fn build_command_with_ssh_key() {
        let exec = ExecutableProcedure {
            name: "test".into(),
            session: make_resolved(|s| {
                s.ssh_key = Some("/keys/mykey".into());
                s.key_source = KeySource::Explicit;
            }),
            commands: vec!["uptime".into()],
            no_tty: false,
            fail_fast: true,
            tags: vec![],
        };
        let spec = build_procedure_command(&exec);
        assert!(spec.args.contains(&"-i".to_string()));
        assert!(spec.args.contains(&"/keys/mykey".to_string()));
    }

    #[test]
    fn build_script_command_basic() {
        let session = make_resolved(|_| {});
        let spec = build_script_command(&session, true);
        assert_eq!(spec.args[0], "-T");
        assert_eq!(spec.args.last().unwrap(), "bash -s");
    }

    #[test]
    fn build_script_command_no_tty_false() {
        let session = make_resolved(|_| {});
        let spec = build_script_command(&session, false);
        assert_ne!(spec.args[0], "-T");
    }
}
