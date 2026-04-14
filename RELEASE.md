# Release v1.0.0

**russh** — a Rust SSH toolkit for managing and connecting to remote hosts.

## Highlights

This is the first stable release of russh, marking the project's graduation from pre-1.0 development. It includes a complete session management system, a new procedures engine, and full CI/CD for cross-compiled releases.

### Procedures Engine (new)

Define reusable scripts and commands in `procedures.toml` and execute them on any configured host:

```bash
russh proc run deploy-app production
russh proc list
russh proc show deploy-app
russh proc check
```

Full CRUD support with `proc insert`, `proc edit`, and `proc export`.

### Session Management

- **list** — tabular overview of all configured sessions
- **show** — detailed raw/resolved view of a session
- **check** — validate all sessions with structured error reporting
- **connect** — SSH into a session with validation
- **menu** — interactive fuzzy-select picker (default)
- **insert** — create sessions from CLI (`russh i myhost admin@10.0.0.1 -J bastion`)
- **edit** — modify existing sessions
- **export** — print current config
- **deploy** — push config to remote hosts via SCP

### Jump Host Support

Chain connections through bastion hosts with `-J` or the `jump` config field. Supports both session name references and arbitrary `user@host:port` specs.

### Config & UX

- TOML-based config at `~/.config/russh/config.toml` (XDG-aware)
- Automatic prompt to create config when missing
- `russh version` shows version and config path

## What's Changed Since v0.2.1

- Procedures system (data model, resolution, validation, CLI commands)
- `edit` command for modifying sessions
- `export` command for printing config
- Arbitrary host specs for jump host `-J`
- GPLv3 license
- GitHub Actions CI (lint + release workflows)
- User-facing documentation wiki

## Installation

### From Release Binaries

Download the appropriate binary for your platform from the [release assets](https://github.com/lacrioque/russh/releases/tag/v1.0.0).

| Platform | Asset |
|----------|-------|
| Linux x86_64 | `russh-x86_64-unknown-linux-gnu.tar.gz` |
| Linux aarch64 | `russh-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `russh-x86_64-apple-darwin.tar.gz` |
| macOS aarch64 | `russh-aarch64-apple-darwin.tar.gz` |

### From Source

```bash
git clone https://github.com/lacrioque/russh.git
cd russh
cargo build --release
# Binary at target/release/russh
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete history across all versions.
