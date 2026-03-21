# Russh Architecture Document

## Document Status

Draft v0.2  
Updated after architecture review and product decisions.

---

## Overview

**Russh** is a lightweight SSH session manager that starts as a CLI-first tool and later expands into a desktop application. Its primary purpose is to help users **store, inspect, validate, and launch SSH targets** through a simple, transparent configuration model.

Russh is not intended to replace SSH itself, keep terminal sessions alive, or manage multiplexed shells. It is a thin operational layer above the native system `ssh` command.

The guiding principle is simple:

> Remember connections well, show them clearly, launch them fast.

---

## Goals

### Primary Goals

- Provide a fast CLI for listing and launching saved SSH targets
- Store session definitions in a human-readable TOML config
- Resolve sensible defaults automatically
- Use the native `ssh` binary for all actual connections
- Keep the UX minimal, scriptable, and predictable
- Build a reusable core that can later support a desktop frontend

### Secondary Goals

- Offer interactive selection for quick connection flow
- Validate configuration and surface errors clearly
- Support future expansion into richer configuration management
- Keep all SSH-related configuration discoverable and manageable from one place
- Remain portable across Linux, macOS, and later Windows

---

## Non-Goals

The following are explicitly out of scope for the first version:

- Keeping SSH sessions alive
- Multiplexing terminal sessions
- Embedding a terminal emulator
- Managing SSH agents
- Generating SSH keys
- Storing passphrases or secrets
- Monitoring remote hosts
- Replacing OpenSSH configuration entirely

---

## Product Scope

Russh is best understood as a **session registry and launcher**.

A **session** in Russh is a saved SSH target definition. It represents connection metadata, not an active connection.

Core user actions:

- list configured sessions
- inspect a session
- validate configuration
- choose a session interactively
- connect using the native SSH client

---

## High-Level Architecture

Russh is divided into three logical layers:

1. **Core Library**
2. **CLI Application**
3. **Desktop Application** (future)

```text
+----------------------+
|   Desktop Frontend   |   (future)
+----------+-----------+
           |
           v
+----------------------+
|    Application API   |   (shared use cases)
+----------+-----------+
           |
           v
+----------------------+
|      Russh Core      |
|----------------------|
| Config parsing       |
| Validation           |
| Default resolution   |
| Session modeling     |
| SSH command building |
| Import support       |
+----------+-----------+
           |
           v
+----------------------+
| Native System SSH    |
| /usr/bin/ssh, etc.   |
+----------------------+

```

The CLI and future desktop app should both rely on the same core behavior to prevent divergence in logic.

## Architectural Principles

1. Native-first

Russh should never reimplement SSH transport. It must invoke the system's installed ssh binary.

2. Transparent configuration

The source of truth is a TOML file the user can read and edit directly.

3. Thin abstraction

Russh should simplify common workflows without hiding what it is doing.

4. Predictable output

Errors, listings, and resolved values should be explicit and stable.

5. Shared business logic

All parsing, validation, and command construction logic should live in the core library.

6. Desktop as a frontend, not a fork

The desktop application should consume shared use cases instead of duplicating rules.

7. One SSH home, without hard-coding one layout

Russh should make it easy to keep SSH-related material together, while allowing users to configure paths and import sources explicitly.

## System Context

### External Dependencies

Russh depends on:

- the native ssh executable available on the system
- the user filesystem for config and key path resolution
- the local OS user account for default username resolution
- optionally user-defined SSH-related directories or import locations

### Trust Boundaries

Russh trusts:

- user-supplied configuration
- local file paths
- native OS command execution

Russh does not trust:

- missing or malformed config
- invalid host definitions
- unreadable SSH key paths
- unsafe assumptions about user environment without validation

## Core Domain Model

### Session

A session is the central domain entity.

### Fields

