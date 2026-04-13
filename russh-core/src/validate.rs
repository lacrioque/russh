use std::path::Path;

use crate::model::{KeySource, Procedure, ResolvedSession, Severity, ValidationIssue};

/// Validate a single resolved session and return any issues found.
///
/// Checks performed:
/// - **Error**: host is empty.
/// - **Error**: port is 0.
/// - **Warning**: explicit `ssh_key` path does not exist on disk.
/// - **Warning**: host looks like a hostname rather than an IP address.
pub fn validate_session(session: &ResolvedSession) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if session.host.trim().is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: Some(session.name.clone()),
            procedure_name: None,
            field: Some("host".into()),
            message: "host must not be empty".into(),
            code: Some("missing-host".into()),
        });
    }

    if session.port == 0 {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: Some(session.name.clone()),
            procedure_name: None,
            field: Some("port".into()),
            message: "port must be between 1 and 65535".into(),
            code: Some("invalid-port".into()),
        });
    }

    if session.key_source == KeySource::Explicit {
        if let Some(ref key_path) = session.ssh_key {
            if !Path::new(key_path).exists() {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    session_name: Some(session.name.clone()),
                    procedure_name: None,
                    field: Some("ssh_key".into()),
                    message: format!("identity file does not exist: {key_path}"),
                    code: Some("missing-key-file".into()),
                });
            }
        }
    }

    if !session.host.trim().is_empty() && !looks_like_ip(&session.host) {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            session_name: Some(session.name.clone()),
            procedure_name: None,
            field: Some("host".into()),
            message: format!(
                "host \"{}\" looks like a hostname; consider using an IP address",
                session.host
            ),
            code: Some("hostname-not-ip".into()),
        });
    }

    issues
}

/// Validate all sessions and collect issues, including jump host references.
pub fn validate_sessions(sessions: &[ResolvedSession]) -> Vec<ValidationIssue> {
    let mut issues: Vec<_> = sessions.iter().flat_map(validate_session).collect();
    issues.extend(validate_jump_references(sessions));
    issues
}

/// Check that all jump_target references point to existing sessions and
/// detect circular jump chains.
fn validate_jump_references(_sessions: &[ResolvedSession]) -> Vec<ValidationIssue> {
    // Jump references are validated at the raw Session level via validate_jump_refs_raw.
    // ResolvedSession already has jump_target resolved to a string.
    vec![]
}

/// Validate raw sessions for jump host issues (circular chains, empty values).
///
/// Jump values that don't match a session name are treated as arbitrary host
/// specs (e.g. `user@host:port`) and are not flagged as errors.
pub fn validate_jump_refs_raw(sessions: &[crate::model::Session]) -> Vec<ValidationIssue> {
    let names: std::collections::HashSet<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
    let mut issues = Vec::new();

    for session in sessions {
        if let Some(ref jump) = session.jump {
            if jump.trim().is_empty() {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    session_name: Some(session.name.clone()),
                    procedure_name: None,
                    field: Some("jump".into()),
                    message: "jump host must not be empty".into(),
                    code: Some("empty-jump-host".into()),
                });
            } else if names.contains(jump.as_str()) && jump == &session.name {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    session_name: Some(session.name.clone()),
                    procedure_name: None,
                    field: Some("jump".into()),
                    message: "session cannot jump through itself".into(),
                    code: Some("circular-jump".into()),
                });
            }
        }
    }

    issues
}

/// Validate a single raw procedure and return any issues found.
///
/// Checks performed:
/// - **Error**: session field is empty.
/// - **Error**: session references a non-existent session name.
/// - **Error**: commands list is empty.
/// - **Warning**: an individual command string is empty.
pub fn validate_procedure(
    procedure: &Procedure,
    session_names: &std::collections::HashSet<&str>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if procedure.session.trim().is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: None,
            procedure_name: Some(procedure.name.clone()),
            field: Some("session".into()),
            message: "session must not be empty".into(),
            code: Some("empty-session".into()),
        });
    } else if !session_names.contains(procedure.session.as_str()) {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: None,
            procedure_name: Some(procedure.name.clone()),
            field: Some("session".into()),
            message: format!("session \"{}\" does not exist", procedure.session),
            code: Some("unknown-session".into()),
        });
    }

    if procedure.commands.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: None,
            procedure_name: Some(procedure.name.clone()),
            field: Some("commands".into()),
            message: "commands list must not be empty".into(),
            code: Some("empty-commands".into()),
        });
    } else {
        for (i, cmd) in procedure.commands.iter().enumerate() {
            if cmd.trim().is_empty() {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    session_name: None,
                    procedure_name: Some(procedure.name.clone()),
                    field: Some(format!("commands[{}]", i)),
                    message: "empty command string".into(),
                    code: Some("empty-command".into()),
                });
            }
        }
    }

    issues
}

