use anyhow::{bail, Context as _};
use std::path::PathBuf;

use russh_core::model::Severity;
use russh_core::proc_run::{
    build_procedure_command, build_script_command, resolve_procedure, spawn_ssh,
    spawn_ssh_with_log, spawn_ssh_with_script, validate_procedure,
};
use russh_core::{paths, proc_config, resolve};

use crate::commands::init_config::load_or_create_config;

/// Run a named procedure on a remote host via SSH.
pub fn run(
    name: &str,
    config_override: Option<&str>,
    from_config: Option<&str>,
    from_script: Option<&str>,
    script_session: Option<&str>,
    log_path: Option<&str>,
    no_tty: bool,
) -> anyhow::Result<()> {
    // Load sessions config
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;
    let sessions = load_or_create_config(&config_path)?;

    // Handle --from-script mode: pipe a local script to a session
    if let Some(script_path) = from_script {
        let session_name = script_session
            .context("--from-script requires --session <name>")?;

        let session = sessions
            .iter()
            .find(|s| s.name == session_name)
            .with_context(|| format!("session not found: {session_name}"))?;

        let resolved = resolve::resolve_session_with_jump(session, &sessions);

        // Validate session
        let issues = russh_core::validate::validate_session(&resolved);
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

        let script_file = PathBuf::from(paths::expand_tilde(script_path));
        if !script_file.exists() {
            bail!("script file not found: {}", script_file.display());
        }

        let spec = build_script_command(&resolved, no_tty);
        eprintln!("▶ {}", spec.display);
        eprintln!("  piping: {}", script_file.display());

        let exit_code = spawn_ssh_with_script(&spec, &script_file)
            .with_context(|| format!("failed to run script on {session_name}"))?;

        if exit_code != 0 {
            bail!("script exited with status {exit_code}");
        }
        return Ok(());
    }

    // Load procedures config
    let proc_path = paths::procedures_path(from_config)
        .context("could not determine procedures config path")?;
    let procedures = proc_config::load_procedures(&proc_path)
        .with_context(|| format!("failed to load procedures from {}", proc_path.display()))?;

    // Find the named procedure
    let proc = procedures
        .iter()
        .find(|p| p.name == name)
        .with_context(|| format!("procedure not found: {name}"))?;

    // Validate
    let issues = validate_procedure(proc, &sessions);
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        bail!("procedure \"{name}\" has launch-blocking errors");
    }

    // Resolve procedure (session lookup + defaults)
    let mut resolved = resolve_procedure(proc, &sessions)
        .with_context(|| format!("could not resolve session for procedure \"{name}\""))?;

    // Apply --no-tty override
    if no_tty {
        resolved.no_tty = true;
    }

    // Validate resolved session
    let session_issues = russh_core::validate::validate_session(&resolved.session);
    let session_errors: Vec<_> = session_issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    if !session_errors.is_empty() {
        for e in &session_errors {
            eprintln!("{e}");
        }
        bail!(
            "session \"{}\" has launch-blocking errors",
            resolved.session.name
        );
    }

    // Build SSH command
    let spec = build_procedure_command(&resolved);
    eprintln!("▶ {}", spec.display);

    // Spawn
    let exit_code = if let Some(log) = log_path {
        let log_file = PathBuf::from(paths::expand_tilde(log));
        eprintln!("  logging to: {}", log_file.display());
        spawn_ssh_with_log(&spec, &log_file)
            .with_context(|| format!("failed to run procedure \"{name}\""))?
    } else {
        spawn_ssh(&spec).with_context(|| format!("failed to run procedure \"{name}\""))?
    };

    // Report exit status
    if exit_code == 0 {
        eprintln!("✓ procedure \"{name}\" completed successfully");
    } else {
        bail!("procedure \"{name}\" exited with status {exit_code}");
    }

    Ok(())
}