|Field|	Type|	Required|	Default|	Description|
|-----|-----|-----------|----------|---------------|
|name|	string|	yes|	none|	Unique identifier for the session|
|host|	string|	yes|	none|	IP address or hostname|
|username|	string|	no|	current| OS user	SSH username|
|ssh_key|	string|	no|	system| SSH default	Path to identity file|
|port|	int|	no|	22|	SSH port|
|tags|	array|	no|	empty|	Optional grouping and filtering labels|

### ResolvedSession

A ResolvedSession is derived from a Session plus runtime defaults.

Resolved fields
|Field|	Description|
|-----|------------|
|name|	session name|
|host|	required host|
|username|	configured user or current OS user|
|port|	configured port or 22|
|ssh_key|	optional normalized key path if configured|
|key_source|	explicit or system_default|
|display_target|	computed string like user@host:22|
|tags|	optional list of grouping labels|

### ValidationIssue

Represents a warning or error encountered while loading or resolving config.

Suggested fields

- severity: error or warning
- session_name: optional
- field: optional
- message: human-readable explanation
- code: optional stable identifier for testing and machine-readable output later

## Configuration Design

### Config Format

Russh uses TOML for readability and ease of manual editing.

Recommended structure

```toml
[sessions.web1]
host = "192.168.1.10"
username = "root"
ssh_key = "~/.ssh/id_ed25519"
port = 22
tags = ["prod", "web"]

[sessions.db-prod]
host = "db.example.internal"
username = "admin"
ssh_key = "~/.ssh/prod_db"
tags = ["prod", "database"]

[sessions.cache]
host = "cache.internal"
tags = ["staging"]
```

This structure is preferred because:

- session names are naturally unique keys
- lookup by session name is straightforward
- the config remains compact and readable
- tags fit naturally into each session block

### Config Path

Default config location:

    Linux/macOS: ~/.config/russh/config.toml

Future Windows path:

    %APPDATA%/russh/config.toml or equivalent platform-appropriate location

The CLI should also support an override:

```bash
russh --config /custom/path/config.toml list
```

### Import and SSH Path Strategy

Russh should support the idea that users may want SSH-related configuration in one place, while recognizing that the native ~/.ssh directory may be permission-sensitive and already managed externally.

### Initial position

- Russh has its own config file under its own config directory
- Russh can reference SSH keys stored anywhere on disk
- Russh may later import from OpenSSH-related locations
- Import sources must be configurable rather than assumed
- Future import direction

Possible future configurable import settings:

´´´toml
[import]
enabled = true
sources = [
  "~/.ssh/config",
  "~/.config/russh/import/extra_hosts.conf"
]
´´´

Import should remain a convenience feature, not the primary configuration format.

### Configuration Rules

#### Required fields

- name is required implicitly through the TOML table key
- host is required

#### Optional fields

- username
- ssh_key
- port
- tags

#### Defaults

- missing username -> current OS user
- missing port -> 22
- missing ssh_key -> no -i flag, let SSH use system defaults
- missing tags -> empty list

### Path resolution

ssh_key paths should support shell-style home expansion
~/.ssh/id_ed25519 must be expanded to the user's home directory before validation and command construction
internal representation should use normalized absolute or canonicalized paths where practical

### Host support rules

Russh should support:

- IPv4
- IPv6 where feasible in parsing and validation
- DNS hostnames
- internal hostnames

### Host validation rules

- hostname values are valid inputs
- IP addresses are preferred
- if a hostname is used instead of an IP address, Russh should issue a warning during check only
- this warning should not block listing or connecting

    Example warning:

    warning[prefer-ip]: session "db-prod" uses hostname "db.example.internal"; an IP address is preferred for operational clarity


### Validation rules

- Errors
- missing host
- invalid port range
- malformed config file
- duplicate session names if an alternate format is ever supported
- invalid tags type
- unexpandable user-home path where required for launch

### Warnings

- ssh_key path does not exist
- ssh_key path is unreadable
- hostname used instead of IP address
- suspicious but technically valid host value

## Command-Line Interface Architecture

