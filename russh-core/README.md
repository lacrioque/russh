# russh-core

Core library for [russh](https://github.com/lacrioque/russh), a Rust SSH toolkit for managing named sessions and running procedures on remote hosts.

This crate provides the building blocks for SSH session and procedure management. It handles config parsing, path resolution, validation, and SSH command construction — everything except the CLI frontend.

## Modules

| Module | Purpose |
|--------|---------|
| `config` | Load and parse TOML session configs with structured errors |
| `model` | Domain types: `Session`, `ResolvedSession`, `Procedure`, `ValidationIssue` |
| `paths` | Config path resolution with XDG and tilde expansion |
| `resolve` | Apply defaults, expand paths, resolve jump hosts |
| `validate` | Validate sessions and procedures, report issues by severity |
| `ssh` | Build SSH/SCP command args, exec or spawn processes |
| `proc_config` | Load and parse procedure definitions from TOML |
| `proc_run` | Resolve procedures against sessions, build execution commands |
| `sync` | Deploy configs to remote hosts via SCP |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
russh-core = "1"
```

### Load and resolve sessions

```rust
use russh_core::config::load_config;
use russh_core::paths::config_path;
use russh_core::resolve::resolve_session;

let path = config_path(None);
let sessions = load_config(&path).unwrap();

for session in &sessions {
    let resolved = resolve_session(session);
    println!("{}: {}@{}:{}", resolved.name, resolved.username, resolved.host, resolved.port);
}
```

### Validate sessions

```rust
use russh_core::validate::validate_sessions;

let issues = validate_sessions(&resolved_sessions);
for issue in &issues {
    println!("{}", issue); // e.g. "[error] dev-server: host is empty (E001)"
}
```

### Build SSH commands

```rust
use russh_core::ssh::build_command;

let cmd = build_command(&resolved_session);
println!("ssh {}", cmd.display); // e.g. "ssh -p 2222 -i ~/.ssh/id_ed25519 deploy@10.0.0.50"
```

## Config format

Sessions are defined in TOML under `[sessions.<name>]`:

```toml
[sessions.dev-server]
host = "10.0.0.50"
username = "deploy"
port = 2222
ssh_key = "~/.ssh/id_ed25519"
tags = ["dev", "linux"]
jump = "bastion"
```

Only `host` is required. See the [main project README](https://github.com/lacrioque/russh#configuration) for full details.

## License

GPL-3.0-only
