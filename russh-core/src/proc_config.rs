use std::collections::BTreeMap;
use std::path::Path;

use crate::model::Procedure;

/// Errors that can occur when loading procedure configuration.
#[derive(Debug, thiserror::Error)]
pub enum ProcConfigError {
    /// Procedures file does not exist at the given path.
    #[error("procedures file not found: {0}")]
    NotFound(String),

    /// Failed to read the procedures file from disk.
    #[error("failed to read procedures file: {0}")]
    ReadError(#[from] std::io::Error),

    /// TOML syntax or structure is invalid.
    #[error("failed to parse procedures: {0}")]
    ParseError(#[from] toml::de::Error),

    /// A procedure is missing a required field or has invalid data.
    #[error("procedure \"{procedure}\": {message}")]
    ValidationError {
        procedure: String,
        message: String,
    },
}

/// Top-level TOML structure: `[procedures.<name>]` tables.
#[derive(Debug, serde::Deserialize)]
struct ProcFile {
    #[serde(default)]
    procedures: BTreeMap<String, Procedure>,
}

/// Load procedures from a TOML config file at `path`.
///
/// Each procedure is defined under `[procedures.<name>]`. The procedure's `name`
/// field is populated from its table key. Returns an error if the file is
/// missing, unreadable, or contains invalid TOML.
pub fn load_procedures(path: &Path) -> Result<Vec<Procedure>, ProcConfigError> {
    if !path.exists() {
        return Err(ProcConfigError::NotFound(path.display().to_string()));
    }

    let content = std::fs::read_to_string(path)?;
    parse_procedures(&content)
}

/// Parse TOML content into a list of procedures.
///
/// Useful for testing without touching the filesystem.
pub fn parse_procedures(content: &str) -> Result<Vec<Procedure>, ProcConfigError> {
    let config: ProcFile = toml::from_str(content)?;

    let mut procedures = Vec::with_capacity(config.procedures.len());
    for (name, mut proc) in config.procedures {
        if proc.session.is_empty() {
            return Err(ProcConfigError::ValidationError {
                procedure: name,
                message: "missing required field \"session\"".into(),
            });
        }
        if proc.commands.is_empty() {
            return Err(ProcConfigError::ValidationError {
                procedure: name,
                message: "commands list must not be empty".into(),
            });
        }
        proc.name = name;
        procedures.push(proc);
    }

    Ok(procedures)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn parse_valid_procedure() {
        let toml = r#"
[procedures.deploy]
session = "prod"
commands = ["systemctl stop app", "rsync ...", "systemctl start app"]
description = "Deploy the application"
tags = ["deploy", "prod"]
"#;
        let procs = parse_procedures(toml).unwrap();
        assert_eq!(procs.len(), 1);

        let p = &procs[0];
        assert_eq!(p.name, "deploy");
        assert_eq!(p.session, "prod");
        assert_eq!(p.commands.len(), 3);
        assert_eq!(p.description.as_deref(), Some("Deploy the application"));
        assert!(p.fail_fast);
        assert!(!p.no_tty);
        assert_eq!(p.tags, vec!["deploy", "prod"]);
    }

    #[test]
    fn parse_multiple_procedures() {
        let toml = r#"
[procedures.backup]
session = "db"
commands = ["pg_dump -Fc mydb > /tmp/backup.dump"]

[procedures.restore]
session = "db"
commands = ["pg_restore -d mydb /tmp/backup.dump"]
"#;
        let procs = parse_procedures(toml).unwrap();
        assert_eq!(procs.len(), 2);
        let names: Vec<&str> = procs.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"backup"));
        assert!(names.contains(&"restore"));
    }

    #[test]
    fn parse_empty_procedures_table() {
        let toml = "[procedures]\n";
        let procs = parse_procedures(toml).unwrap();
        assert!(procs.is_empty());
    }

    #[test]
    fn parse_no_procedures_key() {
        let toml = "# empty config\n";
        let procs = parse_procedures(toml).unwrap();
        assert!(procs.is_empty());
    }

    #[test]
    fn parse_missing_session_error() {
        let toml = r#"
[procedures.bad]
commands = ["echo hi"]
"#;
        let err = parse_procedures(toml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing"), "expected missing field error: {msg}");
    }

    #[test]
    fn parse_empty_session_error() {
        let toml = r#"
[procedures.bad]
session = ""
commands = ["echo hi"]
"#;
        let err = parse_procedures(toml).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing required field \"session\""),
            "expected session error: {msg}"
        );
    }

    #[test]
    fn parse_empty_commands_error() {
        let toml = r#"
[procedures.bad]
session = "dev"
commands = []
"#;
        let err = parse_procedures(toml).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("commands list must not be empty"),
            "expected commands error: {msg}"
        );
    }

    #[test]
    fn parse_missing_commands_error() {
        let toml = r#"
[procedures.bad]
session = "dev"
"#;
        let err = parse_procedures(toml).unwrap_err();
        // TOML deserialization should fail for missing required field
        assert!(matches!(
            err,
            ProcConfigError::ParseError(_)
        ));
    }

    #[test]
    fn parse_malformed_toml() {
        let toml = "this is not [valid toml";
        let err = parse_procedures(toml).unwrap_err();
        assert!(matches!(err, ProcConfigError::ParseError(_)));
    }

    #[test]
    fn parse_defaults_fail_fast_true() {
        let toml = r#"
[procedures.test]
session = "dev"
commands = ["echo test"]
"#;
        let procs = parse_procedures(toml).unwrap();
        assert!(procs[0].fail_fast);
    }

    #[test]
    fn parse_fail_fast_override_false() {
        let toml = r#"
[procedures.test]
session = "dev"
commands = ["echo test"]
fail_fast = false
"#;
        let procs = parse_procedures(toml).unwrap();
        assert!(!procs[0].fail_fast);
    }

    #[test]
    fn parse_no_tty_override_true() {
        let toml = r#"
[procedures.test]
session = "dev"
commands = ["echo test"]
no_tty = true
"#;
        let procs = parse_procedures(toml).unwrap();
        assert!(procs[0].no_tty);
    }

    #[test]
    fn load_procedures_file_not_found() {
        let err = load_procedures(Path::new("/nonexistent/procedures.toml")).unwrap_err();
        assert!(matches!(err, ProcConfigError::NotFound(_)));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn load_procedures_from_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"
[procedures.test]
session = "dev"
commands = ["echo hello"]
description = "Test procedure"
"#
        )
        .unwrap();

        let procs = load_procedures(tmp.path()).unwrap();
        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0].name, "test");
        assert_eq!(procs[0].session, "dev");
        assert_eq!(procs[0].commands, vec!["echo hello"]);
    }

    #[test]
    fn parse_procedure_with_all_fields() {
        let toml = r#"
[procedures.full]
session = "staging"
commands = ["cmd1", "cmd2"]
description = "Full procedure"
no_tty = true
fail_fast = false
tags = ["ci", "staging"]
"#;
        let procs = parse_procedures(toml).unwrap();
        let p = &procs[0];
        assert_eq!(p.session, "staging");
        assert_eq!(p.commands, vec!["cmd1", "cmd2"]);
        assert_eq!(p.description.as_deref(), Some("Full procedure"));
        assert!(p.no_tty);
        assert!(!p.fail_fast);
        assert_eq!(p.tags, vec!["ci", "staging"]);
    }
}
