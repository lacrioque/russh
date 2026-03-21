use anyhow::Result;
use clap::{Parser, Subcommand};

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
    /// Connect to a host
    #[command(alias = "c")]
    Connect {
        /// Host to connect to
        host: String,
    },
    /// Interactive menu
    Menu,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::List => {
            println!("russh {} — list", russh_core::version());
        }
        Command::Show { target } => {
            println!("russh {} — show {target}", russh_core::version());
        }
        Command::Check => {
            commands::check::run(cli.config.as_deref());
        }
        Command::Connect { host } => {
            println!("russh {} — connect {host}", russh_core::version());
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
