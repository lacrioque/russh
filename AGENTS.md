# Russh — AI Agent Usage Guide

Russh is an SSH session manager. It stores named SSH sessions in a TOML config
and delegates to the system `ssh` binary. You can use it to run commands on
remote hosts without needing raw SSH credentials or connection details.

## When to use russh

Use russh when you need to:
- Run a command on a remote host the user has configured
- Check service status, logs, disk usage, or process state on remote machines
- Verify deployments or configuration on remote systems
- List what hosts are available before suggesting SSH operations

Do NOT use russh to:
- Transfer files (use `scp` or `rsync` directly if needed)
- Modify SSH keys or credentials
- Change the russh config without the user asking

## Available commands

### List sessions (safe, read-only)
```bash
russh list
```
Shows all configured SSH sessions with host, user, port, and tags.
Use this first to discover what hosts are available.

### Show session details (safe, read-only)
```bash
russh show <session-name>
```
Shows raw config and resolved values (with defaults applied) for one session.
Use this to understand connection parameters before connecting.

### Validate config (safe, read-only)
```bash
russh check
```
Reports errors and warnings in the config. Run this if connections fail
or after config changes.

### Run a command on a remote host
```bash
russh connect <session-name>
```
Opens an interactive SSH session. This is interactive and will block.

To run a **non-interactive command** on a remote host, use SSH directly
with the session details from `russh show`:
```bash
ssh -p <port> -i <key> <user>@<host> '<command>'
```

Or construct it from `russh show` output. Example workflow:
```bash
# 1. Find the host
russh list

# 2. Get connection details
russh show dev-server

# 3. Run a remote command using those details
ssh -p 2222 -i ~/.ssh/id_ed25519 deploy@10.0.0.50 'systemctl status nginx'
```

### Add a session
```bash
russh insert <name> <user@host> [-p port] [-i keyfile] [-J jump-host]
```
Only use when the user explicitly asks to add a new session.

## Config location

Default: `~/.config/russh/config.toml`

Override: `russh --config /path/to/config.toml <command>`

## Permission boundaries

| Action | Permission level |
|--------|-----------------|
| `russh list` | Always safe — read-only, local |
| `russh show <name>` | Always safe — read-only, local |
| `russh check` | Always safe — read-only, local |
| `russh connect <name>` | Requires user approval — opens SSH session |
| `ssh ... '<command>'` | Requires user approval — executes on remote host |
| `russh insert ...` | Requires user approval — modifies config file |

**Rule**: `list`, `show`, and `check` are free to run without asking.
Anything that connects to a remote host or modifies config — ask first.

## Typical agent workflows

### Check if a service is running
```bash
russh show prod-web          # get connection details
# Then ask user: "Can I run 'systemctl status nginx' on prod-web?"
ssh -p 22 -i ~/.ssh/prod_key ubuntu@prod.example.com 'systemctl status nginx'
```

### Check disk space across hosts
```bash
russh list                   # discover hosts
# For each relevant host:
russh show <name>            # get details
ssh <user>@<host> 'df -h'   # run with user approval
```

### Verify a deployment
```bash
russh show staging
ssh admin@staging.example.com 'cat /etc/app/version.txt && systemctl is-active app'
```

### Debug connectivity
```bash
russh check                  # validate config first
russh show <name>            # inspect resolved session
# Then try: ssh -v <user>@<host> to debug
```

## Tags

Sessions can have tags (e.g., `prod`, `dev`, `database`). Use `russh list`
output to filter by tag when deciding which hosts are relevant to a task.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