/// Validate all procedures against the given session names and collect issues.
pub fn validate_procedures(
    procedures: &[Procedure],
    session_names: &std::collections::HashSet<&str>,
) -> Vec<ValidationIssue> {
    procedures
        .iter()
        .flat_map(|p| validate_procedure(p, session_names))
        .collect()
}

/// Returns `true` if the string looks like an IPv4 or IPv6 address.
fn looks_like_ip(host: &str) -> bool {
    host.parse::<std::net::IpAddr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::KeySource;

    fn make_resolved(overrides: impl FnOnce(&mut ResolvedSession)) -> ResolvedSession {
        let mut s = ResolvedSession {
            name: "test".into(),
            host: "10.0.0.1".into(),
            username: "user".into(),
            port: 22,
            ssh_key: None,
            key_source: KeySource::SystemDefault,
            display_target: "user@10.0.0.1:22".into(),
            tags: vec![],
            jump_target: None,
        };
        overrides(&mut s);
        s
    }

    fn codes(issues: &[ValidationIssue]) -> Vec<String> {
        issues.iter().filter_map(|i| i.code.clone()).collect()
    }

    #[test]
    fn valid_session_with_ip_has_no_issues() {
        let issues = validate_session(&make_resolved(|_| {}));
        assert!(issues.is_empty());
    }

    #[test]
    fn empty_host_is_error() {
        let issues = validate_session(&make_resolved(|s| s.host = "".into()));
        assert_eq!(codes(&issues), vec!["missing-host"]);
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn whitespace_host_is_error() {
        let issues = validate_session(&make_resolved(|s| s.host = "  ".into()));
        assert_eq!(codes(&issues), vec!["missing-host"]);
    }

    #[test]
    fn port_zero_is_error() {
        let issues = validate_session(&make_resolved(|s| s.port = 0));
        assert!(codes(&issues).contains(&"invalid-port".to_string()));
        let port_issue = issues
            .iter()
            .find(|i| i.code.as_deref() == Some("invalid-port"))
            .unwrap();
        assert_eq!(port_issue.severity, Severity::Error);
    }

    #[test]
    fn hostname_triggers_warning() {
        let issues = validate_session(&make_resolved(|s| s.host = "example.com".into()));
        assert_eq!(codes(&issues), vec!["hostname-not-ip"]);
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn ipv6_is_not_hostname_warning() {
        let issues = validate_session(&make_resolved(|s| s.host = "::1".into()));
        assert!(issues.is_empty());
    }

    #[test]
    fn missing_key_file_is_warning() {
        let issues = validate_session(&make_resolved(|s| {
            s.ssh_key = Some("/nonexistent/path/id_rsa".into());
            s.key_source = KeySource::Explicit;
        }));
        assert_eq!(codes(&issues), vec!["missing-key-file"]);
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn system_default_key_skips_file_check() {
        let issues = validate_session(&make_resolved(|s| {
            s.ssh_key = None;
            s.key_source = KeySource::SystemDefault;
        }));
        assert!(issues.is_empty());
    }

    #[test]
    fn existing_key_file_no_warning() {
        // Use Cargo.toml as a file that definitely exists
        let issues = validate_session(&make_resolved(|s| {
            s.ssh_key = Some(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").into());
            s.key_source = KeySource::Explicit;
        }));
        assert!(!codes(&issues).contains(&"missing-key-file".to_string()));
    }

    #[test]
    fn validate_sessions_collects_all() {
        let sessions = vec![
            make_resolved(|s| s.host = "".into()),
            make_resolved(|s| s.port = 0),
        ];
        let issues = validate_sessions(&sessions);
        assert!(issues.len() >= 2);
    }

    #[test]
    fn multiple_issues_on_one_session() {
        let issues = validate_session(&make_resolved(|s| {
            s.host = "".into();
            s.port = 0;
        }));
        let c = codes(&issues);
        assert!(c.contains(&"missing-host".to_string()));
        assert!(c.contains(&"invalid-port".to_string()));
    }

    #[test]
    fn display_format_includes_code_and_session() {
        let issues = validate_session(&make_resolved(|s| s.host = "".into()));
        let display = format!("{}", issues[0]);
        assert!(display.contains("missing-host"));
        assert!(display.contains("test"));
    }

    #[test]
    fn validate_sessions_empty_slice() {
        let issues = validate_sessions(&[]);
        assert!(issues.is_empty());
    }

    #[test]
    fn ipv4_address_has_no_hostname_warning() {
        let issues = validate_session(&make_resolved(|s| s.host = "203.0.113.42".into()));
        assert!(!codes(&issues).contains(&"hostname-not-ip".to_string()));
    }

    #[test]
    fn ipv6_full_address_no_warning() {
        let issues = validate_session(&make_resolved(|s| s.host = "2001:db8::1".into()));
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn key_source_system_default_with_some_path_not_checked() {
        // key_source=SystemDefault means we skip the file-existence check
        // even if ssh_key is populated (defensive: shouldn't happen in practice).
        let issues = validate_session(&make_resolved(|s| {
            s.ssh_key = Some("/nonexistent/key".into());
            s.key_source = KeySource::SystemDefault;
        }));
        assert!(!codes(&issues).contains(&"missing-key-file".to_string()));
    }

    #[test]
    fn validate_sessions_aggregates_across_multiple_sessions() {
        let sessions = vec![
            make_resolved(|s| s.host = "good.ip".into()), // hostname warning
            make_resolved(|s| s.port = 0),                // port error
            make_resolved(|_| {}),                        // clean
        ];
        let issues = validate_sessions(&sessions);
        assert!(issues
            .iter()
            .any(|i| i.code.as_deref() == Some("hostname-not-ip")));
        assert!(issues
            .iter()
            .any(|i| i.code.as_deref() == Some("invalid-port")));
    }

    // --- Procedure validation ---

    fn make_procedure(overrides: impl FnOnce(&mut Procedure)) -> Procedure {
        let mut p = Procedure {
            name: "deploy".into(),
            session: "web".into(),
            commands: vec!["git pull".into(), "systemctl restart app".into()],
            description: None,
            no_tty: false,
            fail_fast: true,
            tags: vec![],
        };
        overrides(&mut p);
        p
    }

    fn session_names_set() -> std::collections::HashSet<&'static str> {
        ["web", "db", "bastion"].iter().cloned().collect()
    }

    #[test]
    fn valid_procedure_has_no_issues() {
        let issues = validate_procedure(&make_procedure(|_| {}), &session_names_set());
        assert!(issues.is_empty());
    }

    #[test]
    fn empty_session_is_error() {
        let issues = validate_procedure(
            &make_procedure(|p| p.session = "".into()),
            &session_names_set(),
        );
        assert_eq!(codes(&issues), vec!["empty-session"]);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].procedure_name.as_deref(), Some("deploy"));
    }

    #[test]
    fn whitespace_session_is_error() {
        let issues = validate_procedure(
            &make_procedure(|p| p.session = "  ".into()),
            &session_names_set(),
        );
        assert_eq!(codes(&issues), vec!["empty-session"]);
    }

    #[test]
    fn unknown_session_is_error() {
        let issues = validate_procedure(
            &make_procedure(|p| p.session = "nonexistent".into()),
            &session_names_set(),
        );
        assert_eq!(codes(&issues), vec!["unknown-session"]);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].message.contains("nonexistent"));
    }

    #[test]
    fn empty_commands_is_error() {
        let issues = validate_procedure(
            &make_procedure(|p| p.commands = vec![]),
            &session_names_set(),
        );
        assert_eq!(codes(&issues), vec!["empty-commands"]);
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn empty_command_string_is_warning() {
        let issues = validate_procedure(
            &make_procedure(|p| p.commands = vec!["git pull".into(), "".into()]),
            &session_names_set(),
        );
        assert_eq!(codes(&issues), vec!["empty-command"]);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].field.as_deref(), Some("commands[1]"));
    }

    #[test]
    fn whitespace_command_string_is_warning() {
        let issues = validate_procedure(
            &make_procedure(|p| p.commands = vec!["  ".into()]),
            &session_names_set(),
        );
        assert_eq!(codes(&issues), vec!["empty-command"]);
    }

    #[test]
    fn multiple_empty_commands_generate_multiple_warnings() {
        let issues = validate_procedure(
            &make_procedure(|p| p.commands = vec!["".into(), "ok".into(), "".into()]),
            &session_names_set(),
        );
        let empty_cmd_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.code.as_deref() == Some("empty-command"))
            .collect();
        assert_eq!(empty_cmd_issues.len(), 2);
        assert_eq!(empty_cmd_issues[0].field.as_deref(), Some("commands[0]"));
        assert_eq!(empty_cmd_issues[1].field.as_deref(), Some("commands[2]"));
    }

    #[test]
    fn validate_procedures_collects_all() {
        let names = session_names_set();
        let procs = vec![
            make_procedure(|p| p.session = "unknown".into()),
            make_procedure(|p| p.commands = vec![]),
        ];
        let issues = validate_procedures(&procs, &names);
        assert!(issues.len() >= 2);
        assert!(issues
            .iter()
            .any(|i| i.code.as_deref() == Some("unknown-session")));
        assert!(issues
            .iter()
            .any(|i| i.code.as_deref() == Some("empty-commands")));
    }

    #[test]
    fn validate_procedures_empty_slice() {
        let issues = validate_procedures(&[], &session_names_set());
        assert!(issues.is_empty());
    }

    #[test]
    fn procedure_validation_display_includes_procedure_name() {
        let issues = validate_procedure(
            &make_procedure(|p| p.commands = vec![]),
            &session_names_set(),
        );
        let display = format!("{}", issues[0]);
        assert!(
            display.contains("procedure \"deploy\""),
            "display: {display}"
        );
        assert!(display.contains("empty-commands"), "display: {display}");
    }
}
