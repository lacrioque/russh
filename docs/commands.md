# Commands

## Global flags

| Flag | Description |
|------|-------------|
| `--config <PATH>` | Override the default config file location |

## russh (no subcommand)

```bash
russh
```

Opens an interactive TUI menu to select and connect to a session. Equivalent to `russh menu`.

## russh menu

```bash
russh menu
```

Interactive session picker. Search and select from your configured sessions, then connect.

## russh list

```bash
russh list
russh list --json
```

List all configured sessions in a table.

**Output columns**: NAME, HOST, USER, PORT, KEY, TAGS

Values shown are resolved (defaults applied).

**Options**:

| Flag | Description |
|------|-------------|
| `--json` | Output as a JSON array of resolved sessions |

With `--json`, the output is a JSON array where each element contains `name`, `host`, `username`, `port`, `ssh_key`, `key_source`, `display_target`, `tags`, and `jump_target`.

## russh show

```bash
russh show <session-name>
```

Display detailed information for a session, including both raw (as written in config) and resolved (with defaults applied) values.

**Output includes**: host, username, port, ssh_key, tags, jump host, resolved display target, resolved jump via target.

## russh connect

```bash
russh connect <session-name>
russh c <session-name>
```

**Alias**: `c`

Connect to a session by name. Resolves all defaults and jump hosts, validates the session, then execs SSH (replaces the current process).

If validation finds launch-blocking errors, the connection is refused with an error message.

## russh exec

```bash
russh exec <session-name> "<command>" [OPTIONS]
```

Execute a one-off command on a remote host. Unlike `connect`, this does not replace the current process — it runs the command, waits for it to finish, and returns.

**Arguments**:

| Argument | Description |
|----------|-------------|
| `session-name` | Name of the session to run on |
| `command` | Shell command to execute remotely |

**Options**:

| Flag | Description |
|------|-------------|
| `--json` | Capture output and return as structured JSON |
| `--to-std` | Capture output and write to stdout/stderr |
| `-T, --no-tty` | Disable pseudo-TTY allocation |

By default (no flags), the remote command inherits the terminal — output streams directly. With `--to-std`, output is captured and replayed cleanly. With `--json`, output is returned as a JSON object with `session`, `command`, `exit_code`, `stdout`, and `stderr` fields.

The process exit code always mirrors the remote command's exit code.

**Examples**:

```bash
# Basic — output streams to terminal
russh exec prod-web "systemctl status nginx"

# Captured — stdout/stderr written cleanly
russh exec prod-web "df -h /" --to-std

# JSON — structured output for scripting
russh exec prod-web "uptime" --json

# Non-interactive command
russh exec prod-db "pg_dump mydb | gzip > /tmp/backup.sql.gz" -T
```

**JSON output format**:

```json
{
  "session": "prod-web",
  "command": "uptime",
  "exit_code": 0,
  "stdout": " 14:32:01 up 42 days,  3:15,  1 user,  load average: 0.08, 0.12, 0.10\n",
  "stderr": ""
}
```

## russh copy

```bash
russh copy <source> <source-path> <dest> [dest-path] [OPTIONS]
```

Copy a file between two configured sessions via SCP.

**Arguments**:

| Argument | Description |
|----------|-------------|
| `source` | Source session name |
| `source-path` | Path on the source host |
| `dest` | Destination session name |
| `dest-path` | Path on the destination host (optional; defaults to `~`) |

**Options**:

| Flag | Description |
|------|-------------|
| `-n, --dry-run` | Show what would be done without executing |

**Jump host handling**:

- If source and destination share the same jump host (or both connect directly), a single `scp` command is used
- If they use **different** jump hosts, russh falls back to a two-step copy via a local temp file so each leg routes through the correct bastion

**Examples**:

```bash
# Direct copy, dest path defaults to ~
russh copy prod-web /var/log/app.log backup-host

# Explicit destination path
russh copy prod-db ~/dump.sql backup-host ~/archives/

# Preview
russh copy prod-web /etc/nginx/nginx.conf staging -n
```

## russh check

```bash
russh check
```

Validate all sessions and procedures, reporting any errors or warnings.

**Exit codes**:
- `0` — No issues found
- `1` — Warnings only
- `2` — At least one error (launch-blocking)

