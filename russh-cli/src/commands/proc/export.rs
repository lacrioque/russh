use anyhow::{bail, Context as _};
use russh_core::{paths, proc_config, resolve, ssh};

use crate::commands::init_config::load_or_create_config;

/// Run the proc export command.
///
/// Without `--script`: print raw TOML (full procedures section or single block).
/// With `--script`: emit a standalone shell script with the full SSH command.
pub fn run(
    name: Option<&str>,
    script: bool,
    proc_config_override: Option<&str>,
    session_config_override: Option<&str>,
) -> anyhow::Result<()> {
    let proc_path = paths::procedures_path(proc_config_override)
        .context("could not determine procedures config path")?;

    if !proc_path.exists() {
        bail!("procedures config file not found: {}", proc_path.display());
    }

    let procedures = proc_config::load_procedures(&proc_path)
        .with_context(|| format!("failed to load procedures from {}", proc_path.display()))?;

    if procedures.is_empty() {
        bail!("no procedures defined in {}", proc_path.display());
    }

    if script {
        export_script(name, &procedures, session_config_override)?;
    } else {
        export_toml(name, &procedures)?;
    }

    Ok(())
}

fn export_toml(
    name: Option<&str>,
    procedures: &[russh_core::model::Procedure],
) -> anyhow::Result<()> {
    match name {
        Some(target) => {
            let proc = procedures
                .iter()
                .find(|p| p.name == target)
                .with_context(|| format!("procedure not found: {target}"))?;
            print_procedure_toml(proc);
        }
        None => {
            for (i, proc) in procedures.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_procedure_toml(proc);
            }
        }
    }
    Ok(())
}

fn print_procedure_toml(proc_: &russh_core::model::Procedure) {
    println!("[procedures.{}]", proc_.name);
    println!("session = \"{}\"", proc_.session);
    print!("commands = [");
    for (i, cmd) in proc_.commands.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        let escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"");
        print!("\"{}\"", escaped);
    }
    println!("]");
    if let Some(ref desc) = proc_.description {
        let escaped = desc.replace('\\', "\\\\").replace('"', "\\\"");
        println!("description = \"{}\"", escaped);
    }
    if proc_.no_tty {
        println!("no_tty = true");
    }
    if !proc_.fail_fast {
        println!("fail_fast = false");
    }
}

fn export_script(
    name: Option<&str>,
    procedures: &[russh_core::model::Procedure],
    session_config_override: Option<&str>,
) -> anyhow::Result<()> {
    let target = name.context("--script requires a procedure name")?;

    let proc_ = procedures
        .iter()
        .find(|p| p.name == target)
        .with_context(|| format!("procedure not found: {target}"))?;

    let config_path = paths::config_path(session_config_override)
        .context("could not determine sessions config path")?;
    let sessions = load_or_create_config(&config_path)?;

    let session = sessions
        .iter()
        .find(|s| s.name == proc_.session)
        .with_context(|| {
            format!(
                "procedure \"{}\" references session \"{}\" which was not found",
                proc_.name, proc_.session
            )
        })?;

    let resolved = resolve::resolve_session_with_jump(session, &sessions);
    let spec = ssh::build_command(&resolved);

    let separator = if proc_.fail_fast { " && " } else { "; " };
    let remote_cmd = proc_
        .commands
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(separator);

    println!("#!/bin/sh");
    if let Some(ref desc) = proc_.description {
        println!("# {}", desc);
    }
    println!("# Procedure: {}", proc_.name);
    println!("# Session: {} ({})", proc_.session, resolved.display_target);
    println!();

    let mut parts = vec![spec.executable.clone()];
    parts.extend(spec.args.iter().cloned());
    if proc_.no_tty {
        let dest_idx = parts.len() - 1;
        parts.insert(dest_idx, "-T".into());
    }

    let quoted = shell_quote(&remote_cmd);
    parts.push(quoted);

    println!("{}", parts.join(" "));

    Ok(())
}

fn shell_quote(s: &str) -> String {
    let escaped = s.replace('\'', "'\\''");
    format!("'{}'", escaped)
}
