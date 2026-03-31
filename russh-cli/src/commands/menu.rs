use anyhow::Context as _;
use russh_core::{paths, resolve};

use super::init_config::load_or_create_config;
use crate::ui::inquire::InquirePicker;
use crate::ui::SessionPicker as _;

/// Run the menu command: load sessions, prompt interactively, then connect.
pub fn run(config_override: Option<&str>) -> anyhow::Result<()> {
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;

    let raw_sessions = load_or_create_config(&config_path)?;

    let sessions: Vec<_> = raw_sessions.iter().map(resolve::resolve_session).collect();

    let picker = InquirePicker;
    match picker.pick(&sessions)? {
        Some(selected) => super::connect::run(&selected.name, config_override),
        None => Ok(()),
    }
}
