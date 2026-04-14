use anyhow::{bail, Context as _};
use russh_core::{model::Severity, paths, resolve, ssh, validate};
use serde::Serialize;
use std::io::Write;
use std::process;

use super::init_config::load_or_create_config;

#[derive(Serialize)]
struct ExecResult {
    session: String,
    command: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

pub fn run(
    session_name: &str,
    command: &str,
    no_tty: bool,
    json: bool,
    to_std: bool,
    config_override: Option<&str>,
) -> anyhow::Result<()> {
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;

    let sessions = load_or_create_config(&config_path)?;

    let session = sessions
        .iter()
        .find(|s| s.name == session_name)
        .with_context(|| format!("session not found: {session_name}"))?;

    let resolved = resolve::resolve_session_with_jump(session, &sessions);

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

    let spec = ssh::build_procedure_command(&resolved, command, no_tty);

    if json {
        let captured = ssh::spawn_ssh_capture(&spec)
            .with_context(|| format!("failed to exec: {}", spec.display))?;

        let result = ExecResult {
            session: session_name.to_string(),
            command: command.to_string(),
            exit_code: captured.exit_code,
            stdout: captured.stdout,
            stderr: captured.stderr,
        };

        println!("{}", serde_json::to_string_pretty(&result)?);

        if captured.exit_code != Some(0) {
            process::exit(captured.exit_code.unwrap_or(1));
        }
    } else if to_std {
        let captured = ssh::spawn_ssh_capture(&spec)
            .with_context(|| format!("failed to exec: {}", spec.display))?;

        std::io::stdout().write_all(captured.stdout.as_bytes())?;
        std::io::stderr().write_all(captured.stderr.as_bytes())?;

        if captured.exit_code != Some(0) {
            process::exit(captured.exit_code.unwrap_or(1));
        }
    } else {
        let status = ssh::spawn_ssh(&spec, None, None)
            .with_context(|| format!("failed to exec: {}", spec.display))?;

        if !status.success() {
            let code = status.code().unwrap_or(1);
            process::exit(code);
        }
    }

    Ok(())
}
