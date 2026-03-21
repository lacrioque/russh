use anyhow::Context as _;
use russh_core::{config, paths, resolve};

use crate::ui::inquire::InquirePicker;
use crate::ui::SessionPicker as _;

/// Run the menu command: load sessions, prompt interactively, then connect.
pub fn run(config_override: Option<&str>) -> anyhow::Result<()> {
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;

    let raw_sessions = config::load_config(&config_path)
        .with_context(|| format!("failed to load config: {}", config_path.display()))?;

    let sessions: Vec<_> = raw_sessions.iter().map(resolve::resolve_session).collect();

    let picker = InquirePicker;
    match picker.pick(&sessions)? {
        Some(selected) => super::connect::run(&selected.name, config_override),
        None => Ok(()),
    }
}
