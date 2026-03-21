use crate::model::ResolvedSession;
use std::io;
use std::process::Command;

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
        let err = Command::new(&spec.executable)
            .args(&spec.args)
            .exec();
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
            vec!["-p", "22", "-i", "/home/admin/.ssh/id_rsa", "admin@10.0.0.1"]
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
        assert!(spec.display.starts_with("ssh "), "display: {}", spec.display);
    }

    #[test]
    fn high_port_number_in_args() {
        let spec = build_command(&make_resolved(|s| s.port = 65535));
        assert!(spec.args.contains(&"65535".to_string()));
    }
}
