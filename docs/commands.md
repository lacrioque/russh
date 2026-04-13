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
```

List all configured sessions in a table.

**Output columns**: NAME, HOST, USER, PORT, KEY, TAGS

Values shown are resolved (defaults applied).

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
russh 0.2.0
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