See [Configuration: Validation rules](configuration.md#validation-rules) for the full list of checks.

## russh insert

```bash
russh insert <name> <target> [OPTIONS]
russh i <name> <target> [OPTIONS]
```

**Alias**: `i`

Add a new session to the config file.

**Arguments**:

| Argument | Description |
|----------|-------------|
| `name` | Unique session identifier (cannot be `NONE`) |
| `target` | `user@host` or just `host` |

**Options**:

| Flag | Description |
|------|-------------|
| `-p, --port <PORT>` | SSH port |
| `-i, --identity <PATH>` | Path to SSH private key |
| `-J, --jump <SESSION_OR_HOST>` | Jump host session name or arbitrary host |

**Examples**:

```bash
# Minimal
russh insert myserver 192.168.1.10

# With user and port
russh insert prod ubuntu@prod.example.com -p 2222

# With key and jump host
russh i internal dbadmin@10.0.0.50 -p 5432 -i ~/.ssh/db_key -J bastion
```

After inserting, russh asks `Connect now? [Y/n]`.

## russh edit

```bash
russh edit [name] [OPTIONS]
russh e [name] [OPTIONS]
```

**Alias**: `e`

Edit a session's fields from the command line, or open the config file in your editor.

**With a session name** — update specific fields:

| Flag | Description |
|------|-------------|
| `--host <HOST>` | Update host address |
| `--user <USER>` | Update username (`NONE` to remove) |
| `-p, --port <PORT>` | Update port (`NONE` to remove) |
| `-i, --identity <PATH>` | Update SSH key (`NONE` to remove) |
| `-J, --jump <SESSION>` | Update jump host (`NONE` to remove) |

The special value `NONE` removes optional fields. It cannot be used with `--host` (host is required).

**Without a session name** — opens the config file in `$VISUAL` or `$EDITOR`:

```bash
russh edit
```

**Examples**:

```bash
# Change port
russh edit myserver -p 2222

# Update user and add jump host
russh edit myserver --user deploy -J bastion

# Remove jump host
russh edit myserver -J NONE

# Open config in editor
russh edit
```

## russh deploy

```bash
russh deploy <session>
russh deploy --all
russh deploy --tag <tag>
```

Deploy your local config file to remote hosts via SCP.

**Arguments/Options**:

| Flag | Description |
|------|-------------|
| `<session>` | Deploy to a single session |
| `--all` | Deploy to all configured sessions |
| `--tag <TAG>` | Deploy to sessions matching a tag |
| `-n, --dry-run` | Show what would be done without executing |

**Behavior**:
- Backs up the existing remote config with a timestamp suffix (e.g., `config.toml.bak.20260413120000`)
- Creates the remote directory (`~/.config/russh/`) if needed
- Copies local config to `~/.config/russh/config.toml` on the remote host
- Respects jump hosts when deploying to internal servers

**Examples**:

```bash
# Preview deployment to all hosts
russh deploy --all -n

# Deploy to all sessions tagged "prod"
russh deploy --tag prod

# Deploy to a single session
russh deploy myserver
```

## russh export

```bash
russh export
```

Print the raw contents of the config file to stdout.

## russh version

```bash
russh version
```

Print the version number and config file path.

**Output example**:

```
russh 1.1.0
config: /home/user/.config/russh/config.toml
```

## russh proc

Procedure commands for managing and running named command sequences. See [Procedures](procedures.md) for the full procedures reference.

**Global proc flag**:

| Flag | Description |
|------|-------------|
| `--from-config <PATH>` | Override the procedures.toml location |

### russh proc list

```bash
russh proc list
```

List all procedures in a table.

**Output columns**: NAME, SESSION, DESCRIPTION, TAGS

### russh proc show

```bash
russh proc show <name>
```

Display detailed information for a procedure, including session, commands, description, and flags.

### russh proc check

```bash
russh proc check
```

Validate all procedures. Exit codes match `russh check` (0, 1, or 2).

See [Procedures: Validation rules](procedures.md#validation-rules) for checks performed.

### russh proc run

```bash
russh proc run <name> [OPTIONS]
```

Execute a named procedure on its configured session.

| Flag | Description |
|------|-------------|
| `--log <PATH>` | Redirect stdout/stderr to a log file |
| `-T, --no-tty` | Disable pseudo-TTY allocation (overrides procedure setting) |
| `--from-script <PATH>` | Pipe a local script to the remote host instead |
| `--session <NAME>` | Session for script mode (required with `--from-script`) |

**Examples**:

```bash
# Run a procedure
russh proc run deploy

# Run with logging
russh proc run deploy --log deploy.log

# Run a local script on a remote host
russh proc run deploy --from-script ./install.sh --session prod

# Force no TTY
russh proc run backup -T
```

In script mode (`--from-script`), the local file is piped to `bash` on the remote host via SSH stdin.

### russh proc insert

```bash
russh proc insert <name> --session <SESSION> [OPTIONS]
```

Add a new procedure to procedures.toml.

| Flag | Description |
|------|-------------|
| `--session <SESSION>` | Session to execute on (required) |
| `-c, --command <CMD>` | Command to execute (repeatable) |
| `--description <TEXT>` | Human-readable description |
| `--no-tty` | Disable TTY allocation |
| `--no-fail-fast` | Don't stop on first failure |

**Example**:

```bash
russh proc insert deploy --session prod \
  -c "systemctl stop app" \
  -c "rsync -avz /build/ /opt/app/" \
  -c "systemctl start app" \
  --description "Deploy app to production"
```

### russh proc edit

```bash
russh proc edit
```

Open procedures.toml in `$VISUAL` or `$EDITOR`.

### russh proc export

```bash
russh proc export [name] [--script]
```

Export procedures as TOML or as a standalone shell script.

| Flag | Description |
|------|-------------|
| `name` | Export a single procedure (optional; omit for all) |
| `--script` | Emit a shell script instead of TOML (requires `name`) |

**Examples**:

```bash
# Export all procedures as TOML
russh proc export

# Export one procedure as TOML
russh proc export deploy

# Export as a standalone shell script
russh proc export deploy --script
```
