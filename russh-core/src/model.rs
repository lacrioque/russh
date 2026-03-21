use serde::Deserialize;
use std::fmt;

/// Raw session as deserialized from TOML config.
///
/// Optional fields are `None` when not specified by the user.
/// Use the resolve module to produce a [`ResolvedSession`] with defaults applied.
#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    /// Unique identifier (derived from the TOML table key, not deserialized directly).
    #[serde(skip)]
    pub name: String,
    /// IP address or hostname (required).
    pub host: String,
    /// SSH username (defaults to current OS user when resolved).
    pub username: Option<String>,
    /// Path to identity file, may contain `~` (defaults to system SSH behavior).
    pub ssh_key: Option<String>,
    /// SSH port (defaults to 22 when resolved).
    pub port: Option<u16>,
    /// Optional grouping/filtering labels.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Where the SSH key came from after resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    /// User explicitly configured an `ssh_key` path.
    Explicit,
    /// No key configured; SSH will use its own default behavior.
    SystemDefault,
}

impl fmt::Display for KeySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeySource::Explicit => write!(f, "explicit"),
            KeySource::SystemDefault => write!(f, "system_default"),
        }
    }
}

/// A session with all defaults resolved and paths expanded.
///
/// Every field is explicit — no further inference needed downstream.
#[derive(Debug, Clone)]
pub struct ResolvedSession {
    /// Session name.
    pub name: String,
    /// Host (IP or hostname).
    pub host: String,
    /// Resolved username (configured value or current OS user).
    pub username: String,
    /// Resolved port (configured value or 22).
    pub port: u16,
    /// Normalized key path if configured, `None` if using system default.
    pub ssh_key: Option<String>,
    /// Whether the key was explicitly configured or left to system default.
    pub key_source: KeySource,
    /// Computed display string, e.g. `user@host:22`.
    pub display_target: String,
    /// Tags for grouping and filtering.
    pub tags: Vec<String>,
}

/// Severity of a validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// Must be fixed before the session can be used.
    Error,
    /// Advisory; does not block usage.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

/// A warning or error found during config validation.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Whether this is an error or a warning.
    pub severity: Severity,
    /// The session this issue relates to, if applicable.
    pub session_name: Option<String>,
    /// The specific field with the problem, if applicable.
    pub field: Option<String>,
    /// Human-readable explanation.
    pub message: String,
    /// Stable identifier for testing and machine-readable output (e.g. `"missing-host"`).
    pub code: Option<String>,
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format: severity[code]: session "name" field "field": message
        write!(f, "{}", self.severity)?;
        if let Some(ref code) = self.code {
            write!(f, "[{code}]")?;
        }
        write!(f, ":")?;
        if let Some(ref name) = self.session_name {
            write!(f, " session \"{name}\"")?;
        }
        if let Some(ref field) = self.field {
            write!(f, " field \"{field}\"")?;
        }
        write!(f, " {}", self.message)
    }
}
