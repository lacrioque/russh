# ADR-0001: Core Design Decisions for Russh v0.1

## Status

Accepted

## Date

2026-03-21

---

## Context

Russh is being designed as a lightweight SSH session manager with a CLI-first approach and a future desktop application. The goal is to create a tool that improves how users **store, inspect, validate, and launch SSH connections**, without replacing or reimplementing SSH itself.

At this stage, multiple foundational decisions must be made to ensure:

- long-term architectural clarity
- consistency between CLI and future desktop UI
- minimal complexity in the first release
- a strong, reusable core

This ADR consolidates the key architectural and product decisions that define Russh v0.1.

---

## Decision Summary

The following core decisions are established:

1. Russh is a **session registry and launcher**, not a terminal or SSH replacement
2. The system is **CLI-first**, with a **shared core library** for future desktop use
3. Configuration is stored in **TOML**
4. Russh **delegates all connections to native SSH**
5. Session definitions are **static targets**, not active connections
6. Default values are resolved at runtime
7. **Shell-style path expansion** is supported
8. **Hostnames are supported**, but **IP addresses are preferred (warning only)**
9. Warnings are surfaced only through `russh check`
10. Running `russh` with no arguments opens an **interactive menu**
11. Sessions support optional **tags for grouping**
12. Import from SSH-related sources is planned but **deferred and configurable**

Each of these decisions is detailed below.

---

## Decision 1: Russh is a Session Registry and Launcher

### Decision

Russh will manage **stored SSH session definitions** and provide commands to inspect and launch them. It will not manage active sessions.

### Rationale

- Keeps the scope focused and maintainable
- Avoids complexity of session lifecycle management
- Aligns with CLI-first workflows
- Prevents overlap with tools like tmux or terminal emulators

### Consequences

- No session persistence
- No multiplexing
- No session monitoring
- Strong clarity of purpose

---

## Decision 2: CLI-First with Shared Core Library

### Decision

Russh will be implemented as:

- a reusable **core library (`russh-core`)**
- a **CLI frontend (`russh-cli`)**
- a future **desktop frontend (`russh-desktop`)**

### Rationale

- Prevents duplication of logic
- Ensures consistent behavior across interfaces
- Enables future expansion without architectural rewrites

### Consequences

- Core must remain UI-agnostic
- CLI and desktop must rely on shared logic
- Requires disciplined module boundaries early

---

## Decision 3: TOML-Based Configuration

### Decision

Russh will use **TOML** as its configuration format.

### Rationale

- Human-readable and structured
- Native support in Rust ecosystem
- Familiar to developers
- Suitable for hierarchical session definitions

### Consequences

- Users must edit config manually in v0.1
- Schema must remain stable once published
- Parsing and validation must be robust

---

## Decision 4: Native SSH Delegation

### Decision

Russh will not implement SSH. It will construct arguments and invoke the system `ssh` binary.

### Rationale

- Avoids reimplementing a complex protocol
- Leverages existing user environment (SSH config, agent, etc.)
- Ensures compatibility with existing workflows

### Consequences

- Behavior depends on system SSH installation
- Russh must clearly map config to SSH flags
- Debugging may involve underlying SSH behavior

---

## Decision 5: Sessions Are Static Definitions

### Decision

A session represents a **stored target**, not an active connection.

### Rationale

- Simplifies mental model
- Avoids runtime state tracking
- Aligns with config-driven design

### Consequences

- No session state
- No reconnection logic
- No connection history in v0.1

---

## Decision 6: Runtime Default Resolution

### Decision

Missing fields are resolved at runtime:

- `username` → current OS user
- `port` → 22
- `ssh_key` → system default behavior

### Rationale

- Reduces config verbosity
- Matches expectations of SSH users
- Keeps config clean and minimal

### Consequences

- Requires environment-aware resolution logic
- Output must clearly reflect resolved values

---

## Decision 7: Shell-Style Path Expansion

### Decision

