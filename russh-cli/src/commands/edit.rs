use anyhow::{bail, Context as _};
use russh_core::paths;
use std::fs;
use std::process::Command as ProcessCommand;

/// The sentinel value that means "remove this field".
const NONE_KEYWORD: &str = "NONE";

/// Run the edit command.
///
/// If `name` is `None`, open the config file in `$EDITOR` / `$VISUAL`.
/// Otherwise, apply the supplied field overrides to the named session.
pub fn run(
    name: Option<&str>,
    host: Option<&str>,
    user: Option<&str>,
    port: Option<&str>,
    identity: Option<&str>,
    jump: Option<&str>,
    config_override: Option<&str>,
) -> anyhow::Result<()> {
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;

    // No name → open in $EDITOR
    let name = match name {
        Some(n) => n,
        None => {
            let editor = std::env::var("VISUAL")
                .or_else(|_| std::env::var("EDITOR"))
                .map_err(|_| anyhow::anyhow!("no $EDITOR or $VISUAL set"))?;

            if !config_path.exists() {
                bail!("config file not found: {}", config_path.display());
            }

            let status = ProcessCommand::new(&editor)
                .arg(&config_path)
                .status()
                .with_context(|| format!("failed to launch editor: {editor}"))?;

            if !status.success() {
                bail!("editor exited with status {status}");
            }
            return Ok(());
        }
    };

    if name == NONE_KEYWORD {
        bail!("\"NONE\" is a reserved keyword and cannot be used as a session name");
    }

    if !config_path.exists() {
        bail!("config file not found: {}", config_path.display());
    }

    // Check that at least one flag was provided
    if host.is_none() && user.is_none() && port.is_none() && identity.is_none() && jump.is_none() {
        bail!("no fields to edit; pass at least one of --host, --user, -p, -i, -J");
    }

    let contents = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config: {}", config_path.display()))?;

    let mut doc: toml_edit::DocumentMut = contents
        .parse()
        .with_context(|| format!("failed to parse config: {}", config_path.display()))?;

    let sessions = doc
        .get_mut("sessions")
        .and_then(|s| s.as_table_like_mut())
        .ok_or_else(|| anyhow::anyhow!("no [sessions] table in config"))?;

    let session = sessions
        .get_mut(name)
        .and_then(|s| s.as_table_like_mut())
        .ok_or_else(|| anyhow::anyhow!("session \"{name}\" not found"))?;

    // Apply edits
    apply_field(session, "host", host, false)?;
    apply_field(session, "username", user, true)?;
    apply_field(session, "ssh_key", identity, true)?;
    apply_field(session, "jump", jump, true)?;

    // Port needs special handling (integer field)
    if let Some(val) = port {
        if val == NONE_KEYWORD {
            session.remove("port");
        } else {
            let p: u16 = val
                .parse()
                .with_context(|| format!("invalid port: {val}"))?;
            session.insert(
                "port",
                toml_edit::Item::Value(toml_edit::Value::Integer(toml_edit::Formatted::new(
                    i64::from(p),
                ))),
            );
        }
    }

    fs::write(&config_path, doc.to_string())
        .with_context(|| format!("failed to write config: {}", config_path.display()))?;

    println!("Session \"{name}\" updated in {}", config_path.display());
    Ok(())
}

/// Apply a string field edit. If `removable` is true, the NONE keyword removes the field.
/// If `removable` is false (e.g. host), NONE is rejected.
fn apply_field(
    session: &mut dyn toml_edit::TableLike,
    key: &str,
    value: Option<&str>,
    removable: bool,
) -> anyhow::Result<()> {
    let val = match value {
        Some(v) => v,
        None => return Ok(()),
    };

    if val == NONE_KEYWORD {
        if removable {
            session.remove(key);
        } else {
            bail!("cannot remove required field \"{key}\"");
        }
    } else {
        session.insert(
            key,
            toml_edit::Item::Value(toml_edit::Value::String(toml_edit::Formatted::new(
                val.to_string(),
            ))),
        );
    }
    Ok(())
}
