use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod ui;

use ui::SessionPicker as _;
use ui::inquire::InquirePicker;

#[derive(Parser)]
#[command(name = "russh", version, about = "A custom SSH client")]
struct Cli {
    /// Path to config file (overrides default location)
    #[arg(long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List available SSH sessions/hosts
    List,
    /// Show details of a session or host
    Show {
        /// Session or host identifier
        target: String,
    },
    /// Validate all sessions and report issues
    Check,
    /// Connect to a session by name
    #[command(alias = "c")]
    Connect {
        /// Session name
        session: String,
    },
    /// Interactive menu
    Menu,
}

fn default_config_path() -> PathBuf {
    dirs_next::config_dir()
        .map(|d| d.join("russh").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("~/.config/russh/config.toml"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = cli.config.as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(default_config_path);

    match cli.command {
        Command::List => {
            commands::list::run(cli.config.as_deref())?;
        }
        Command::Show { target } => {
            commands::show::run(&target, &config_path)?;
        }
        Command::Check => {
            commands::check::run(cli.config.as_deref());
        }
        Command::Connect { session } => {
            commands::connect::run(&session, cli.config.as_deref())?;
        }
        Command::Menu => {
            let picker = InquirePicker;
            let sessions = vec![]; // placeholder: resolved sessions injected here by ru-jba.7
            match picker.pick(&sessions)? {
                Some(session) => println!("russh {} — connecting to {}", russh_core::version(), session.display_target),
                None => println!("russh {} — no session selected", russh_core::version()),
            }
        }
    }

    Ok(())
}
