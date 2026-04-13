use std::collections::HashSet;
use std::process;

use russh_core::config::load_config;
use russh_core::model::Severity;
use russh_core::paths::{config_path, procedures_path};
use russh_core::proc_config::load_procedures;
use russh_core::validate::validate_procedures;

/// Run `proc check`: validate all procedures and report issues.
///
/// Exit codes:
/// - 0: no issues
/// - 1: warnings only
/// - 2: at least one error
pub fn run(proc_config_override: Option<&str>, session_config_override: Option<&str>) -> ! {
    let proc_path = match procedures_path(proc_config_override) {
        Some(p) => p,
        None => {
            eprintln!("error: cannot determine procedures config path");
            process::exit(2);
        }
    };

    let procedures = match load_procedures(&proc_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(2);
        }
    };

    // Load sessions to validate references
    let session_names_owned: Vec<String> = match config_path(session_config_override) {
        Some(sp) => match load_config(&sp) {
            Ok(sessions) => sessions.iter().map(|s| s.name.clone()).collect(),
            Err(e) => {
                eprintln!("warning: could not load sessions config: {e}");
                Vec::new()
            }
        },
        None => {
            eprintln!("warning: could not determine sessions config path");
            Vec::new()
        }
    };

    let session_names: HashSet<&str> = session_names_owned.iter().map(|s| s.as_str()).collect();
    let issues = validate_procedures(&procedures, &session_names);

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
            "OK: {} procedure(s) validated, no issues found",
            procedures.len()
        );
        process::exit(0);
    } else if errors.is_empty() {
        process::exit(1);
    } else {
        process::exit(2);
    }
}
