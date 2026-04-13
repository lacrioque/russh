use anyhow::{Context, Result};
use russh_core::paths::procedures_path;
use russh_core::proc_config::load_procedures;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "SESSION")]
    session: String,
    #[tabled(rename = "DESCRIPTION")]
    description: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

pub fn run(config_override: Option<&str>) -> Result<()> {
    let path =
        procedures_path(config_override).context("could not determine procedures config path")?;

    let procedures = load_procedures(&path)
        .with_context(|| format!("failed to load procedures from {}", path.display()))?;

    let rows: Vec<Row> = procedures
        .iter()
        .map(|p| Row {
            name: p.name.clone(),
            session: p.session.clone(),
            description: p.description.as_deref().unwrap_or("(none)").to_string(),
            tags: if p.tags.is_empty() {
                String::new()
            } else {
                p.tags.join(",")
            },
        })
        .collect();

    if rows.is_empty() {
        println!("No procedures configured.");
        return Ok(());
    }

    println!("{}", Table::new(rows));
    Ok(())
}
