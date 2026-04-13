use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};

/// Print the raw config file contents to stdout.
pub fn run(config_path: &Path) -> Result<()> {
    let contents = fs::read_to_string(config_path)
        .with_context(|| format!("config file not found: {}", config_path.display()))?;
    print!("{contents}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn write_config(content: &str) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "{content}").unwrap();
        tmp
    }

    #[test]
    fn export_prints_config_contents() {
        let content = "[sessions.dev]\nhost = \"10.0.0.1\"\n";
        let tmp = write_config(content);
        assert!(run(tmp.path()).is_ok());
    }

    #[test]
    fn export_missing_file_errors() {
        let result = run(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("config file not found"), "{err}");
    }
}
