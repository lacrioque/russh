use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "russh", version, about = "A custom SSH client")]
struct Cli {
    /// Target host to connect to
    host: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(host) = cli.host {
        println!("russh {} — connecting to {host}", russh_core::version());
    } else {
        println!("russh {}", russh_core::version());
    }

    Ok(())
}
