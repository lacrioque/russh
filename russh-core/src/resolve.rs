use crate::model::{KeySource, Procedure, ResolvedProcedure, ResolvedSession, Session};
use crate::paths::expand_tilde;
use std::env;

const DEFAULT_PORT: u16 = 22;

/// Resolves a raw [`Session`] into a [`ResolvedSession`] with all defaults applied,
/// including jump host resolution.
///
/// If the session has a `jump` field referencing another session name, that session
/// is resolved and its `user@host:port` string is stored in `jump_target`.
/// If the `jump` value does not match any session name, it is treated as an
/// arbitrary host spec (e.g. `user@host:port`) and passed through directly.
///
/// Use this when you have the full session list available. Falls back to
/// [`resolve_session`] for sessions without a jump host.
pub fn resolve_session_with_jump(session: &Session, all_sessions: &[Session]) -> ResolvedSession {
    let mut resolved = resolve_session(session);

    if let Some(ref jump_value) = session.jump {
        if let Some(jump_session) = all_sessions.iter().find(|s| s.name == *jump_value) {
            // Jump value matches a session name — resolve it
            let jump_resolved = resolve_session(jump_session);
            resolved.jump_target = Some(format!(
                "{}@{}:{}",
                jump_resolved.username, jump_resolved.host, jump_resolved.port
            ));
        } else {
            // Not a session name — treat as arbitrary host spec (passed directly to ssh -J)
            resolved.jump_target = Some(jump_value.clone());
        }
    }

    resolved
}

/// Resolves a raw [`Procedure`] into a [`ResolvedProcedure`].
///
/// Looks up the referenced session by name from the sessions list, resolves it
/// (with jump host support), joins commands with `&&` (fail_fast=true) or `;`
/// (fail_fast=false) into `shell_command`, and normalizes tags.
///
/// Returns `None` if the referenced session name is not found in `all_sessions`.
pub fn resolve_procedure(
    procedure: &Procedure,
    all_sessions: &[Session],
) -> Option<ResolvedProcedure> {
    let session = all_sessions.iter().find(|s| s.name == procedure.session)?;
    let resolved_session = resolve_session_with_jump(session, all_sessions);

    let separator = if procedure.fail_fast { " && " } else { " ; " };
    let shell_command = procedure.commands.join(separator);

    let tags = normalize_tags(&procedure.tags);

    Some(ResolvedProcedure {
        name: procedure.name.clone(),
        session: resolved_session,
        commands: procedure.commands.clone(),
        shell_command,
        description: procedure.description.clone(),
        no_tty: procedure.no_tty,
        fail_fast: procedure.fail_fast,
        tags,
    })
}

/// Resolves a raw [`Session`] into a [`ResolvedSession`] with all defaults applied.
///
/// - Missing `username` → current OS user (`USER` on Unix, `USERNAME` on Windows).
/// - Missing `port` → 22.
/// - `ssh_key` with leading `~` → expanded via [`expand_tilde`].
/// - Tags are trimmed, deduplicated, and sorted.
///
/// Note: Does not resolve jump hosts. Use [`resolve_session_with_jump`] when
/// the full session list is available.
pub fn resolve_session(session: &Session) -> ResolvedSession {
    let username = session
        .username
        .clone()
        .unwrap_or_else(|| current_username().unwrap_or_else(|| String::from("unknown")));

    let port = session.port.unwrap_or(DEFAULT_PORT);

    let (ssh_key, key_source) = match &session.ssh_key {
        Some(raw) => (Some(expand_tilde(raw)), KeySource::Explicit),
        None => (None, KeySource::SystemDefault),
    };

    let tags = normalize_tags(&session.tags);

    let display_target = format!("{}@{}:{}", username, session.host, port);

    ResolvedSession {
        name: session.name.clone(),
        host: session.host.clone(),
        username,
        port,
        ssh_key,
        key_source,
        display_target,
        tags,
        jump_target: None,
    }
}

/// Returns the current OS username, or `None` if unavailable.
fn current_username() -> Option<String> {
    #[cfg(unix)]
    {
        env::var("USER").ok()
    }
    #[cfg(windows)]
    {
        env::var("USERNAME").ok()
    }
    #[cfg(not(any(unix, windows)))]
    {
        env::var("USER").ok()
    }
}