The CLI is the first production frontend. It should remain small and focused.

### Primary Commands

´´´russh´´´

With no subcommand, Russh should open the interactive menu by default.


´´´russh list´´´

Lists all configured sessions with resolved values.


´´´russh show <name>´´´

Displays detailed information for one session.


´´´russh connect <name>´´´

Launches native SSH for the specified session.

Alias: ´´´russh c <name>´´´


´´´russh menu´´´

Shows an interactive session picker.


´´´russh check´´´

Validates configuration and reports warnings/errors.


´´´russh config path´´´


Prints the resolved config path.

´´´russh config edit´´´

Opens the config file in the user's preferred editor.


### CLI Behavior

´´´russh´´´

If no subcommand is given:

- load config
- resolve sessions
- launch interactive menu
- connect on selection

This makes the tool feel immediate and efficient.


´´´list´´´

Should show resolved data, not only raw config values.

Example:

NAME      HOST                USER     PORT  KEY                  TAGS
web1      192.168.1.10        root     22    ~/.ssh/id_ed25519   prod,web
db-prod   db.example.internal admin    22    ~/.ssh/prod_db      prod,database
cache     cache.internal      markus   22    system default      staging

list should not emit warnings. Warnings belong to check.


´´´show´´´

Should display raw and resolved details when useful.


´´´connect´´´

Should:

- load config
- locate session by name
- resolve defaults
- validate launch-relevant fields
- build SSH command arguments
- execute native ssh


´´´menu´´´

Should provide:

- searchable list of session names
- compact target summary
- keyboard-first selection
- direct handoff to connect


´´´check´´´

Should be safe and non-invasive. It must not open connections.

It is the only command that should surface non-fatal warnings such as:

- hostname preferred over IP
- missing configured key file
- unreadable key path

### SSH Launching Design

Russh does not implement SSH. It builds arguments and delegates to the system executable.

Command construction rules

Given:

´´´toml
[sessions.web1]
host = "192.168.1.10"
username = "root"
ssh_key = "~/.ssh/id_ed25519"
port = 22
´´´

Russh should construct:

´´´ssh -i /home/user/.ssh/id_ed25519 -p 22 root@192.168.1.10´´´

If ssh_key is omitted:

´´´ssh -p 22 markus@cache.internal´´´

### Argument mapping

|Session field	|SSH mapping|
|-|-|
|username + host	|user@host|
|port	|-p <port>|
|ssh_key	|-i <expanded-path>|

### Execution behavior

The CLI should replace or spawn a child process that attaches to the native terminal session, preserving expected SSH behavior.

## Internal Module Design

A clean modular structure is essential for CLI and desktop reuse.

Proposed packages

### russh-core

Contains domain and application logic.

Suggested modules:

- config
- model
- resolve
- validate
- ssh
- errors
- paths
- import

### russh-cli

Contains CLI argument parsing and command orchestration.

Suggested modules:

- main
- commands/list
- commands/show
- commands/connect
- commands/check
- commands/menu
- output

### russh-desktop (future)

Contains desktop-specific UI and platform integration.

Suggested modules:

- ui/session_list
- ui/session_editor
- ui/validation_panel
- ui/connect_action
- ui/tag_filter

## Core Library Responsibilities

### Config module

Responsibilities:

- load TOML from file
- deserialize session definitions
- normalize structures
- return domain models

### Model module

Responsibilities:

- define Session
- define ResolvedSession
- define validation and error types

### Resolve module

Responsibilities:

- fill default values
- expand user-relative paths like ~/.ssh/...
- resolve current OS username
- normalize tag values
- produce ResolvedSession

### Validate module

Responsibilities:

- static validation of config structure
- semantic validation of fields
- filesystem checks for SSH key paths
- host preference warnings
- return warnings/errors without side effects

### SSH module

Responsibilities:

- convert ResolvedSession into native SSH arguments
- expose dry-run command rendering
- execute the command through system process APIs

