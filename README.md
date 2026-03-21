# russher

A Rust SSH toolkit for managing and connecting to named SSH sessions via a TOML config file.

## Overview

russher is a Cargo workspace containing two crates:

- **russh-core** — core library: config loading, session model, path resolution, and validation
- **russh-cli** — command-line interface (`russh`) built on top of russh-core

## Install

```bash
cargo build --release
# Binary is at: target/release/russh
```

Or install directly:

```bash
cargo install --path russh-cli
```

## Usage

```
russh [--config <path>] [COMMAND]
```

With no subcommand, `russh` opens an interactive menu.

### Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `russh` | — | Interactive session picker (default) |
| `russh menu` | — | Interactive session picker |
| `russh list` | — | List all configured sessions |
| `russh show <name>` | — | Show session details (raw and resolved) |
| `russh connect <name>` | `c` | Connect to a session by name |
| `russh check` | — | Validate config and report issues |

### Examples

```bash
# List all sessions
russh list

# Connect to a session named "dev-server"
russh connect dev-server
russh c dev-server

# Show resolved details for a session
russh show prod-web

# Validate your config
russh check

# Use a custom config file
russh --config ~/work/ssh.toml list
```

## Configuration

The default config location is:

```
~/.config/russh/config.toml
```

The `XDG_CONFIG_HOME` environment variable is respected. Override the path at runtime with `--config`.

### Config Format

Sessions are defined as TOML tables under `[sessions.<name>]`.

```toml
[sessions.<name>]
host     = "hostname or IP"   # required
username = "user"             # optional; defaults to current OS user
port     = 22                 # optional; defaults to 22
ssh_key  = "~/.ssh/id_ed25519" # optional; defaults to system SSH behavior
tags     = ["tag1", "tag2"]   # optional; used for grouping/filtering
```

Only `host` is required. All other fields are optional.

See [`examples/config.toml`](examples/config.toml) for a complete sample configuration.

## Development

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p russh-core
cargo test -p russh-cli
```
