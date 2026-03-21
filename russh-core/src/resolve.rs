use crate::model::{KeySource, ResolvedSession, Session};
use crate::paths::expand_tilde;
use std::env;

const DEFAULT_PORT: u16 = 22;

/// Resolves a raw [`Session`] into a [`ResolvedSession`] with all defaults applied.
///
/// - Missing `username` → current OS user (`USER` on Unix, `USERNAME` on Windows).
/// - Missing `port` → 22.
/// - `ssh_key` with leading `~` → expanded via [`expand_tilde`].
/// - Tags are trimmed, deduplicated, and sorted.
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
        };
        overrides(&mut s);
        s
    }

    fn with_env<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
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
}