### Paths module

Responsibilities:

- determine config location
- support platform-specific path rules
- resolve CLI overrides
- expand home-directory paths

### Import module

Responsibilities:

- define import source abstraction
- parse supported import sources in future versions
- transform imported host definitions into Russh sessions
- preserve explicit user control over import paths

Application Flow
list flow
CLI command received
    -> determine config path
    -> load config
    -> parse sessions
    -> resolve defaults
    -> format output table
    -> print
connect flow
CLI command received
    -> determine config path
    -> load config
    -> find session by name
    -> resolve defaults
    -> validate launch-critical fields
    -> build SSH arguments
    -> execute native ssh
check flow
CLI command received
    -> determine config path
    -> load config
    -> parse sessions
    -> validate all sessions
    -> print warnings/errors
    -> exit with appropriate status code
default russh flow
CLI command received with no subcommand
    -> determine config path
    -> load config
    -> parse sessions
    -> resolve defaults
    -> launch menu
    -> connect selected session
Error Handling Strategy

Russh should fail clearly and without drama.

Error categories
Config errors

Examples:

config file missing
TOML parse failure
invalid schema
required field missing
Resolution errors

Examples:

OS user cannot be resolved
config path cannot be determined
home-directory expansion failed
Validation errors

Examples:

invalid port
empty host
unsupported value format
Execution errors

Examples:

ssh binary not found
failed to spawn child process
permission denied on executable
Output principles
error messages should mention the session name when relevant
field-specific problems should identify the field
warnings should not stop list unless resolution is impossible
check should return non-zero on errors
connect should refuse to proceed on launch-critical errors
hostname-preference guidance should be a warning, never an error
Output Strategy

The CLI should support both human-readable and machine-readable output.

Human-readable output

Default for interactive use.

Examples:

tables
compact detail views
validation summaries
Machine-readable output

Future support via flags like:

russh list --json
russh show web1 --json
russh check --json

This enables scripting and desktop integration reuse if needed.

Interactive Menu Design

The interactive menu is a convenience layer, not a different feature model.

Requirements
keyboard-first
searchable
low visual overhead
shows the same resolved session data as list
selecting an entry behaves the same as connect
launched by default when russh is invoked without subcommands
Suggested display fields
name
user@host
port if non-default or always shown
key summary
tags where space allows

Example row:

web1      root@192.168.1.10:22     ~/.ssh/id_ed25519     prod,web
Security Considerations

Russh deals with connection metadata and must stay conservative.

Security posture
do not store private key contents
do not store passphrases
do not log secrets
only reference key file paths
avoid printing full executable command lines by default if they may expose sensitive paths
Key handling rules
ssh_key is a filesystem path only
paths should be shell-expanded before use
missing key path should warn in check
missing key path should fail on connect if explicitly configured and unavailable
Trust model

Russh assumes the user controls their own local config and shell environment. It is not responsible for remote authentication policies.

Platform Considerations
Phase 1 target platforms
Linux
macOS
Later target platform
Windows
Portability concerns
config path handling
shell/process launching behavior
username resolution APIs
terminal invocation patterns for future desktop launch integration
home-directory expansion semantics

To preserve portability, platform-specific logic should be isolated behind small interfaces.

Desktop Architecture (Future)

The desktop application should be a configuration-focused UI on top of the same core library.

Desktop goals
browse saved sessions
create and edit sessions
validate config before saving
show resolved preview
connect through native terminal
filter by tags or groups
Desktop non-goals for first release
embedded terminal
tabs
session persistence
remote file browsing
connection telemetry
Desktop component model
+---------------------------+
| Session List              |
|---------------------------|
| Search / Filter           |
| Tag filter                |
| Session rows              |
+-------------+-------------+
              |
              v
+---------------------------+
| Session Detail / Editor   |
|---------------------------|
| host                      |
| username                  |
| ssh_key                   |
| port                      |
| tags                      |
| validation state          |
+-------------+-------------+
              |
              v
