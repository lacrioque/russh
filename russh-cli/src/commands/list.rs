use anyhow::{Context, Result};
use russh_core::config::load_config;
use russh_core::model::KeySource;
use russh_core::paths::config_path;
use russh_core::resolve::resolve_session;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "HOST")]
    host: String,
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "PORT")]
    port: u16,
    #[tabled(rename = "KEY")]
    key: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

pub fn run(config_override: Option<&str>) -> Result<()> {
    let path = config_path(config_override)
        .context("could not determine config path")?;

    let sessions = load_config(&path)
        .with_context(|| format!("failed to load config: {}", path.display()))?;

    let rows: Vec<Row> = sessions
        .iter()
        .map(|s| {
            let r = resolve_session(s);
            let key = match r.key_source {
                KeySource::Explicit => r.ssh_key.unwrap_or_default(),
                KeySource::SystemDefault => "system default".to_string(),
            };
            Row {
                name: r.name,
                host: r.host,
                user: r.username,
                port: r.port,
                key,
                tags: r.tags.join(","),
            }
        })
        .collect();

    if rows.is_empty() {
        println!("No sessions configured.");
        return Ok(());
    }

    println!("{}", Table::new(rows));
    Ok(())
}