Paths such as `~/.ssh/id_ed25519` must be expanded internally.

### Rationale

- Matches user expectations
- Common convention in SSH workflows
- Improves usability without requiring full paths

### Consequences

- Requires platform-aware home directory resolution
- Must be applied consistently in validation and execution

---

## Decision 8: Hostname Support with Preference Warning

### Decision

Russh supports both:

- IP addresses
- hostnames (DNS or internal)

However, **hostnames will trigger a warning in `check`**, indicating that IPs are preferred.

### Rationale

- Hostnames are widely used and necessary
- IP addresses provide operational clarity and stability
- Encourages good practices without blocking flexibility

### Consequences

- Validation must detect hostname vs IP
- Warning must not block usage
- No warnings during `list` or `connect`

---

## Decision 9: Warnings Only in `check`

### Decision

Non-fatal warnings (e.g., hostname preference, missing key files) are only displayed in:

```bash
russh check
```

### Rationale

- Keeps normal commands clean and predictable
- Avoids noise in automation or daily usage
- Provides a dedicated diagnostic command

### Consequences

- Users must run check proactively
- CLI commands remain quiet by default
- Validation logic must distinguish errors vs warnings


## Decision 10: Default Command Opens Interactive Menu

### Decision

Running:

```
russh
```

(with no subcommand) opens the interactive session picker.

### Rationale

- Provides immediate utility
- Reduces friction for common workflows
- Aligns with CLI tools that offer interactive modes

### Consequences

- CLI must handle no-argument case explicitly
- Menu must be reliable and responsive
- Behavior must be clearly documented

## Decision 11: Tag Support for Sessions

### Decision

Sessions may include optional tags:

```
tags = ["prod", "web"]
```

### Rationale

- Enables grouping and filtering
- Prepares for future UI features (desktop, filtering)
- Adds minimal complexity to the model

### Consequences

- Must be parsed and validated
- Must appear in list and menu output
- Filtering functionality can be added later without schema change


## Decision 12: Import from SSH Sources (Deferred, Configurable)

### Decision

Russh may support importing from SSH-related sources (e.g., ~/.ssh/config) in the future, but:

- it is not part of v0.1
- it must be explicitly configurable

### Rationale

- Many users already maintain SSH config files
- Import can improve onboarding
- ~/.ssh is often permission-sensitive and must not be assumed

### Consequences
- Import logic must be isolated in a dedicated module
- Must not override Russh config implicitly
- Must be opt-in and transparent

## Alternatives Considered

### Using YAML or JSON instead of TOML

Rejected:

- YAML is more flexible but less predictable
- JSON is less readable for manual editing

### Embedding a terminal

Rejected:

- Adds complexity
- Not required for core value
- Better handled by desktop version later

### Parsing and replacing SSH config entirely

Rejected:

- Too invasive
- Breaks user expectations
- Increases maintenance burden

### Always showing warnings

Rejected:

- Creates noise
- Reduces usability for frequent commands

### Requiring IP addresses only

Rejected:

- Too restrictive
- Breaks common workflows using hostnames


## Consequences (Overall)

### Positive
- Clear and focused product scope
- Strong foundation for future expansion
- Minimal complexity in v0.1
- High usability for CLI users
- Clean separation of concerns

### Negative

- Some advanced SSH use cases are not supported initially
- Users must run check explicitly for diagnostics
- No built-in editing or import in v0.1
- Requires careful documentation to avoid confusion

### Future Evolution

These decisions intentionally leave space for:

- desktop UI built on the same core
- import capabilities from SSH config
- tagging and filtering features
- JSON output for scripting
- richer validation and diagnostics
- bastion type ssh over ssh

The architecture is designed to grow without breaking the original simplicity.

## Final Statement

Russh is built on restraint.

It does not try to be everything SSH could be. Instead, it becomes a steady hand: remembering connections, presenting them clearly, and opening them without friction.

This ADR defines that quiet discipline—the shape of a tool that does less, and therefore does it well.