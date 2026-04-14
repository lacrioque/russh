# russh-cli

CLI tool for managing and connecting to SSH sessions. Built on [russh-core](https://crates.io/crates/russh-core).

Define your SSH sessions once in a TOML config, then connect by name, run procedures on remote hosts, and manage your config — all from one tool.

## Install

```bash
cargo install russh-cli
```

The binary is called `russh`.

## Quick start

```bash
# Add a session
russh insert dev-server deploy@10.0.0.50 -p 2222 -i ~/.ssh/id_ed25519

# Connect by name
russh connect dev-server

# Or launch the interactive picker
russh
```

## Commands

### Sessions

| Command | Alias | Description |
|---------|-------|-------------|
| `russh list` | — | List all configured sessions |
| `russh show <name>` | — | Show session details (raw and resolved) |
| `russh connect <name>` | `c` | Connect to a session |
| `russh insert <name> <target>` | `i` | Add a new session |
| `russh edit [<name>]` | `e` | Edit a session or open config in `$EDITOR` |
| `russh check` | — | Validate config and report issues |
| `russh deploy [<name>]` | — | Deploy config to remote hosts via SCP |
| `russh export` | — | Print current config to stdout |
| `russh menu` | — | Interactive session picker (default) |
| `russh version` | — | Show version and config path |

### Procedures

Procedures are named command sequences executed on remote sessions. Defined in `~/.config/russh/procedures.toml`.

| Command | Description |
|---------|-------------|
| `russh proc list` | List all procedures |
| `russh proc show <name>` | Show procedure details |
| `russh proc check` | Validate all procedures |
| `russh proc run <name>` | Run a procedure on its configured session |
| `russh proc insert <name>` | Add a new procedure |
| `russh proc edit` | Open procedures config in `$EDITOR` |
| `russh proc export [<name>]` | Export as TOML or shell script |

## Configuration

Default location: `~/.config/russh/config.toml` (XDG-aware, overridable with `--config`).

```toml
[sessions.dev-server]
host = "10.0.0.50"           # required
username = "deploy"           # defaults to current OS user
port = 2222                   # defaults to 22
ssh_key = "~/.ssh/id_ed25519" # defaults to system SSH behavior
tags = ["dev", "linux"]       # for grouping/filtering
jump = "bastion"              # connect through a jump host
```

```toml
# ~/.config/russh/procedures.toml
[procedures.health-check]
session = "prod"
commands = [
    "systemctl is-active myapp",
    "curl -sf http://localhost:8080/health",
]
description = "Quick health check"
fail_fast = true
```

## Features

- **Jump hosts** — ProxyJump support via session names or `user@host:port` specs
- **Config validation** — catch errors and warnings before connecting
- **Deploy** — push your config to remote hosts with backup and dry-run
- **Interactive menu** — fuzzy-select sessions without remembering names
- **Procedures** — store and execute multi-command workflows on remote hosts

## License

GPL-3.0-only
