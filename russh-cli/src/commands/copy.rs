use anyhow::{bail, Context as _};
use russh_core::{model::Severity, paths, resolve, sync, validate};

use super::init_config::load_or_create_config;

const DEFAULT_PATH: &str = "~";

pub fn run(
    source_name: &str,
    source_path: &str,
    dest_name: &str,
    dest_path: Option<&str>,
    dry_run: bool,
    config_override: Option<&str>,
) -> anyhow::Result<()> {
    let config_path =
        paths::config_path(config_override).context("could not determine config path")?;

    let sessions = load_or_create_config(&config_path)?;

    let source = sessions
        .iter()
        .find(|s| s.name == source_name)
        .with_context(|| format!("source session not found: {source_name}"))?;
    let dest = sessions
        .iter()
        .find(|s| s.name == dest_name)
        .with_context(|| format!("destination session not found: {dest_name}"))?;

    let source_resolved = resolve::resolve_session_with_jump(source, &sessions);
    let dest_resolved = resolve::resolve_session_with_jump(dest, &sessions);

    for (label, resolved) in [
        ("source", &source_resolved),
        ("destination", &dest_resolved),
    ] {
        let errors: Vec<_> = validate::validate_session(resolved)
            .into_iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        if !errors.is_empty() {
            for e in &errors {
                eprintln!("{e}");
            }
            bail!(
                "{label} session \"{}\" has launch-blocking errors",
                resolved.name
            );
        }
    }

    let dest_path = dest_path.unwrap_or(DEFAULT_PATH);
    let strategy = sync::host_copy_strategy(&source_resolved, &dest_resolved);

    match strategy {
        sync::CopyStrategy::Direct => {
            let args = sync::build_host_copy_args(
                &source_resolved,
                source_path,
                &dest_resolved,
                dest_path,
            );
            let display = format!(
                "{}@{}:{} -> {}@{}:{}",
                source_resolved.username,
                source_resolved.host,
                source_path,
                dest_resolved.username,
                dest_resolved.host,
                dest_path
            );

            if dry_run {
                println!("[dry-run] scp {} (direct)", args.join(" "));
                return Ok(());
            }

            println!("copying {display} (direct)");
            sync::run_scp(&args, &display)?;
            println!("done");
        }
        sync::CopyStrategy::ViaLocal => {
            let tmp = tempfile::NamedTempFile::new()
                .context("failed to create local temp file for two-step copy")?;
            let tmp_path = tmp.path();

            let download_args = sync::build_download_args(&source_resolved, source_path, tmp_path);
            let upload_args = sync::build_upload_args(&dest_resolved, tmp_path, dest_path);

            let download_display = format!(
                "{}@{}:{} -> (local)",
                source_resolved.username, source_resolved.host, source_path
            );
            let upload_display = format!(
                "(local) -> {}@{}:{}",
                dest_resolved.username, dest_resolved.host, dest_path
            );

            if dry_run {
                println!("[dry-run] source and destination use different jump hosts");
                println!("[dry-run]   step 1: scp {}", download_args.join(" "));
                println!("[dry-run]   step 2: scp {}", upload_args.join(" "));
                return Ok(());
            }

            println!("copying (via local temp, source and destination have different jump hosts)");
            println!("  step 1: {download_display}");
            sync::run_scp(&download_args, &download_display)?;
            println!("  step 2: {upload_display}");
            sync::run_scp(&upload_args, &upload_display)?;
            println!("done");
        }
    }

    Ok(())
}
