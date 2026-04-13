use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod ui;

#[derive(Parser)]
#[command(name = "russh", version, about = "A custom SSH client")]
struct Cli {
    /// Path to config file (overrides default location)
    #[arg(long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
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
    /// Insert a new session into the config
    ///
    /// Example: russh i myconn user@1.2.3.4 -p 2222 -i ~/.ssh/id_ed25519
    #[command(alias = "i")]
    Insert {
        /// Session name
        name: String,
        /// SSH target (user@host or just host)
        target: String,
        /// SSH port
        #[arg(short, long)]
        port: Option<u16>,
        /// Path to SSH identity file (private key)
        #[arg(short = 'i', long = "identity")]
        identity: Option<String>,
        /// Jump host — session name or arbitrary host (e.g. user@host:port)
        #[arg(short = 'J', long = "jump")]
        jump: Option<String>,
    },
    /// Edit an existing session (or open config in $EDITOR)
    ///
    /// Example: russh edit myconn -p 2222 --user deploy
    #[command(alias = "e")]
    Edit {
        /// Session name (omit to open config in $EDITOR)
        name: Option<String>,
        /// New SSH port (use NONE to remove)
        #[arg(short, long)]
        port: Option<String>,
        /// New SSH identity file path (use NONE to remove)
        #[arg(short = 'i', long = "identity")]
        identity: Option<String>,
        /// New jump host session name (use NONE to remove)
        #[arg(short = 'J', long = "jump")]
        jump: Option<String>,
        /// New host address
        #[arg(long)]
        host: Option<String>,
        /// New SSH username (use NONE to remove)
        #[arg(long)]
        user: Option<String>,
    },
    /// Deploy config to remote host(s) via SCP
    Deploy {
        /// Session name to deploy to (omit for --all or --tag)
        session: Option<String>,
        /// Deploy to all configured sessions
        #[arg(long)]
        all: bool,
        /// Deploy to sessions matching this tag
        #[arg(long)]
        tag: Option<String>,
        /// Show what would be done without executing
        #[arg(long, short = 'n')]
        dry_run: bool,
    },
    /// Print the current config file contents to stdout
    Export,
    /// Manage procedures (named command sequences on remote hosts)
    Proc {
        /// Path to procedures config file (overrides default location)
        #[arg(long = "from-config")]
        from_config: Option<String>,

        #[command(subcommand)]
        action: ProcCommand,
    },
    /// Interactive menu (default when no subcommand given)
    Menu,
    /// Show version and default config path
    Version,
}

#[derive(Subcommand)]
enum ProcCommand {
    /// List all configured procedures
    List,
    /// Show details of a named procedure
    Show {
        /// Procedure name
        name: String,
    },
    /// Validate all procedures and report issues
    Check,
    /// Run a named procedure on a remote host
    Run {
        /// Procedure name (from procedures.toml)
        name: String,
        /// Redirect output to a log file
        #[arg(long)]
        log: Option<String>,
        /// Disable pseudo-TTY allocation (overrides procedure setting)
        #[arg(long, short = 'T')]
        no_tty: bool,
        /// Pipe a local script to the remote host instead of running a procedure
        #[arg(long)]
        from_script: Option<String>,
        /// Session name to run the script on (required with --from-script)
        #[arg(long, requires = "from_script")]
        session: Option<String>,
    },
}

fn default_config_path() -> PathBuf {
    dirs_next::config_dir()
        .map(|d| d.join("russh").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("~/.config/russh/config.toml"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = cli
        .config
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(default_config_path);

    match cli.command.unwrap_or(Command::Menu) {
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
        Command::Insert {
            name,
            target,
            port,
            identity,
            jump,
        } => {
            commands::insert::run(
                &name,
                &target,
                port,
                identity.as_deref(),
                jump.as_deref(),
                cli.config.as_deref(),
            )?;
        }
        Command::Edit {
            name,
            port,
            identity,
            jump,
            host,
            user,
        } => {
            commands::edit::run(
                name.as_deref(),
                host.as_deref(),
                user.as_deref(),
                port.as_deref(),
                identity.as_deref(),
                jump.as_deref(),
                cli.config.as_deref(),
            )?;
        }
        Command::Deploy {
            session,
            all,
            tag,
            dry_run,
        } => {
            commands::deploy::run(
                session.as_deref(),
                all,
                tag.as_deref(),
                dry_run,
                cli.config.as_deref(),
            )?;
        }
        Command::Export => {
            commands::export::run(&config_path)?;
        }
        Command::Proc {
            from_config,
            action,
        } => match action {
            ProcCommand::List => {
                commands::proc::list::run(from_config.as_deref())?;
            }
            ProcCommand::Show { name } => {
                commands::proc::show::run(
                    &name,
                    from_config.as_deref(),
                    cli.config.as_deref(),
                )?;
            }
            ProcCommand::Check => {
                commands::proc::check::run(from_config.as_deref(), cli.config.as_deref());
            }
            ProcCommand::Run {
                name,
                log,
                no_tty,
                from_script,
                session,
            } => {
                commands::proc::run::run(
                    &name,
                    cli.config.as_deref(),
                    from_config.as_deref(),
                    from_script.as_deref(),
                    session.as_deref(),
                    log.as_deref(),
                    no_tty,
                )?;
            }
        },
        Command::Menu => {
            commands::menu::run(cli.config.as_deref())?;
        }
        Command::Version => {
            commands::version::run();
        }
    }

    Ok(())
}
