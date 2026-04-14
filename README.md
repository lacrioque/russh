# russher

A Rust SSH toolkit for managing named SSH sessions and running procedures on remote hosts via a TOML config.

## Features

- **Named sessions** — define SSH connections once, connect by name
- **Procedures** — store and run multi-command sequences on remote hosts
- **Jump hosts** — ProxyJump support for accessing internal networks
- **Config validation** — catch misconfigurations before connecting
- **Deploy** — sync your config to remote hosts via SCP
- **Interactive menu** — pick a session without remembering names

## Install

```bash
cargo install --path russh-cli
```

Or build from source:

```bash
cargo build --release
# Binary: target/release/russh
```

## Quick Start

```bash
# Add a session
russh insert dev-server deploy@10.0.0.50 -p 2222 -i ~/.ssh/id_ed25519

# Connect
russh connect dev-server

# Or just launch the interactive menu
russh
```

## Usage

```
russh [--config <path>] <COMMAND>
```

With no subcommand, `russh` opens an interactive session picker.

### Session Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `russh list [--json]` | — | List all configured sessions |
| `russh show <name>` | — | Show session details (raw and resolved) |
| `russh exec <name> <cmd>` | — | Run a one-off command on a remote host |
| `russh connect <name>` | `c` | Connect to a session by name |
| `russh insert <name> <target>` | `i` | Add a new session to the config |
| `russh edit [<name>]` | `e` | Edit a session (or open config in `$EDITOR`) |
| `russh check` | — | Validate config and report issues |
| `russh deploy [<name>]` | — | Deploy config to remote host(s) via SCP |
| `russh export` | — | Print current config to stdout |
| `russh menu` | — | Interactive session picker |
| `russh version` | — | Show version and config path |

### Procedure Commands

Procedures are named command sequences you run on remote sessions. They live in a separate config file (`~/.config/russh/procedures.toml`).

| Command | Description |
|---------|-------------|
| `russh proc list` | List all configured procedures |
| `russh proc show <name>` | Show details of a procedure |
| `russh proc check` | Validate all procedures |
| `russh proc run <name>` | Run a procedure on its configured session |
| `russh proc insert <name>` | Add a new procedure |
| `russh proc edit` | Open procedures config in `$EDITOR` |
| `russh proc export [<name>]` | Export procedures as TOML or shell script |

### Examples

```bash
# List all sessions
russh list
russh list --json

# Run a command on a remote host
russh exec dev-server "uptime"
russh exec dev-server "df -h" --json
russh exec dev-server "whoami" --to-std

# Connect to a session (interactive)
russh connect dev-server
russh c dev-server

# Show resolved details for a session
russh show prod-web

# Add a session with a jump host
russh insert internal-db dbadmin@10.0.0.50 -p 5432 -J bastion

# Edit a session's port
russh edit dev-server -p 3022

# Deploy config to all hosts tagged "prod"
russh deploy --tag prod

# Validate your config
russh check

# Run a procedure
russh proc run deploy

# Run a local script on a remote host
russh proc run --from-script ./setup.sh --session dev-server

# Export a procedure as a standalone shell script
russh proc export deploy --script
```

## Configuration

### Sessions

Default location: `~/.config/russh/config.toml`

The `XDG_CONFIG_HOME` environment variable is respected. Override at runtime with `--config`.

```toml
[sessions.dev-server]
host = "10.0.0.50"           # required
username = "deploy"           # optional; defaults to current OS user
port = 2222                   # optional; defaults to 22
ssh_key = "~/.ssh/id_ed25519" # optional; defaults to system SSH behavior
tags = ["dev", "linux"]       # optional; used for grouping/filtering

[sessions.internal-db]
host = "10.0.0.50"
username = "dbadmin"
port = 5432
jump = "bastion"              # connect through another session as jump host
tags = ["internal", "database"]
```

Only `host` is required. All other fields are optional.

### Procedures

Default location: `~/.config/russh/procedures.toml`

```toml
[procedures.health-check]
session = "prod"              # required; session name to run on
commands = [                  # required; commands to execute in order
    "systemctl is-active myapp",
    "curl -sf http://localhost:8080/health",
    "df -h /",
]
description = "Quick health check on production"
fail_fast = false             # optional; default true (stop on first failure)
no_tty = false                # optional; default false (allocate pseudo-TTY)
tags = ["monitoring"]         # optional
```

See [`examples/config.toml`](examples/config.toml) and [`examples/procedures.toml`](examples/procedures.toml) for complete samples.

## Project Structure

russher is a Cargo workspace with two crates:

- **russh-core** — core library: config loading, session/procedure models, validation
- **russh-cli** — CLI binary (`russh`) built on top of russh-core

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

## License

MIT
