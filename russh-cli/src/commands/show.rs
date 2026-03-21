use anyhow::Result;
use russh_core::config::load_config;
use russh_core::resolve::resolve_session;
use std::path::Path;

/// Display detailed information for a named session.
///
/// Prints both the raw config values (as written by the user) and the
/// resolved values (with defaults applied) so the user can see exactly
/// what SSH will use.
pub fn run(target: &str, config_path: &Path) -> Result<()> {
    let sessions = load_config(config_path)?;

    let session = sessions
        .iter()
        .find(|s| s.name == target)
        .ok_or_else(|| anyhow::anyhow!("unknown session: \"{target}\""))?;

    let resolved = resolve_session(session);

    println!("Session: {}", session.name);
    println!();

    println!("  host        {}", session.host);

    match &session.username {
        Some(u) => println!("  username    {} (configured)", u),
        None => println!("  username    {} (default: OS user)", resolved.username),
    }

    match session.port {
        Some(p) => println!("  port        {} (configured)", p),
        None => println!("  port        {} (default)", resolved.port),
    }

    match &session.ssh_key {
        Some(raw) => {
            let expanded = resolved.ssh_key.as_deref().unwrap_or(raw);
            if expanded != raw {
                println!("  ssh_key     {} → {} (expanded)", raw, expanded);
            } else {
                println!("  ssh_key     {}", raw);
            }
        }
        None => println!("  ssh_key     (system default)"),
    }

    if session.tags.is_empty() {
        println!("  tags        (none)");
    } else {
        println!("  tags        {}", session.tags.join(", "));
    }

    println!();
    println!("  target      {}", resolved.display_target);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn write_config(content: &str) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "{content}").unwrap();
        tmp
    }

    #[test]
    fn show_known_session() {
        let tmp = write_config(
            r#"
[sessions.dev]
host = "10.0.0.1"
username = "admin"
port = 2222
ssh_key = "/etc/ssh/key"
tags = ["dev"]
"#,
        );
        assert!(run("dev", tmp.path()).is_ok());
    }

    #[test]
    fn show_unknown_session_errors() {
        let tmp = write_config(
            r#"
[sessions.dev]
host = "10.0.0.1"
"#,
        );
        let err = run("nope", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("unknown session"), "{err}");
    }

    #[test]
    fn show_unknown_includes_name() {
        let tmp = write_config(
            r#"
[sessions.dev]
host = "10.0.0.1"
"#,
        );
        let err = run("missing-host", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("missing-host"), "{err}");
    }
}
