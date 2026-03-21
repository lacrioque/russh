use anyhow::Result;
use russh_core::model::ResolvedSession;

pub mod inquire;

/// Trait for interactive session selection.
///
/// Decouples command logic from the specific UI backend.
/// The menu command accepts any `SessionPicker` implementation,
/// allowing a future ratatui backend to slot in without touching command code.
pub trait SessionPicker {
    fn pick(&self, sessions: &[ResolvedSession]) -> Result<Option<ResolvedSession>>;
}
