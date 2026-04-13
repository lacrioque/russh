use anyhow::{bail, Context as _};
use russh_core::paths;
use std::fs;
use std::io::Write as _;

const RESERVED_NAME: &str = "NONE";

/// Run the proc insert command: build a TOML block and append to procedures config.
pub fn run(
    name: &str,
    session: &str,
    commands: &[String],
    description: Option<&str>,
    no_tty: bool,
    no_fail_fast: bool,
    config_override: Option<&str>,
) -> anyhow::Result<()> {
    if name.eq_ignore_ascii_case(RESERVED_NAME) {
        bail!("\"{}\" is a reserved procedure name", RESERVED_NAME);
    }

    if session.is_empty() {
        bail!("session name cannot be empty");
    }

    if commands.is_empty() {
        bail!("at least one command is required (-c 'command')");
    }

    let config_path = paths::procedures_path(config_override)
        .context("could not determine procedures config path")?;

    // Check for duplicate procedure name
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config: {}", config_path.display()))?;
        let key = format!("[procedures.{}]", name);
        if contents.contains(&key) {
            bail!(
                "procedure \"{}\" already exists in {}",
                name,
                config_path.display()
            );
        }
    }

    // Build the TOML block
    let mut block = format!("\n[procedures.{}]\nsession = \"{}\"\n", name, session);
    block.push_str("commands = [");
    for (i, cmd) in commands.iter().enumerate() {
        if i > 0 {
            block.push_str(", ");
        }
        let escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"");
        block.push_str(&format!("\"{}\"", escaped));
    }
    block.push_str("]\n");

    if let Some(desc) = description {
        let escaped = desc.replace('\\', "\\\\").replace('"', "\\\"");
        block.push_str(&format!("description = \"{}\"\n", escaped));
    }
    if no_tty {
        block.push_str("no_tty = true\n");
    }
    if no_fail_fast {
        block.push_str("fail_fast = false\n");
    }

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory: {}", parent.display()))?;
    }

    // Append to config file
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .with_context(|| format!("failed to open config: {}", config_path.display()))?;

    file.write_all(block.as_bytes())
        .with_context(|| "failed to write procedure to config")?;

    println!("Procedure \"{}\" added to {}", name, config_path.display());
    println!("  Session:     {}", session);
    println!("  Commands:    {}", commands.len());
    for cmd in commands {
        println!("    - {}", cmd);
    }
    if let Some(desc) = description {
        println!("  Description: {}", desc);
    }
    if no_tty {
        println!("  TTY:         disabled");
    }
    if no_fail_fast {
        println!("  Fail-fast:   disabled");
    }

    Ok(())
}
