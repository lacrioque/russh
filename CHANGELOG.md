# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-04-14

### Added

- GitHub Actions test workflow for workspace unit and integration tests
- GitHub Actions publish workflow for automated crates.io releases on tags
- crates.io metadata (`description`, `repository`) for both crates

## [1.0.0] - 2026-04-14

### Added

- **Procedures system**: define named procedures (scripts/commands) in `procedures.toml` and execute them on remote hosts via SSH
  - `proc run` — execute a named procedure on a target session
  - `proc list` / `proc show` / `proc check` — manage and validate procedures
  - `proc insert` / `proc edit` / `proc export` — create, modify, and export procedure definitions
  - Procedure resolution and validation with structured error reporting
  - Procedure/script command builders and `spawn_ssh` execution
- `edit` command to modify existing sessions from the CLI
- `export` command to print the current config file contents
- Accept arbitrary `user@host:port` specs for jump host `-J` flag (not just session names)
- GPLv3 license for public release
- User-facing documentation wiki
- GitHub Actions lint workflow (rustfmt + clippy)
- GitHub Actions release workflow for cross-compiled binaries (Linux x86_64, aarch64, macOS)

### Fixed

- CI test failures: skip config creation prompt when stdin is not a TTY

## [0.2.1] - 2026-03-31

### Added

- `deploy` command for config sync via SCP — push `config.toml` to remote hosts with dry-run, backup, and tag filtering
- AGENTS.md for AI code companion integration

### Fixed

- Serialize env-mutating tests with mutex to prevent race conditions

## [0.2.0] - 2026-03-31

### Added

- `version` subcommand showing version and config path
- Prompt to create config file when missing (replaces hard error)
- Cross-platform Makefile (macOS + Linux/WSL)

## [0.1.2] - 2026-03-28

### Added

- `insert` (`i`) command to create sessions from CLI — parses `user@host`, supports `-p` (port) and `-i` (key) flags, prompts to connect after insert
- Jump host (ProxyJump) support — sessions can define a `jump` field referencing another session, resolved as `-J user@host:port`

### Fixed

- CI: correct binary name in build artifact copy
- CI: rename artifact from `russh-cli` to `russh`

## [0.1.0] - 2026-03-21

### Added

- Cargo workspace with `russh-core` (library) and `russh-cli` (binary)
- **Core library** (`russh-core`):
  - Domain model: `Session`, `ResolvedSession`, `ValidationIssue`, `KeySource`, `Severity`
  - Config module: TOML loading and parsing with structured errors
  - Paths module: config path resolution, XDG support, tilde expansion
  - Resolve module: default value resolution for sessions
  - Validate module: session validation (empty host, port, missing key, hostname warnings)
  - SSH module: command builder and exec launcher
- **CLI** (`russh-cli`):
  - `list` — tabular session listing
  - `show` — raw/resolved detail view of a session
  - `check` — validate all sessions with exit codes (0/1/2)
  - `connect` — session lookup, validation, and SSH exec
  - `menu` — interactive fuzzy-select session picker (default when no subcommand)
  - `SessionPicker` trait with `InquirePicker` backend
- Unit tests for all `russh-core` modules (83 tests)
- Integration tests for `russh-cli` (17 tests)
- Release profile with `opt-level=z`, LTO, and strip
- GitLab CI pipeline (lint, test, build, release)
- README and example config

## [0.0.1] - 2026-03-21

### Added

- Initial project scaffold
- rustfmt + clippy clean baseline

[1.0.1]: https://github.com/lacrioque/russh/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/lacrioque/russh/compare/v0.2.1...v1.0.0
[0.2.1]: https://github.com/lacrioque/russh/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/lacrioque/russh/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/lacrioque/russh/compare/v0.0.1...v0.1.2
[0.1.0]: https://github.com/lacrioque/russh/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/lacrioque/russh/releases/tag/v0.0.1
