use serde::{Deserialize, Serialize};
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
    /// Optional jump host — session name or arbitrary host spec (e.g. `user@host:port`).
    pub jump: Option<String>,
}

/// Helper for `fail_fast` default — serde requires a function path.
fn default_true() -> bool {
    true
}

/// Raw procedure as deserialized from TOML config.
///
/// Represents a named sequence of shell commands to execute on a remote host.
/// Use the resolve module to produce a [`ResolvedProcedure`] with the session
/// resolved and commands joined.
#[derive(Debug, Clone, Deserialize)]
pub struct Procedure {
    /// Unique identifier (derived from the TOML table key, not deserialized directly).
    #[serde(skip)]
    pub name: String,
    /// Name of the session (from config.toml) to execute on.
    pub session: String,
    /// Shell commands to execute in order.
    pub commands: Vec<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// If true, disable TTY allocation (ssh -T). Defaults to false.
    #[serde(default)]
    pub no_tty: bool,
    /// If true, join commands with `&&` (stop on first failure).
    /// If false, join with `;` (run all regardless). Defaults to true.
    #[serde(default = "default_true")]
    pub fail_fast: bool,
    /// Optional grouping/filtering labels.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A procedure with its session resolved and commands joined into a shell command.
#[derive(Debug, Clone)]
pub struct ResolvedProcedure {
    /// Procedure name.
    pub name: String,
    /// The fully resolved session to execute on.
    pub session: ResolvedSession,
    /// Original command list.
    pub commands: Vec<String>,
    /// Commands joined with `&&` or `;` depending on `fail_fast`.
    pub shell_command: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// Whether TTY allocation is disabled.
    pub no_tty: bool,
    /// Whether to stop on first command failure.
    pub fail_fast: bool,
    /// Tags for grouping and filtering.
    pub tags: Vec<String>,
}

/// Where the SSH key came from after resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Serialize)]
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
    /// Resolved jump host target string (e.g. `ops@bastion:2222`), or `None`.
    pub jump_target: Option<String>,
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
    /// The procedure this issue relates to, if applicable.
    pub procedure_name: Option<String>,
    /// The specific field with the problem, if applicable.
    pub field: Option<String>,
    /// Human-readable explanation.
    pub message: String,
    /// Stable identifier for testing and machine-readable output (e.g. `"missing-host"`).
    pub code: Option<String>,
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format: severity[code]: [procedure "name"] [session "name"] field "field": message
        write!(f, "{}", self.severity)?;
        if let Some(ref code) = self.code {
            write!(f, "[{code}]")?;
        }
        write!(f, ":")?;
        if let Some(ref name) = self.procedure_name {
            write!(f, " procedure \"{name}\"")?;
        }
        if let Some(ref name) = self.session_name {
            write!(f, " session \"{name}\"")?;
        }
        if let Some(ref field) = self.field {
            write!(f, " field \"{field}\"")?;
        }
        write!(f, " {}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- KeySource ---

    #[test]
    fn key_source_display_explicit() {
        assert_eq!(KeySource::Explicit.to_string(), "explicit");
    }

    #[test]
    fn key_source_display_system_default() {
        assert_eq!(KeySource::SystemDefault.to_string(), "system_default");
    }

    #[test]
    fn key_source_clone_and_eq() {
        let a = KeySource::Explicit;
        assert_eq!(a.clone(), KeySource::Explicit);
        assert_ne!(KeySource::Explicit, KeySource::SystemDefault);
    }

    // --- Severity ---

    #[test]
    fn severity_display_error() {
        assert_eq!(Severity::Error.to_string(), "error");
    }

    #[test]
    fn severity_display_warning() {
        assert_eq!(Severity::Warning.to_string(), "warning");
    }

    #[test]
    fn severity_clone_and_eq() {
        assert_eq!(Severity::Error.clone(), Severity::Error);
        assert_ne!(Severity::Error, Severity::Warning);
    }

    // --- ValidationIssue Display ---

    fn full_issue() -> ValidationIssue {
        ValidationIssue {
            severity: Severity::Error,
            session_name: Some("myhost".into()),
            procedure_name: None,
            field: Some("host".into()),
            message: "host must not be empty".into(),
            code: Some("missing-host".into()),
        }
    }

    #[test]
    fn validation_issue_display_full() {
        let s = full_issue().to_string();
        assert!(s.contains("error"), "missing severity: {s}");
        assert!(s.contains("missing-host"), "missing code: {s}");
        assert!(s.contains("myhost"), "missing session name: {s}");
        assert!(s.contains("host"), "missing field: {s}");
        assert!(s.contains("host must not be empty"), "missing message: {s}");
    }

    #[test]
    fn validation_issue_display_no_code() {
        let issue = ValidationIssue {
            severity: Severity::Warning,
            session_name: Some("s".into()),
            procedure_name: None,
            field: None,
            message: "advisory note".into(),
            code: None,
        };
        let s = issue.to_string();
        assert!(s.starts_with("warning:"), "expected no code brackets: {s}");
        assert!(s.contains("advisory note"), "{s}");
    }

    #[test]
    fn validation_issue_display_no_session_name() {
        let issue = ValidationIssue {
            severity: Severity::Error,
            session_name: None,
            procedure_name: None,
            field: Some("port".into()),
            message: "bad port".into(),
            code: Some("invalid-port".into()),
        };
        let s = issue.to_string();
        assert!(!s.contains("session"), "unexpected session prefix: {s}");
        assert!(s.contains("invalid-port"), "{s}");
        assert!(s.contains("bad port"), "{s}");
    }

    #[test]
    fn validation_issue_display_no_optional_fields() {
        let issue = ValidationIssue {
            severity: Severity::Warning,
            session_name: None,
            procedure_name: None,
            field: None,
            message: "generic warning".into(),
            code: None,
        };
        let s = issue.to_string();
        assert_eq!(s, "warning: generic warning");
    }

    #[test]
    fn validation_issue_clone() {
        let issue = full_issue();
        let cloned = issue.clone();
        assert_eq!(cloned.message, issue.message);
        assert_eq!(cloned.code, issue.code);
    }

    #[test]
    fn validation_issue_display_with_procedure_name() {
        let issue = ValidationIssue {
            severity: Severity::Error,
            session_name: None,
            procedure_name: Some("deploy".into()),
            field: Some("session".into()),
            message: "session must not be empty".into(),
            code: Some("empty-session".into()),
        };
        let s = issue.to_string();
        assert!(s.contains("procedure \"deploy\""), "missing procedure: {s}");
        assert!(s.contains("empty-session"), "missing code: {s}");
        assert!(!s.contains("session \""), "unexpected session prefix: {s}");
    }

    #[test]
    fn validation_issue_display_with_both_procedure_and_session() {
        let issue = ValidationIssue {
            severity: Severity::Warning,
            session_name: Some("web".into()),
            procedure_name: Some("deploy".into()),
            field: None,
            message: "some advisory".into(),
            code: None,
        };
        let s = issue.to_string();
        assert!(s.contains("procedure \"deploy\""), "{s}");
        assert!(s.contains("session \"web\""), "{s}");
    }

    // --- Session construction ---

    #[test]
    fn session_field_defaults() {
        let s = Session {
            name: "demo".into(),
            host: "1.2.3.4".into(),
            username: None,
            ssh_key: None,
            port: None,
            tags: vec![],
            jump: None,
        };
        assert_eq!(s.name, "demo");
        assert_eq!(s.host, "1.2.3.4");
        assert!(s.username.is_none());
        assert!(s.ssh_key.is_none());
        assert!(s.port.is_none());
        assert!(s.tags.is_empty());
        assert!(s.jump.is_none());
    }

    #[test]
    fn session_clone_is_independent() {
        let s = Session {
            name: "orig".into(),
            host: "10.0.0.1".into(),
            username: Some("alice".into()),
            ssh_key: Some("~/.ssh/id_rsa".into()),
            port: Some(2222),
            tags: vec!["prod".into()],
            jump: None,
        };
        let mut clone = s.clone();
        clone.name = "copy".into();
        assert_eq!(s.name, "orig");
    }

    // --- ResolvedSession construction ---

    #[test]
    fn resolved_session_clone() {
        let r = ResolvedSession {
            name: "r".into(),
            host: "10.0.0.1".into(),
            username: "bob".into(),
            port: 22,
            ssh_key: Some("/keys/k".into()),
            key_source: KeySource::Explicit,
            display_target: "bob@10.0.0.1:22".into(),
            tags: vec!["web".into()],
            jump_target: None,
        };
        let c = r.clone();
        assert_eq!(c.name, r.name);
        assert_eq!(c.key_source, r.key_source);
        assert_eq!(c.tags, r.tags);
    }

    // --- Procedure ---

    #[test]
    fn procedure_fail_fast_defaults_to_true() {
        let toml = r#"
            session = "dev"
            commands = ["echo hi"]
        "#;
        let p: Procedure = toml::from_str(toml).unwrap();
        assert!(p.fail_fast);
    }

    #[test]
    fn procedure_no_tty_defaults_to_false() {
        let toml = r#"
            session = "dev"
            commands = ["echo hi"]
        "#;
        let p: Procedure = toml::from_str(toml).unwrap();
        assert!(!p.no_tty);
    }

    #[test]
    fn procedure_clone_is_independent() {
        let p = Procedure {
            name: "deploy".into(),
            session: "prod".into(),
            commands: vec!["systemctl restart app".into()],
            description: Some("Deploy the app".into()),
            no_tty: false,
            fail_fast: true,
            tags: vec!["deploy".into()],
        };
        let mut c = p.clone();
        c.name = "other".into();
        assert_eq!(p.name, "deploy");
    }

    // --- ResolvedProcedure ---

    fn make_resolved_session() -> ResolvedSession {
        ResolvedSession {
            name: "db".into(),
            host: "10.0.0.1".into(),
            username: "admin".into(),
            port: 22,
            ssh_key: None,
            key_source: KeySource::SystemDefault,
            display_target: "admin@10.0.0.1:22".into(),
            tags: vec![],
            jump_target: None,
        }
    }

    #[test]
    fn resolved_procedure_clone() {
        let r = ResolvedProcedure {
            name: "backup".into(),
            session: make_resolved_session(),
            commands: vec!["pg_dump ...".into()],
            shell_command: "pg_dump ...".into(),
            description: Some("Run backup".into()),
            no_tty: true,
            fail_fast: false,
            tags: vec!["ops".into()],
        };
        let c = r.clone();
        assert_eq!(c.name, r.name);
        assert_eq!(c.session.name, r.session.name);
        assert_eq!(c.commands, r.commands);
        assert_eq!(c.shell_command, r.shell_command);
        assert_eq!(c.no_tty, r.no_tty);
        assert_eq!(c.fail_fast, r.fail_fast);
    }
}