/// Trims whitespace, removes empty strings, deduplicates, and sorts tags.
fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut out: Vec<String> = tags
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Session;

    fn make_session(overrides: impl FnOnce(&mut Session)) -> Session {
        let mut s = Session {
            name: "test".into(),
            host: "10.0.0.1".into(),
            username: None,
            ssh_key: None,
            port: None,
            tags: vec![],
            jump: None,
        };
        overrides(&mut s);
        s
    }

    /// Acquires ENV_MUTEX to prevent races with other env-mutating tests.
    fn with_env<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
        let _lock = crate::test_util::ENV_MUTEX.lock().unwrap();
        let originals: Vec<_> = vars
            .iter()
            .map(|(k, _)| (*k, std::env::var_os(k)))
            .collect();
        for (k, v) in vars {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
        f();
        for (k, original) in &originals {
            match original {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }

    #[test]
    fn defaults_port_to_22() {
        let r = resolve_session(&make_session(|_| {}));
        assert_eq!(r.port, 22);
    }

    #[test]
    fn preserves_explicit_port() {
        let r = resolve_session(&make_session(|s| s.port = Some(2222)));
        assert_eq!(r.port, 2222);
    }

    #[test]
    fn defaults_username_to_os_user() {
        with_env(&[("USER", Some("testuser"))], || {
            let r = resolve_session(&make_session(|_| {}));
            assert_eq!(r.username, "testuser");
        });
    }

    #[test]
    fn preserves_explicit_username() {
        let r = resolve_session(&make_session(|s| s.username = Some("admin".into())));
        assert_eq!(r.username, "admin");
    }

    #[test]
    fn expands_tilde_in_ssh_key() {
        with_env(&[("HOME", Some("/fakehome"))], || {
            let r = resolve_session(&make_session(|s| {
                s.ssh_key = Some("~/.ssh/id_rsa".into());
            }));
            assert_eq!(r.ssh_key.as_deref(), Some("/fakehome/.ssh/id_rsa"));
            assert_eq!(r.key_source, KeySource::Explicit);
        });
    }

    #[test]
    fn no_key_gives_system_default() {
        let r = resolve_session(&make_session(|_| {}));
        assert!(r.ssh_key.is_none());
        assert_eq!(r.key_source, KeySource::SystemDefault);
    }

    #[test]
    fn normalizes_tags() {
        let r = resolve_session(&make_session(|s| {
            s.tags = vec![
                " prod ".into(),
                "web".into(),
                "prod".into(),
                "  ".into(),
                "web".into(),
            ];
        }));
        assert_eq!(r.tags, vec!["prod", "web"]);
    }

    #[test]
    fn display_target_format() {
        with_env(&[("USER", Some("alice"))], || {
            let r = resolve_session(&make_session(|s| {
                s.host = "example.com".into();
                s.port = Some(8022);
                s.username = Some("bob".into());
            }));
            assert_eq!(r.display_target, "bob@example.com:8022");
        });
    }

    #[test]
    fn display_target_with_defaults() {
        with_env(&[("USER", Some("alice"))], || {
            let r = resolve_session(&make_session(|s| {
                s.host = "10.0.0.1".into();
            }));
            assert_eq!(r.display_target, "alice@10.0.0.1:22");
        });
    }

    #[test]
    fn absolute_ssh_key_unchanged() {
        let r = resolve_session(&make_session(|s| {
            s.ssh_key = Some("/etc/ssh/my_key".into());
        }));
        assert_eq!(r.ssh_key.as_deref(), Some("/etc/ssh/my_key"));
    }

    #[test]
    fn empty_tags_stay_empty() {
        let r = resolve_session(&make_session(|_| {}));
        assert!(r.tags.is_empty());
    }

    #[test]
    fn unknown_fallback_when_user_not_set() {
        with_env(&[("USER", None)], || {
            let r = resolve_session(&make_session(|_| {}));
            assert_eq!(r.username, "unknown");
        });
    }

    #[test]
    fn name_and_host_carry_through() {
        let r = resolve_session(&make_session(|s| {
            s.name = "mynode".into();
            s.host = "192.168.100.200".into();
        }));
        assert_eq!(r.name, "mynode");
        assert_eq!(r.host, "192.168.100.200");
    }

    #[test]
    fn tags_whitespace_only_filtered() {
        let r = resolve_session(&make_session(|s| {
            s.tags = vec!["  ".into(), "\t".into()];
        }));
        assert!(r.tags.is_empty());
    }

    #[test]
    fn tags_sorted_alphabetically() {
        let r = resolve_session(&make_session(|s| {
            s.tags = vec!["zebra".into(), "apple".into(), "mango".into()];
        }));
        assert_eq!(r.tags, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn tilde_only_ssh_key_expands() {
        with_env(&[("HOME", Some("/home/testuser"))], || {
            let r = resolve_session(&make_session(|s| {
                s.ssh_key = Some("~".into());
            }));
            assert_eq!(r.ssh_key.as_deref(), Some("/home/testuser"));
            assert_eq!(r.key_source, KeySource::Explicit);
        });
    }

    #[test]
    fn jump_resolves_session_name() {
        let bastion = make_session(|s| {
            s.name = "bastion".into();
            s.host = "10.0.0.1".into();
            s.username = Some("ops".into());
            s.port = Some(2222);
        });
        let target = make_session(|s| {
            s.name = "internal".into();
            s.host = "10.0.1.5".into();
            s.jump = Some("bastion".into());
        });
        let sessions = vec![bastion, target.clone()];
        let resolved = resolve_session_with_jump(&target, &sessions);
        assert_eq!(resolved.jump_target, Some("ops@10.0.0.1:2222".into()));
    }

    #[test]
    fn jump_arbitrary_host_passthrough() {
        let target = make_session(|s| {
            s.jump = Some("admin@jumpbox.example.com:2222".into());
        });
        let resolved = resolve_session_with_jump(&target, &[]);
        assert_eq!(
            resolved.jump_target,
            Some("admin@jumpbox.example.com:2222".into())
        );
    }

    #[test]
    fn jump_arbitrary_host_without_user() {
        let target = make_session(|s| {
            s.jump = Some("jumpbox.example.com".into());
        });
        let resolved = resolve_session_with_jump(&target, &[]);
        assert_eq!(resolved.jump_target, Some("jumpbox.example.com".into()));
    }

    #[test]
    fn jump_prefers_session_name_over_passthrough() {
        let bastion = make_session(|s| {
            s.name = "bastion".into();
            s.host = "10.0.0.1".into();
            s.username = Some("ops".into());
            s.port = Some(22);
        });
        let target = make_session(|s| {
            s.jump = Some("bastion".into());
        });
        let sessions = vec![bastion, target.clone()];
        let resolved = resolve_session_with_jump(&target, &sessions);
        // Should resolve via the session, not pass through "bastion" literally
        assert_eq!(resolved.jump_target, Some("ops@10.0.0.1:22".into()));
    }

    #[test]
    fn jump_none_gives_no_target() {
        let target = make_session(|s| {
            s.jump = None;
        });
        let resolved = resolve_session_with_jump(&target, &[]);
        assert!(resolved.jump_target.is_none());
    }

    // --- Procedure resolution ---

    fn make_procedure(overrides: impl FnOnce(&mut Procedure)) -> Procedure {
        let mut p = Procedure {
            name: "test_proc".into(),
            session: "test".into(),
            commands: vec!["echo hello".into(), "echo world".into()],
            description: None,
            no_tty: false,
            fail_fast: true,
            tags: vec![],
        };
        overrides(&mut p);
        p
    }

    fn default_sessions() -> Vec<Session> {
        vec![
            make_session(|s| {
                s.name = "web".into();
                s.host = "10.0.0.1".into();
            }),
            make_session(|s| {
                s.name = "test".into();
                s.host = "10.0.0.2".into();
            }),
        ]
    }

    #[test]
    fn resolve_procedure_basic() {
        let sessions = default_sessions();
        let proc = make_procedure(|_| {});
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert_eq!(resolved.name, "test_proc");
        assert_eq!(resolved.session.name, "test");
        assert_eq!(resolved.session.host, "10.0.0.2");
        assert_eq!(resolved.commands, vec!["echo hello", "echo world"]);
        assert_eq!(resolved.shell_command, "echo hello && echo world");
        assert!(resolved.fail_fast);
        assert!(!resolved.no_tty);
    }

    #[test]
    fn resolve_procedure_fail_fast_false_uses_semicolon() {
        let sessions = default_sessions();
        let proc = make_procedure(|p| p.fail_fast = false);
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert_eq!(resolved.shell_command, "echo hello ; echo world");
        assert!(!resolved.fail_fast);
    }

    #[test]
    fn resolve_procedure_single_command() {
        let sessions = default_sessions();
        let proc = make_procedure(|p| p.commands = vec!["ls -la".into()]);
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert_eq!(resolved.shell_command, "ls -la");
    }

    #[test]
    fn resolve_procedure_unknown_session_returns_none() {
        let sessions = default_sessions();
        let proc = make_procedure(|p| p.session = "nonexistent".into());
        assert!(resolve_procedure(&proc, &sessions).is_none());
    }

    #[test]
    fn resolve_procedure_normalizes_tags() {
        let sessions = default_sessions();
        let proc = make_procedure(|p| {
            p.tags = vec![
                " deploy ".into(),
                "prod".into(),
                "deploy".into(),
                "  ".into(),
            ];
        });
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert_eq!(resolved.tags, vec!["deploy", "prod"]);
    }

    #[test]
    fn resolve_procedure_no_description_is_none() {
        let sessions = default_sessions();
        let proc = make_procedure(|p| p.description = None);
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert!(resolved.description.is_none());
    }

    #[test]
    fn resolve_procedure_preserves_description() {
        let sessions = default_sessions();
        let proc = make_procedure(|p| p.description = Some("Deploy app".into()));
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert_eq!(resolved.description.as_deref(), Some("Deploy app"));
    }

    #[test]
    fn resolve_procedure_preserves_no_tty() {
        let sessions = default_sessions();
        let proc = make_procedure(|p| p.no_tty = true);
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert!(resolved.no_tty);
    }

    #[test]
    fn resolve_procedure_with_jump_host() {
        let sessions = vec![
            make_session(|s| {
                s.name = "bastion".into();
                s.host = "10.0.0.1".into();
                s.username = Some("ops".into());
            }),
            make_session(|s| {
                s.name = "internal".into();
                s.host = "10.0.0.2".into();
                s.jump = Some("bastion".into());
            }),
        ];
        let proc = make_procedure(|p| p.session = "internal".into());
        let resolved = resolve_procedure(&proc, &sessions).unwrap();
        assert_eq!(resolved.session.name, "internal");
        assert!(resolved.session.jump_target.is_some());
    }
}
