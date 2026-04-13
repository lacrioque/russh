# Getting Started

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.56 or later
- SSH client (`ssh`, `scp`) installed and on your `PATH`

### Build from source

```bash
git clone <repo-url>
cd russh

# Build the release binary (optimized, stripped, LTO)
cargo build --release

# Binary is at: target/release/russh
```

### Install with Cargo

```bash
cargo install --path russh-cli
```

This places the `russh` binary in your Cargo bin directory (typically `~/.cargo/bin/`).

## First configuration

russh reads sessions from a TOML config file. The default location is:

```
~/.config/russh/config.toml
```

Create the file and add your first session:

```bash
mkdir -p ~/.config/russh
```

```toml
# ~/.config/russh/config.toml

[sessions.myserver]
host = "192.168.1.10"
username = "admin"
```

Only `host` is required. If `username` is omitted, your current OS user is used. If `port` is omitted, it defaults to 22.

### Or use the CLI to add sessions

```bash
russh insert myserver admin@192.168.1.10
```

This appends the session to your config file automatically.

## First connection

Connect by session name:

```bash
russh connect myserver
# or use the alias:
russh c myserver
```

Or launch the interactive picker (no arguments):

```bash
russh
```

This opens a TUI menu where you can search and select from your configured sessions.

## Verify your config

Check for errors and warnings:

```bash
russh check
```

Exit codes: `0` = no issues, `1` = warnings only, `2` = errors found.

## List your sessions

```bash
russh list
```

Outputs a table with columns: NAME, HOST, USER, PORT, KEY, TAGS.

## What's next

- [Configuration reference](configuration.md) for all config options and defaults
- [Commands reference](commands.md) for every CLI command and flag
- [Procedures](procedures.md) for running named command sequences on remote hosts
- [Jump hosts](jump-hosts.md) for accessing internal hosts through bastion servers
