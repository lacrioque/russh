use anyhow::{Context as _, Result};
use russh_core::{paths, resolve, sync};

use super::init_config::load_or_create_config;

/// Run the deploy command: push local config to remote host(s) via SCP.
///
/// Modes:
/// - Single session: `russh deploy <session>`
/// - All sessions: `russh deploy --all`
/// - By tag: `russh deploy --tag <tag>`
pub fn run(
    session: Option<&str>,
    all: bool,
    tag: Option<&str>,
    dry_run: bool,
    config_override: Option<&str>,
) -> Result<()> {
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;

    let sessions = load_or_create_config(&config_path)?;

    if sessions.is_empty() {
        anyhow::bail!("no sessions configured — run `russh insert` to add one");
    }

    // Determine which sessions to deploy to
    let targets: Vec<_> = if all {
        sessions.iter().collect()
    } else if let Some(tag_filter) = tag {
        let filtered: Vec<_> = sessions
            .iter()
            .filter(|s| s.tags.iter().any(|t| t == tag_filter))
            .collect();
        if filtered.is_empty() {
            anyhow::bail!("no sessions found with tag: {tag_filter}");
        }
        filtered
    } else if let Some(name) = session {
        let s = sessions
            .iter()
            .find(|s| s.name == name)
            .with_context(|| format!("session not found: {name}"))?;
        vec![s]
    } else {
        anyhow::bail!("specify a session name, --all, or --tag <tag>");
    };

    let mut had_errors = false;

    for target in &targets {
        let resolved = resolve::resolve_session_with_jump(target, &sessions);

        match sync::deploy_to_session(&resolved, &config_path, dry_run) {
            Ok(result) => {
                if result.success {
                    eprintln!("  {} {}", result.session_name, result.message);
                } else {
                    eprintln!("  {} FAILED: {}", result.session_name, result.message);
                    had_errors = true;
                }
            }
            Err(e) => {
                eprintln!("  {} ERROR: {e}", target.name);
                had_errors = true;
            }
        }
    }

    if had_errors {
        anyhow::bail!("some deploys failed — see errors above");
    }

    if !dry_run {
        eprintln!(
            "deployed config to {} session{}",
            targets.len(),
            if targets.len() == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
