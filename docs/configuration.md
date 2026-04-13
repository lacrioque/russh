# Configuration

## Config file location

russh looks for its config file in this order:

1. `--config <PATH>` flag (highest priority)
2. `$XDG_CONFIG_HOME/russh/config.toml` (if `XDG_CONFIG_HOME` is set and non-empty)
3. `~/.config/russh/config.toml` (fallback)

Tilde (`~`) in paths is expanded to your home directory.

## Format

Sessions are defined as TOML tables under `[sessions.<name>]`:

```toml
[sessions.myserver]
host     = "192.168.1.10"
username = "admin"
port     = 2222
ssh_key  = "~/.ssh/id_ed25519"
tags     = ["prod", "web"]
jump     = "bastion"
```

## Session fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `host` | string | **yes** | — | IP address or hostname |
| `username` | string | no | Current OS user | SSH login username |
| `port` | integer | no | `22` | SSH port (1-65535) |
| `ssh_key` | string | no | System default | Path to SSH private key |
| `tags` | array of strings | no | `[]` | Labels for grouping/filtering |
| `jump` | string | no | — | Jump host (session name or `user@host:port`) |

### host

The remote host address. Can be an IP address (`192.168.1.10`) or hostname (`example.com`). This is the only required field.

`russh check` emits a warning if a hostname is used instead of an IP address.

### username

SSH login user. If omitted, defaults to your current OS user (`$USER` on Unix, `$USERNAME` on Windows).

### port

SSH port number, 1 through 65535. Defaults to `22` if omitted. A port of `0` is an error.

### ssh_key

Path to an SSH private key file. Tilde expansion is supported (`~/.ssh/id_ed25519` becomes `/home/you/.ssh/id_ed25519`).

If omitted, SSH uses its default key search behavior (typically `~/.ssh/id_rsa`, `~/.ssh/id_ed25519`, etc.).

`russh check` emits a warning if the specified key file does not exist.

### tags

An array of strings used for grouping and filtering sessions. Tags are automatically trimmed, deduplicated, and sorted alphabetically.

Tags are used by `russh deploy --tag <tag>` to deploy config to matching sessions.

```toml
tags = ["prod", "web", "us-east"]
```

### jump

Specifies a jump host (SSH ProxyJump) for reaching the target. See [Jump hosts](jump-hosts.md) for details.

Two formats are accepted:

- **Session name**: `jump = "bastion"` — looks up the named session and resolves its connection details
- **Arbitrary host spec**: `jump = "ops@jumpbox.example.com:2222"` — passed directly to `ssh -J`

## Defaults summary

| Field | Default value |
|-------|--------------|
| `username` | Current OS user |
| `port` | `22` |
| `ssh_key` | System SSH default |
| `tags` | `[]` (empty) |
| `jump` | None (direct connection) |

## Full example

```toml
# Minimal session — only host required
[sessions.homelab]
host = "192.168.1.10"

# Full session with all fields
[sessions.dev-server]
host = "10.0.0.50"
username = "deploy"
port = 2222
ssh_key = "~/.ssh/id_ed25519"
tags = ["dev", "linux"]

# Production server
[sessions.prod-web]
host = "prod.example.com"
username = "ubuntu"
ssh_key = "~/.ssh/prod_key"
tags = ["prod", "web"]

# Jump host (bastion)
[sessions.bastion]
host = "bastion.example.com"
username = "ops"
port = 2222
tags = ["infra", "bastion"]

# Internal server accessible through the bastion
[sessions.internal-db]
host = "10.0.0.50"
username = "dbadmin"
port = 5432
jump = "bastion"
tags = ["internal", "database"]
```

See also [`examples/config.toml`](../examples/config.toml) for a ready-to-copy sample.

## Validation rules

Run `russh check` to validate your config. The following rules are checked:

| Severity | Code | Condition |
|----------|------|-----------|
| ERROR | `missing-host` | `host` is empty |
| ERROR | `invalid-port` | `port` is 0 |
| ERROR | `empty-jump-host` | `jump` is set but empty |
| ERROR | `circular-jump` | Session references itself as jump host |
| WARNING | `missing-key-file` | `ssh_key` points to a file that doesn't exist |
| WARNING | `hostname-not-ip` | `host` is a hostname rather than an IP address |

Errors are launch-blocking (prevent connection). Warnings are informational.
