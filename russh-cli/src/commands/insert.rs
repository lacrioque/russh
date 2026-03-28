use anyhow::{bail, Context as _};
use russh_core::paths;
use std::fs;
use std::io::{self, Write as _};

/// Parse a `user@host` string into (username, host).
/// If no `@`, treat the entire string as a host.
fn parse_target(target: &str) -> (Option<String>, String) {
    if let Some((user, host)) = target.split_once('@') {
        (Some(user.to_string()), host.to_string())
    } else {
        (None, target.to_string())
    }
}

/// Run the insert command: parse SSH-style arguments, write to config, optionally connect.
///
/// Usage: russh i <name> user@host [-p port] [-i /path/to/key] [-J jump_session]
pub fn run(
    name: &str,
    target: &str,
    port: Option<u16>,
    identity: Option<&str>,
    jump: Option<&str>,
    config_override: Option<&str>,
) -> anyhow::Result<()> {
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;

    // Parse user@host
    let (username, host) = parse_target(target);

    if host.is_empty() {
        bail!("host cannot be empty");
    }

    // Check for duplicate session name
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config: {}", config_path.display()))?;
        let key = format!("[sessions.{}]", name);
        if contents.contains(&key) {
            bail!("session \"{name}\" already exists in {}", config_path.display());
        }
    }

    // Build the TOML block
    let mut block = format!("\n[sessions.{}]\nhost = \"{}\"\n", name, host);
    if let Some(ref user) = username {
        block.push_str(&format!("username = \"{}\"\n", user));
    }
    if let Some(p) = port {
        block.push_str(&format!("port = {}\n", p));
    }
    if let Some(key) = identity {
        block.push_str(&format!("ssh_key = \"{}\"\n", key));
    }
    if let Some(j) = jump {
        block.push_str(&format!("jump = \"{}\"\n", j));
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
        .with_context(|| "failed to write session to config")?;

    // Summary
    let display_user = username.as_deref().unwrap_or("(default)");
    let display_port = port.map_or("22".to_string(), |p| p.to_string());
    let display_key = identity.unwrap_or("(system default)");
    let display_jump = jump.unwrap_or("(none)");

    println!("Session \"{}\" added to {}", name, config_path.display());
    println!("  Host:     {}", host);
    println!("  User:     {}", display_user);
    println!("  Port:     {}", display_port);
    println!("  SSH Key:  {}", display_key);
    println!("  Jump:     {}", display_jump);

    // Ask if user wants to connect
    print!("\nConnect now? [Y/n] ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    if answer.is_empty() || answer == "y" || answer == "yes" {
        // Delegate to connect command
        super::connect::run(name, config_override)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_user_at_host() {
        let (user, host) = parse_target("deploy@10.0.0.1");
        assert_eq!(user, Some("deploy".to_string()));
        assert_eq!(host, "10.0.0.1");
    }

    #[test]
    fn parse_host_only() {
        let (user, host) = parse_target("10.0.0.1");
        assert_eq!(user, None);
        assert_eq!(host, "10.0.0.1");
    }

    #[test]
    fn parse_user_at_hostname() {
        let (user, host) = parse_target("root@prod.example.com");
        assert_eq!(user, Some("root".to_string()));
        assert_eq!(host, "prod.example.com");
    }
}
