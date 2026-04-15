# Makefile for russh — cross-platform (macOS / Linux / WSL)

CARGO   ?= cargo
PREFIX  ?= /usr/local
BINDIR  ?= $(PREFIX)/bin
BIN     := russh

# Detect OS
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    INSTALL := install -m 755
else
    INSTALL := install -D -m 755
endif

.PHONY: all build release test check fmt lint clippy ci install uninstall clean help

all: build

build:
	$(CARGO) build

release:
	$(CARGO) build --release

test:
	$(CARGO) test --workspace

# Individual quality gates
fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

# Composite: run all checks (lint + clippy + tests). Use before committing.
check: lint clippy test

# CI target: what GitHub Actions runs.
ci: check

install: release
	$(INSTALL) target/release/$(BIN) $(DESTDIR)$(BINDIR)/$(BIN)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BIN)

clean:
	$(CARGO) clean

help:
	@echo "russh — Makefile targets"
	@echo ""
	@echo "  build      Debug build (cargo build)"
	@echo "  release    Optimized build (cargo build --release)"
	@echo "  test       Run all workspace tests"
	@echo "  fmt        Apply rustfmt"
	@echo "  lint       Check rustfmt (no changes)"
	@echo "  clippy     Run clippy with -D warnings"
	@echo "  check      lint + clippy + test (pre-commit gate)"
	@echo "  ci         Alias for check"
	@echo "  install    Install release binary to \$$DESTDIR\$$PREFIX/bin"
	@echo "  uninstall  Remove installed binary"
	@echo "  clean      Remove build artifacts"
