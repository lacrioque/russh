use std::collections::BTreeMap;
use std::path::Path;

use crate::model::Session;

/// Errors that can occur when loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Config file does not exist at the given path.
    #[error("config file not found: {0}")]
    NotFound(String),

    /// Failed to read the config file from disk.
    #[error("failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    /// TOML syntax or structure is invalid.
    #[error("failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    /// A session is missing a required field.
    #[error("session \"{session}\": missing required field \"{field}\"")]
    MissingField {
        session: String,
        field: &'static str,
    },
}

/// Top-level TOML structure: `[sessions.<name>]` tables.
#[derive(Debug, serde::Deserialize)]
struct ConfigFile {
    #[serde(default)]
    sessions: BTreeMap<String, Session>,
}

/// Load sessions from a TOML config file at `path`.
///
/// Each session is defined under `[sessions.<name>]`. The session's `name`
/// field is populated from its table key. Returns an error if the file is
/// missing, unreadable, or contains invalid TOML.
pub fn load_config(path: &Path) -> Result<Vec<Session>, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound(path.display().to_string()));
    }

    let content = std::fs::read_to_string(path)?;
    parse_config(&content)
}

/// Parse TOML content into a list of sessions.
///
/// Useful for testing without touching the filesystem.
pub fn parse_config(content: &str) -> Result<Vec<Session>, ConfigError> {
    let config: ConfigFile = toml::from_str(content)?;

    let mut sessions = Vec::with_capacity(config.sessions.len());
    for (name, mut session) in config.sessions {
        if session.host.is_empty() {
            return Err(ConfigError::MissingField {
                session: name,
                field: "host",
            });
        }
        session.name = name;
        sessions.push(session);
    }

    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn parse_valid_config() {
        let toml = r#"
[sessions.dev]
host = "10.0.0.1"
username = "admin"
port = 2222
ssh_key = "~/.ssh/dev_key"
tags = ["dev", "linux"]

[sessions.prod]
host = "prod.example.com"
"#;
        let sessions = parse_config(toml).unwrap();
        assert_eq!(sessions.len(), 2);

        let dev = sessions.iter().find(|s| s.name == "dev").unwrap();
        assert_eq!(dev.host, "10.0.0.1");
        assert_eq!(dev.username.as_deref(), Some("admin"));
        assert_eq!(dev.port, Some(2222));
        assert_eq!(dev.ssh_key.as_deref(), Some("~/.ssh/dev_key"));
        assert_eq!(dev.tags, vec!["dev", "linux"]);

        let prod = sessions.iter().find(|s| s.name == "prod").unwrap();
        assert_eq!(prod.host, "prod.example.com");
        assert_eq!(prod.username, None);
        assert_eq!(prod.port, None);
        assert_eq!(prod.tags, Vec::<String>::new());
    }

    #[test]
    fn parse_empty_sessions_table() {
        let toml = "[sessions]\n";
        let sessions = parse_config(toml).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn parse_no_sessions_key() {
        let toml = "# empty config\n";
        let sessions = parse_config(toml).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn parse_missing_host_error() {
        let toml = r#"
[sessions.bad]
username = "root"
"#;
        let err = parse_config(toml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing"), "expected missing field error: {msg}");
    }

    #[test]
    fn parse_empty_host_error() {
        let toml = r#"
[sessions.bad]
host = ""
"#;
        let err = parse_config(toml).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing required field"),
            "expected missing field error: {msg}"
        );
    }

    #[test]
    fn parse_malformed_toml() {
        let toml = "this is not [valid toml";
        let err = parse_config(toml).unwrap_err();
        assert!(matches!(err, ConfigError::ParseError(_)));
    }

    #[test]
    fn load_config_file_not_found() {
        let err = load_config(Path::new("/nonexistent/config.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::NotFound(_)));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn load_config_from_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"
[sessions.test]
host = "192.168.1.1"
username = "deploy"
"#
        )
        .unwrap();

        let sessions = load_config(tmp.path()).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "test");
        assert_eq!(sessions[0].host, "192.168.1.1");
        assert_eq!(sessions[0].username.as_deref(), Some("deploy"));
    }

    #[test]
    fn session_names_from_table_keys() {
        let toml = r#"
[sessions.alpha]
host = "a.example.com"

[sessions.bravo]
host = "b.example.com"

[sessions.charlie]
host = "c.example.com"
"#;
        let sessions = parse_config(toml).unwrap();
        let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"bravo"));
        assert!(names.contains(&"charlie"));
    }
}
