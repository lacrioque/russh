# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

**russher** is a Rust SSH toolkit built as a Cargo workspace.

## Workspace Structure

```
russher/
├── russh-core/    # Core SSH library (sessions, channels, crypto)
└── russh-cli/    # CLI interface for SSH operations
```

## Development Commands

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run tests for a specific crate
cargo test -p russh-core
cargo test -p russh-cli
```

## Language

This is a **Rust** project. Use `cargo` commands, not `go` commands.
