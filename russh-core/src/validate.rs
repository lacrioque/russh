use std::path::Path;

use crate::model::{KeySource, ResolvedSession, Severity, ValidationIssue};

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
            field: Some("host".into()),
            message: "host must not be empty".into(),
            code: Some("missing-host".into()),
        });
    }

    if session.port == 0 {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            session_name: Some(session.name.clone()),
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

/// Validate all sessions and collect issues.
pub fn validate_sessions(sessions: &[ResolvedSession]) -> Vec<ValidationIssue> {
    sessions.iter().flat_map(validate_session).collect()
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
        };
        overrides(&mut s);
        s
    }

    fn codes(issues: &[ValidationIssue]) -> Vec<String> {
        issues
            .iter()
            .filter_map(|i| i.code.clone())
            .collect()
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
        let port_issue = issues.iter().find(|i| i.code.as_deref() == Some("invalid-port")).unwrap();
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
}
