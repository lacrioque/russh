# Russh — AI Agent Usage Guide

Russh is an SSH session manager. It stores named SSH sessions in a TOML config
and delegates to the system `ssh` binary. AI coding agents can use it to run
commands on remote hosts without handling raw SSH credentials or connection strings.

## Quick start for agents

```bash
# Discover available hosts
russh list                              # table format
russh list --json                       # machine-readable

# Run a command on a remote host
russh exec dev-server "uptime"          # output inherits terminal
russh exec dev-server "df -h" --to-std  # captured, written to stdout/stderr
russh exec dev-server "whoami" --json   # structured JSON output

# Inspect a session before using it
russh show dev-server

# Validate config
russh check
```

## Core commands

### Read-only (always safe, no approval needed)

| Command | Purpose |
|---------|---------|
| `russh list` | List all sessions (table) |
| `russh list --json` | List all sessions (JSON array of resolved sessions) |
| `russh show <name>` | Show raw + resolved details for one session |
| `russh check` | Validate config, report errors/warnings |
| `russh export` | Print raw config file to stdout |
| `russh version` | Show version and config path |
| `russh proc list` | List all procedures |
| `russh proc show <name>` | Show procedure details + SSH command preview |
| `russh proc check` | Validate all procedures |

### Remote execution (requires user approval)

| Command | Purpose |
|---------|---------|
| `russh exec <session> "<command>"` | Run a one-off command on a remote host |
| `russh exec <session> "<cmd>" --json` | Same, but output as JSON with exit code |
| `russh exec <session> "<cmd>" --to-std` | Same, but capture and write to stdout/stderr |
| `russh exec <session> "<cmd>" -T` | Disable pseudo-TTY (for non-interactive commands) |
| `russh proc run <name>` | Run a named procedure |
| `russh copy <src> <src-path> <dst> [dst-path]` | Copy a file between two sessions via SCP (dst path defaults to `~`) |
| `russh connect <name>` | Open interactive SSH session (blocks, replaces process) |

### Config modification (requires user approval)

| Command | Purpose |
|---------|---------|
| `russh insert <name> user@host` | Add a new session |
| `russh edit <name> --host X` | Modify an existing session |
| `russh deploy <name>` | Push config to remote host via SCP |

## The `exec` command — primary tool for agents

`russh exec` is the command designed for programmatic use. It resolves the
session, validates it, runs the command via SSH, and returns.

```bash
# Basic — output goes to terminal (inherited stdio)
russh exec prod-web "systemctl status nginx"

# Captured — stdout/stderr are captured and replayed cleanly
russh exec prod-web "systemctl status nginx" --to-std

# JSON — structured output for parsing
russh exec prod-web "systemctl status nginx" --json
```

### JSON output format

```json
{
  "session": "prod-web",
  "command": "systemctl status nginx",
  "exit_code": 0,
  "stdout": "● nginx.service - A high performance web server...\n",
  "stderr": ""
}
```

The process exit code mirrors the remote command's exit code, so you can
check `$?` even in JSON mode.

### When to use which flag

| Scenario | Flag | Why |
|----------|------|-----|
| Quick check, human-readable | (none) | Output streams directly |
| Parsing output programmatically | `--json` | Structured, includes exit code |
| Piping output to another command | `--to-std` | Clean capture without JSON wrapper |
| Non-interactive command (no TTY) | `-T` | Avoids TTY allocation errors |

## `list --json` output format

```json
[
  {
    "name": "dev-server",
    "host": "10.0.0.50",
    "username": "deploy",
    "port": 2222,
    "ssh_key": "/home/user/.ssh/id_ed25519",
    "key_source": "explicit",
    "display_target": "deploy@10.0.0.50:2222",
    "tags": ["dev", "linux"],
    "jump_target": null
  }
]
```

## Procedures

Procedures are named command sequences stored in `~/.config/russh/procedures.toml`.
Use them for multi-step operations.

```bash
# Inspect
russh proc list
russh proc show health-check

# Run
russh proc run health-check
russh proc run health-check --log /tmp/health.log  # redirect output to file
```

## Permission boundaries

| Action | Safety | Notes |
|--------|--------|-------|
| `list`, `show`, `check`, `export`, `version` | Safe | Read-only, local only |
| `proc list`, `proc show`, `proc check` | Safe | Read-only, local only |
| `exec <session> "<cmd>"` | Ask first | Executes on remote host |
| `proc run <name>` | Ask first | Executes on remote host |
| `connect <session>` | Ask first | Replaces process with SSH |
| `insert`, `edit`, `deploy` | Ask first | Modifies config or remote state |

**Rule**: Anything that touches a remote host or modifies config — ask the user first.
Read-only local commands are always safe.

## Config location

Default: `~/.config/russh/config.toml` (XDG-aware)

Override per-command: `russh --config /path/to/config.toml <command>`

## Typical agent workflows

### Check if a service is running
```bash
russh exec prod-web "systemctl is-active nginx" --to-std
```

### Check disk space across hosts
```bash
# Discover hosts
russh list --json | jq '.[].name' -r

# For each host
russh exec prod-web "df -h /" --to-std
russh exec prod-db "df -h /" --to-std
```

### Copy a file between hosts
```bash
# Direct copy (single scp)
russh copy prod-db ~/dump.sql backup-host ~/archives/

# When sessions use different jump hosts, russh automatically falls back
# to a two-step copy via a local temp file — each leg uses its own jump.
russh copy -n prod-db ~/dump.sql staging-db ~/  # preview first
```

### Verify a deployment
```bash
russh exec staging "cat /etc/app/version.txt && systemctl is-active app" --json
```

### Debug connectivity
```bash
russh check                    # validate config
russh show <name>              # inspect resolved session
russh exec <name> "echo ok"   # test connectivity
```

### Run a multi-step operation
```bash
russh proc show deploy         # inspect what it does
russh proc run deploy          # run it (with user approval)
```
