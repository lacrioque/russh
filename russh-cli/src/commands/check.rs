use std::path::PathBuf;
use std::process;

use russh_core::config::load_config;
use russh_core::model::Severity;
use russh_core::paths::config_path;
use russh_core::resolve::resolve_session;
use russh_core::validate::validate_sessions;

/// Run `check`: validate all sessions and report issues grouped by severity.
///
/// Exit codes (ADR-0006):
/// - 0: no issues
/// - 1: warnings only
/// - 2: at least one error
pub fn run(config_override: Option<&str>) -> ! {
    let path: PathBuf = match config_path(config_override) {
        Some(p) => p,
        None => {
            eprintln!("error: cannot determine config path");
            process::exit(2);
        }
    };

    let sessions = match load_config(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(2);
        }
    };

    let resolved: Vec<_> = sessions.iter().map(resolve_session).collect();
    let issues = validate_sessions(&resolved);

    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    let warnings: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .collect();

    if !errors.is_empty() {
        println!("Errors ({}):", errors.len());
        for issue in &errors {
            println!("  {issue}");
        }
    }

    if !warnings.is_empty() {
        println!("Warnings ({}):", warnings.len());
        for issue in &warnings {
            println!("  {issue}");
        }
    }

    if issues.is_empty() {
        println!(
            "OK: {} session(s) validated, no issues found",
            resolved.len()
        );
        process::exit(0);
    } else if errors.is_empty() {
        process::exit(1);
    } else {
        process::exit(2);
    }
}
