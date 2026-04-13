use anyhow::{Context, Result};
use russh_core::config::load_config;
use russh_core::paths::{config_path, procedures_path};
use russh_core::proc_config::load_procedures;
use russh_core::resolve::resolve_session_with_jump;
use russh_core::ssh::build_command;

pub fn run(
    target: &str,
    proc_config_override: Option<&str>,
    session_config_override: Option<&str>,
) -> Result<()> {
    let proc_path = procedures_path(proc_config_override)
        .context("could not determine procedures config path")?;
    let procedures = load_procedures(&proc_path)
        .with_context(|| format!("failed to load procedures from {}", proc_path.display()))?;

    let proc = procedures
        .iter()
        .find(|p| p.name == target)
        .ok_or_else(|| anyhow::anyhow!("unknown procedure: \"{target}\""))?;

    println!("Procedure: {}", proc.name);
    println!();

    println!("  session       {}", proc.session);

    match &proc.description {
        Some(desc) => println!("  description   {desc}"),
        None => println!("  description   (none)"),
    }

    if proc.tags.is_empty() {
        println!("  tags          (none)");
    } else {
        println!("  tags          {}", proc.tags.join(", "));
    }

    if proc.commands.is_empty() {
        println!("  commands      (none)");
    } else {
        println!("  commands:");
        for (i, cmd) in proc.commands.iter().enumerate() {
            println!("    {}. {}", i + 1, cmd);
        }
    }

    // Resolve session info if available
    let session_path = config_path(session_config_override);
    if let Some(ref sp) = session_path {
        if let Ok(sessions) = load_config(sp) {
            if let Some(session) = sessions.iter().find(|s| s.name == proc.session) {
                let resolved = resolve_session_with_jump(session, &sessions);
                println!();
                println!("  Resolved session:");
                println!("    target      {}", resolved.display_target);
                if let Some(ref jt) = resolved.jump_target {
                    println!("    jump via    {jt}");
                }

                // SSH command preview
                let spec = build_command(&resolved);
                if !proc.commands.is_empty() {
                    let remote_cmd = proc.commands.join(" && ");
                    println!();
                    println!("  SSH command preview:");
                    println!("    {} {}", spec.display, shell_quote(&remote_cmd));
                }
            } else {
                println!();
                println!(
                    "  warning: session \"{}\" not found in config",
                    proc.session
                );
            }
        }
    }

    Ok(())
}

fn shell_quote(s: &str) -> String {
    if s.contains('\'') {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        format!("'{s}'")
    }
}
