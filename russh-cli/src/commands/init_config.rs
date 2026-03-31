use std::fs;
use std::io::{self, Write as _};
use std::path::Path;

use anyhow::{Context as _, Result};
use russh_core::config::{load_config, ConfigError};
use russh_core::model::Session;

/// Load sessions from config, prompting to create the file if it doesn't exist.
///
/// When the config file is missing, the user is asked whether to create an
/// empty one.  If they decline, an error is returned so the caller can exit
/// cleanly.
pub fn load_or_create_config(path: &Path) -> Result<Vec<Session>> {
    match load_config(path) {
        Ok(sessions) => Ok(sessions),
        Err(ConfigError::NotFound(_)) => prompt_create_config(path),
        Err(e) => Err(e).with_context(|| format!("failed to load config: {}", path.display())),
    }
}

fn prompt_create_config(path: &Path) -> Result<Vec<Session>> {
    eprintln!("Config file not found: {}", path.display());
    eprint!("Create it now? [Y/n] ");
    io::stderr().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    if !answer.is_empty() && answer != "y" && answer != "yes" {
        anyhow::bail!("no config file — run `russh insert` to create one with a session");
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory: {}", parent.display()))?;
    }

    fs::write(
        path,
        "# russh configuration — add sessions with `russh insert`\n",
    )
    .with_context(|| format!("failed to write config: {}", path.display()))?;

    eprintln!("Created {}", path.display());
    Ok(Vec::new())
}
