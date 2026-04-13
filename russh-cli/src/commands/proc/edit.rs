use anyhow::{bail, Context as _};
use russh_core::paths;
use std::process::Command;

/// Run the proc edit command: open the procedures config file in $EDITOR.
pub fn run(config_override: Option<&str>) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .map_err(|_| anyhow::anyhow!("no editor set — set $EDITOR or $VISUAL"))?;

    let config_path = paths::procedures_path(config_override)
        .context("could not determine procedures config path")?;

    if !config_path.exists() {
        bail!(
            "procedures config file not found: {}\nRun 'russh proc insert' to create one.",
            config_path.display()
        );
    }

    let status = Command::new(&editor)
        .arg(&config_path)
        .status()
        .with_context(|| format!("failed to launch editor: {}", editor))?;

    if !status.success() {
        bail!("editor exited with status: {}", status);
    }

    Ok(())
}
