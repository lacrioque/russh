
```markdown
# Russh v0.1 Definition of Done

## Version Scope

This Definition of Done applies to **Russh v0.1**, the first usable CLI release.

The purpose of v0.1 is to deliver a stable foundation that allows users to:

- define SSH sessions in TOML
- list and inspect them
- validate them
- select them from a simple menu
- connect through the native system `ssh`

---

## Release Goal

Russh v0.1 is done when a user can install it, define a small set of SSH targets in config, validate them, inspect them, and launch one from either a command or an interactive menu without needing any desktop UI.

---

## In-Scope Features

The following features must be complete for v0.1:

- TOML config file support
- session model with `name`, `host`, `username`, `ssh_key`, `port`, and `tags`
- session listing
- session inspection
- session validation
- native SSH launch
- interactive menu
- no-subcommand default to menu
- shell-style `~` expansion for `ssh_key`
- hostname support
- warning for hostname usage during `check`
- support for defaults:
  - current user if `username` is empty
  - port `22` if `port` is empty
  - system SSH behavior if `ssh_key` is empty

---

## Out-of-Scope for v0.1

The following must not be required for release:

- desktop app
- session persistence
- terminal multiplexing
- embedded terminal
- SSH config import
- config editing commands
- JSON output
- dry-run mode
- add/remove/update commands
- advanced SSH argument passthrough
- secret or passphrase storage

---

## Functional Definition of Done

### 1. Config loading
Done when:

- Russh can load a TOML config from the default config path
- Russh can fail gracefully with a clear error if the config is missing or malformed
- session blocks under `[sessions.<name>]` are parsed correctly
- tags parse correctly as an optional string array

### 2. Default resolution
Done when:

- missing `username` resolves to the current OS user
- missing `port` resolves to `22`
- missing `ssh_key` results in no `-i` flag being passed
- `ssh_key` values containing `~` are expanded before validation and connection

### 3. Session listing
Done when:

- `russh list` prints all configured sessions
- output includes at least: name, host, resolved user, resolved port, key summary
- tags are shown in output
- `list` does not emit non-fatal warnings such as hostname preference warnings

### 4. Session inspection
Done when:

- `russh show <name>` prints details for one session
- unknown session names result in a clear error
- output reflects resolved values, not only raw config data

### 5. Validation
Done when:

- `russh check` validates all sessions without opening any connection
- invalid config returns a non-zero exit code
- missing required fields are reported as errors
- invalid port values are reported as errors
- configured key paths that do not exist are reported as warnings or launch-blocking errors as appropriate
- hostname usage produces a warning, not an error
- warnings are shown only in `check`

### 6. Connection
Done when:

- `russh connect <name>` launches the native system `ssh`
- `russh c <name>` behaves identically
- the generated SSH arguments reflect resolved values
- `ssh_key` is passed via `-i` only when explicitly configured
- launch-blocking errors stop execution with a clear message
- Russh does not reimplement SSH behavior

### 7. Interactive menu
Done when:

- `russh menu` opens an interactive session picker
- the picker is keyboard usable
- the picker allows selection of a stored session
- selecting a session launches the same connect flow as `russh c <name>`
- invoking `russh` with no subcommand opens this menu by default

---

## UX Definition of Done

Done when:

- command names are consistent and documented
- output is readable in a standard terminal
- errors identify the session and field when possible
- warnings are understandable without reading source code
- the CLI feels predictable and quiet
- the menu is simple and unopinionated, appropriate for an admin tool

---

## Technical Definition of Done

### Core architecture
Done when:

- config parsing, validation, resolution, and SSH command construction live in shared core logic
- CLI-specific parsing and rendering are separate from core logic
- no business-critical logic is duplicated across commands

### Path handling
Done when:

- config path resolution works for supported v0.1 platforms
- `ssh_key` home expansion is implemented centrally
- expanded paths are used consistently in validation and launch

### Process execution
Done when:

- Russh uses the system `ssh` executable
- SSH is launched using process APIs appropriate to the platform
- failure to find or execute `ssh` is reported clearly

---

## Quality Definition of Done

### Tests
Done when the following test coverage exists at minimum:

#### Unit tests
- valid config parsing
- malformed config failure
- required field validation
- username default resolution
- port default resolution
- `~` expansion for key paths
- hostname warning generation
- SSH command argument generation

#### Integration tests
- `russh list` with fixture config
- `russh show <name>`
- `russh check` exit code behavior
- `russh c <name>` command preparation path
- no-subcommand menu entry path

### Manual verification
Done when the following are manually verified on target development platform(s):

- create a config from scratch
- run `russh check`
- run `russh list`
- run `russh show <name>`
- run `russh c <name>`
- run `russh` and select a session from the menu

---

## Documentation Definition of Done

Done when:

- the config format is documented
- each v0.1 command is documented
- defaults are documented
- warning behavior is documented
- hostname support and hostname warning policy are documented
- at least one example config file is included
- installation/build instructions exist for developers

---

## Packaging Definition of Done

Done when:

- the project builds cleanly from a fresh checkout
- there is a reproducible release build process
- the produced binary can run without requiring the desktop app
- version `0.1.0` is stamped consistently in package metadata and CLI version output

---

## Acceptance Criteria

Russh v0.1 is accepted when all of the following are true:

1. A user can define at least three sessions in TOML and Russh reads them successfully.
2. `russh list` shows all sessions with resolved values.
3. `russh show <name>` displays one named session correctly.
4. `russh check` reports:
   - errors for invalid config
   - warnings for hostname use
   - warnings for problematic key paths
5. `russh c <name>` launches the native SSH client with the correct arguments.
6. `russh` with no subcommand opens the interactive menu.
7. Selecting a session from the menu launches the same connect flow as `russh c <name>`.
8. Non-fatal warnings are limited to `check`.
9. The codebase is structured so the core can be reused later by a desktop frontend.
10. Tests and documentation exist at the level defined above.

---

## Release Checklist

- [ ] Core config parser implemented
- [ ] Session model implemented
- [ ] Default resolution implemented
- [ ] Home-path expansion implemented
- [ ] Validation implemented
- [ ] Hostname warning policy implemented
- [ ] SSH command builder implemented
- [ ] Native SSH execution implemented
- [ ] `list` command implemented
- [ ] `show` command implemented
- [ ] `connect` / `c` commands implemented
- [ ] `check` command implemented
- [ ] `menu` command implemented
- [ ] no-subcommand default to menu implemented
- [ ] unit tests passing
- [ ] integration tests passing
- [ ] example config included
- [ ] CLI help text complete
- [ ] architecture and usage docs updated
- [ ] release binary builds successfully

---

## Final Release Standard

Russh v0.1 is done when it is not merely feature-complete, but trustworthy:

- simple enough to understand
- sharp enough to be useful
- stable enough to become the root of future versions
- disciplined enough that the desktop application can one day grow from it without bending the trunk