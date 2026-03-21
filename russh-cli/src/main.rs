use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "russh", version, about = "A custom SSH client")]
struct Cli {
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
    /// Check connectivity to a host
    Check {
        /// Host to check
        host: String,
    },
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
        Command::Check { host } => {
            println!("russh {} — check {host}", russh_core::version());
        }
        Command::Connect { host } => {
            println!("russh {} — connect {host}", russh_core::version());
        }
        Command::Menu => {
            println!("russh {} — menu", russh_core::version());
        }
    }

    Ok(())
}
