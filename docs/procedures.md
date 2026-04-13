# Procedures

Procedures are named sequences of shell commands that run on a remote session. They are defined in `procedures.toml` and executed with `russh proc run`.

## Procedures file location

russh looks for `procedures.toml` using the same resolution as config.toml:

1. `--from-config <PATH>` flag on `russh proc` commands
2. `$XDG_CONFIG_HOME/russh/procedures.toml`
3. `~/.config/russh/procedures.toml`

## Format

Procedures are TOML tables under `[procedures.<name>]`:

```toml
[procedures.deploy]
session = "prod"
commands = [
    "systemctl stop myapp",
    "rsync -avz /local/build/ /opt/myapp/",
    "systemctl start myapp",
]
description = "Deploy the application to production"
tags = ["deploy", "prod"]
```

## Procedure fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `session` | string | **yes** | — | Session name from config.toml to execute on |
| `commands` | array of strings | **yes** | — | Shell commands to run remotely |
| `description` | string | no | — | Human-readable description |
| `no_tty` | boolean | no | `false` | Disable TTY allocation (`ssh -T`) |
| `fail_fast` | boolean | no | `true` | Stop on first failure (`&&`) or continue (`; `) |
| `tags` | array of strings | no | `[]` | Labels for grouping |

### session

Must reference an existing session name in your config.toml. The session's connection details (host, user, port, key, jump host) are resolved automatically.

### commands

A non-empty array of shell commands. How they are joined depends on `fail_fast`:

- `fail_fast = true` (default): commands joined with ` && ` — execution stops at the first failure
- `fail_fast = false`: commands joined with ` ; ` — all commands run regardless of failures

### no_tty

When `true`, SSH runs with `-T` (no pseudo-terminal). Useful for non-interactive commands like backups, cron jobs, or commands that produce binary output.

Can be overridden at runtime with `russh proc run --no-tty`.

### fail_fast

Controls error handling:

- `true` (default): `cmd1 && cmd2 && cmd3` — stops at first non-zero exit
- `false`: `cmd1 ; cmd2 ; cmd3` — runs all commands even if earlier ones fail

Use `false` for "best effort" procedures like health checks where you want all results.

## Full example

```toml
[procedures.deploy]
session = "prod"
commands = [
    "systemctl stop myapp",
    "rsync -avz /local/build/ /opt/myapp/",
    "systemctl start myapp",
]
description = "Deploy the application to production"
tags = ["deploy", "prod"]

[procedures.backup]
session = "db"
commands = [
    "pg_dump -Fc mydb > /tmp/mydb_$(date +%Y%m%d).dump",
    "aws s3 cp /tmp/mydb_*.dump s3://backups/",
]
description = "Backup the database and upload to S3"
no_tty = true
tags = ["backup", "ops"]

[procedures.health-check]
session = "prod"
commands = [
    "systemctl is-active myapp",
    "curl -sf http://localhost:8080/health",
    "df -h /",
]
description = "Quick health check on production"
fail_fast = false
tags = ["monitoring"]
```

See also [`examples/procedures.toml`](../examples/procedures.toml).

## Commands

### List procedures

```bash
russh proc list
```

Shows a table with NAME, SESSION, DESCRIPTION, and TAGS columns.

### Show procedure details

```bash
russh proc show deploy
```

Displays the procedure's session, commands, description, TTY/fail_fast settings, and tags.

### Run a procedure

```bash
russh proc run deploy
```

Resolves the procedure's session, validates it, builds the SSH command, and executes it. The SSH command is printed to stderr before running.

**Options**:

```bash
# Log output to a file
russh proc run deploy --log deploy.log

# Disable TTY (overrides procedure setting)
russh proc run deploy -T

# Run a local script on a remote host instead
russh proc run deploy --from-script ./install.sh --session prod
```

In `--from-script` mode, the local file is piped to `bash` on the remote host via SSH stdin. The `--session` flag is required in this mode.

### Validate procedures

```bash
russh proc check
```

Exit codes: `0` = no issues, `1` = warnings only, `2` = errors found.

### Add a procedure

```bash
russh proc insert deploy --session prod \
  -c "systemctl stop app" \
  -c "rsync -avz /build/ /opt/app/" \
  -c "systemctl start app" \
  --description "Deploy app to production"
```

Appends the procedure to procedures.toml. The name `NONE` is reserved.

Optional flags: `--no-tty`, `--no-fail-fast`.

### Edit procedures

```bash
russh proc edit
```

Opens procedures.toml in `$VISUAL` or `$EDITOR`.

### Export procedures

```bash
# Export all as TOML
russh proc export

# Export one as TOML
russh proc export deploy

# Export as standalone shell script
russh proc export deploy --script
```

The `--script` flag generates a self-contained shell script with the SSH commands embedded. Requires a procedure name.

## Validation rules

Run `russh proc check` to validate. The following rules are checked:

| Severity | Code | Condition |
|----------|------|-----------|
| ERROR | `empty-session` | `session` field is empty |
| ERROR | `unknown-session` | `session` references a name not in config.toml |
| ERROR | `empty-commands` | `commands` array is empty |
| WARNING | — | An individual command string is empty |