+---------------------------+
| Connect / Save Actions    |
+---------------------------+
Desktop behavior

The desktop app should either:

edit the same TOML file directly, or
use the core library to read/write config consistently

The desktop app should never invent its own incompatible config model.

Suggested Repository Structure
russh/
├── crates/
│   ├── russh-core/
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── errors.rs
│   │   │   ├── import.rs
│   │   │   ├── model.rs
│   │   │   ├── paths.rs
│   │   │   ├── resolve.rs
│   │   │   ├── ssh.rs
│   │   │   └── validate.rs
│   │   └── Cargo.toml
│   ├── russh-cli/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   └── output/
│   │   └── Cargo.toml
│   └── russh-desktop/
│       └── (future)
├── docs/
│   └── architecture.md
├── examples/
│   └── config.toml
└── Cargo.toml
Suggested Interfaces
Core library API sketch
load_config(path) -> Config
list_sessions(config) -> [Session]
resolve_session(session, env) -> ResolvedSession
validate_config(config) -> [ValidationIssue]
build_ssh_command(resolved_session) -> CommandSpec
execute_ssh(command_spec) -> ProcessResult
CommandSpec

Represents the executable and arguments before process launch.

Suggested shape:

executable: ssh
args: string array
display: optional safe string for dry-run/debug

This separation makes testing easier.

Testing Strategy

Testing should focus heavily on deterministic behavior in the core library.

Unit tests
Config parsing
valid TOML loads correctly
malformed TOML returns parse errors
missing host is rejected
tags parse correctly
Resolution
username defaults to current OS user
port defaults to 22
omitted key means no -i
~ in key path is expanded correctly
Validation
invalid ports are rejected
non-existent key path returns warning or error depending on context
hostname input creates warning in check
IP input does not create hostname preference warning
SSH command building
correct order and formatting of SSH arguments
no -i flag when ssh_key is empty
correct target string generation
expanded key path is used in command args
Integration tests
CLI list output with fixture config
CLI show for named session
CLI check exit codes
CLI no-subcommand launches menu path
CLI connect dry-run path if implemented
Future UI tests

For desktop:

form validation
session creation/edit flows
tag filtering
correct handoff to core library
Observability

Russh should stay quiet by default.

Logging

For early versions:

minimal logging in normal use
optional verbose/debug mode later
never log secrets or key contents
Diagnostics

Useful future flags:

russh check --verbose
russh c web1 --dry-run

A --dry-run mode would be particularly helpful for troubleshooting.

Versioned Roadmap
v0.1
TOML config loading
list
show
connect / c
default no-subcommand menu
menu
check
path resolution
default handling
shell-style key path expansion
hostname support with warning in check
tag support in config and output
v0.2
better validation messages
config path
config edit
--json output
dry-run connect
v0.3
add/remove/update commands
tag filtering in menu
configurable import support from SSH-related sources
v1.0
stable config schema
stable CLI output contract
solid cross-platform behavior
production-ready documentation
import feature sufficiently hardened for daily use
v2.0
desktop application with config editing and launch controls
shared business logic via russh-core
Design Decisions Summary
Chosen
CLI-first approach
TOML configuration
native SSH execution
configuration-centered product design
shared reusable core library
session definitions as remembered targets, not active sessions
hostname support
shell expansion for key paths
default menu when no subcommand is provided
warnings surfaced through check only
tag support in configuration
Deferred
write operations via CLI
JSON output
dry-run connect
configurable import implementation
desktop application
terminal embedding
Rejected for MVP
session persistence
connection monitoring
terminal multiplexing
secret storage
SSH protocol reimplementation
Final Position

Russh succeeds by staying restrained.

It should not become a noisy control tower for every SSH-related workflow. Its strength lies in clarity: a small, dependable tool that remembers targets, shows them plainly, validates them honestly, and opens the road with a single command.

That restraint is not a limitation. It is the architecture's quiet power.