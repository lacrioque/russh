use anyhow::Result;
use inquire::Select;
use russh_core::model::ResolvedSession;

use crate::ui::SessionPicker;

/// Session picker backed by [inquire]'s fuzzy-searchable `Select` prompt.
pub struct InquirePicker;

impl SessionPicker for InquirePicker {
    fn pick(&self, sessions: &[ResolvedSession]) -> Result<Option<ResolvedSession>> {
        if sessions.is_empty() {
            return Ok(None);
        }

        let labels: Vec<String> = sessions
            .iter()
            .map(|s| format!("{} ({})", s.name, s.display_target))
            .collect();

        let selected = Select::new("Select session:", labels.clone()).prompt_skippable()?;

        match selected {
            None => Ok(None),
            Some(label) => {
                let idx = labels.iter().position(|l| *l == label).unwrap();
                Ok(Some(sessions[idx].clone()))
            }
        }
    }
}
