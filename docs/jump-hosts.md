# Jump Hosts

Jump hosts (also called bastion hosts) let you reach internal servers that aren't directly accessible from your machine. russh supports this via SSH's ProxyJump (`-J`) mechanism.

## Configuration

Add a `jump` field to any session:

```toml
# The bastion/jump host itself
[sessions.bastion]
host = "bastion.example.com"
username = "ops"
port = 2222
tags = ["infra", "bastion"]

# Internal server, accessible only through the bastion
[sessions.internal-db]
host = "10.0.0.50"
username = "dbadmin"
port = 5432
jump = "bastion"
tags = ["internal", "database"]
```

Connecting to `internal-db` automatically routes through `bastion`:

```bash
russh connect internal-db
# Executes: ssh -J ops@bastion.example.com:2222 -p 5432 dbadmin@10.0.0.50
```

## Session name vs. arbitrary host

The `jump` field accepts two formats:

### Session name

```toml
jump = "bastion"
```

References another session defined in your config. russh resolves the session's host, username, and port into a `user@host:port` spec for the `-J` flag.

This is the recommended approach — it keeps jump host details in one place and reuses your existing session configuration.

### Arbitrary host spec

```toml
jump = "ops@jumpbox.example.com:2222"
```

If the value doesn't match any session name, it is passed directly to `ssh -J`. This is useful for jump hosts that you don't want to manage as a named session.

Supported formats:
- `host`
- `user@host`
- `host:port`
- `user@host:port`

## How it works

When you connect to a session with a `jump` field:

1. russh resolves the target session (host, user, port, key)
2. russh resolves the jump host:
   - If `jump` matches a session name: resolves that session's details
   - Otherwise: passes the value through as-is
3. The resolved jump spec is added as `ssh -J <jump-spec>` in the SSH command

## Deploy through jump hosts

`russh deploy` respects jump hosts. When deploying config to an internal server, SCP uses the `-J` flag automatically:

```bash
# Deploys config to internal-db through bastion
russh deploy internal-db
```

## Procedures through jump hosts

Procedures inherit the session's jump host configuration. No extra setup is needed:

```toml
[procedures.db-backup]
session = "internal-db"    # This session uses jump = "bastion"
commands = ["pg_dump -Fc mydb > /tmp/backup.dump"]
```

```bash
russh proc run db-backup
# SSH connection routes through bastion automatically
```

## Validation

`russh check` detects these jump host issues:

| Severity | Code | Condition |
|----------|------|-----------|
| ERROR | `empty-jump-host` | `jump` is set but empty string |
| ERROR | `circular-jump` | Session references itself as its own jump host |

Note: multi-hop loops (A jumps through B, B jumps through A) are not currently detected.

## Example: multi-tier access

```toml
# External bastion
[sessions.bastion]
host = "bastion.example.com"
username = "ops"
port = 2222

# Application server behind bastion
[sessions.app]
host = "10.0.0.10"
username = "deploy"
jump = "bastion"
tags = ["app"]

# Database server behind bastion
[sessions.db]
host = "10.0.0.20"
username = "dbadmin"
port = 5432
jump = "bastion"
tags = ["database"]
```

```bash
russh connect app    # Routes through bastion
russh connect db     # Routes through bastion
russh deploy --all   # Deploys to all three, routing through bastion as needed
```
