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

.PHONY: build release install uninstall clean test check

build:
	$(CARGO) build

release:
	$(CARGO) build --release

test:
	$(CARGO) test

check:
	$(CARGO) clippy -- -D warnings
	$(CARGO) fmt --check

install: release
	$(INSTALL) target/release/$(BIN) $(DESTDIR)$(BINDIR)/$(BIN)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BIN)

clean:
	$(CARGO) clean
