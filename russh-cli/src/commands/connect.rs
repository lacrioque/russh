use anyhow::{bail, Context as _};
use russh_core::{
    config,
    model::Severity,
    paths,
    resolve,
    ssh,
    validate,
};

/// Run the connect command: locate session by name, validate, and exec SSH.
///
/// Loads the config from the default path (or `config_override` if given),
/// finds the session by name, resolves defaults, checks for launch-blocking
/// errors, then execs `ssh`. On success this function never returns —
/// the process is replaced by the SSH process.
pub fn run(session_name: &str, config_override: Option<&str>) -> anyhow::Result<()> {
    let config_path = paths::config_path(config_override)
        .context("could not determine config path")?;

    let sessions = config::load_config(&config_path)
        .with_context(|| format!("failed to load config: {}", config_path.display()))?;

    let session = sessions
        .iter()
        .find(|s| s.name == session_name)
        .with_context(|| format!("session not found: {session_name}"))?;

    let resolved = resolve::resolve_session(session);

    let issues = validate::validate_session(&resolved);
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        bail!("session \"{session_name}\" has launch-blocking errors");
    }

    let spec = ssh::build_command(&resolved);
    ssh::exec_ssh(&spec)
        .with_context(|| format!("failed to exec: {}", spec.display))?;

    Ok(())
}
