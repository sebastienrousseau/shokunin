# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# The Unix install contract for ssg: `make`, `make test`, `make install`,
# `make uninstall`, honouring PREFIX and DESTDIR and installing to FHS
# paths.
#
# WHY THIS FILE AND NOT Makefile
# ------------------------------
# GNU make reads GNUmakefile *instead of* Makefile when both exist — it
# does not merge them. Adding this file therefore hides every developer
# target the existing Makefile provides (init, lint, coverage, a11y,
# visual, doc, bench, ...), which would be a silent and confusing
# regression. The catch-all rule at the bottom forwards anything defined
# there, so `make lint` still reaches Makefile's `lint`. Keep new
# developer-workflow targets in Makefile; keep packaging here.

BIN         := ssg
CARGO       ?= cargo
CARGO_FLAGS ?= --locked

# GNU standard installation directory variables. Packagers override any of
# these on the command line; DESTDIR is prepended at install time only and
# never compiled into anything.
PREFIX      ?= /usr/local
EXEC_PREFIX ?= $(PREFIX)
BINDIR      ?= $(EXEC_PREFIX)/bin
DATAROOTDIR ?= $(PREFIX)/share
DATADIR     ?= $(DATAROOTDIR)
MANDIR      ?= $(DATAROOTDIR)/man
MAN1DIR     ?= $(MANDIR)/man1
DOCDIR      ?= $(DATAROOTDIR)/doc/$(BIN)

# Completion directories. bash, zsh and fish have settled conventions;
# PowerShell has none on Unix, so its script is generated and shipped in
# the dist tree but installed only where a packager points PWSHCOMPDIR.
BASHCOMPDIR ?= $(DATAROOTDIR)/bash-completion/completions
ZSHCOMPDIR  ?= $(DATAROOTDIR)/zsh/site-functions
FISHCOMPDIR ?= $(DATAROOTDIR)/fish/vendor_completions.d
PWSHCOMPDIR ?=

INSTALL         ?= install
INSTALL_PROGRAM ?= $(INSTALL) -m 755
INSTALL_DATA    ?= $(INSTALL) -m 644
# `install -D` is a GNU extension that BSD and macOS lack, so directories
# are created explicitly. This target has to work on a packager's machine,
# not only on a Linux CI runner.
MKDIR_P         ?= mkdir -p

DIST    := target/dist
RELEASE := target/release/$(BIN)

.PHONY: all
all: build man completions ## Build the binary, man page and completions.

.PHONY: build
build: $(RELEASE) ## Build the release binary.

$(RELEASE):
	$(CARGO) build $(CARGO_FLAGS) --release --bin $(BIN)

.PHONY: test
test: ## Run the test suite.
	$(CARGO) test $(CARGO_FLAGS) --tests

# Both artefacts come from one generator run, because both are derived
# from the same clap definition and there is no reason to walk it twice.
# `--release` so this shares the dependency build with the `build` target
# above rather than compiling the whole tree a second time in the debug
# profile, which is what `make install` would otherwise cost on a cold CI
# runner.
.PHONY: artifacts
artifacts:
	@$(MKDIR_P) $(DIST)
	$(CARGO) run $(CARGO_FLAGS) --release --quiet \
	    --example gen-artifacts -- $(DIST)

.PHONY: man
man: artifacts ## Generate $(DIST)/man/$(BIN).1 from the CLI definition.

.PHONY: completions
completions: artifacts ## Generate shell completions from the CLI definition.

.PHONY: install
install: build artifacts ## Install to $(DESTDIR)$(PREFIX).
	$(MKDIR_P) $(DESTDIR)$(BINDIR)
	$(INSTALL_PROGRAM) $(RELEASE) $(DESTDIR)$(BINDIR)/$(BIN)
	$(MKDIR_P) $(DESTDIR)$(MAN1DIR)
	$(INSTALL_DATA) $(DIST)/man/$(BIN).1 $(DESTDIR)$(MAN1DIR)/$(BIN).1
	$(MKDIR_P) $(DESTDIR)$(BASHCOMPDIR)
	$(INSTALL_DATA) $(DIST)/completions/$(BIN) \
	    $(DESTDIR)$(BASHCOMPDIR)/$(BIN)
	$(MKDIR_P) $(DESTDIR)$(ZSHCOMPDIR)
	$(INSTALL_DATA) $(DIST)/completions/_$(BIN) \
	    $(DESTDIR)$(ZSHCOMPDIR)/_$(BIN)
	$(MKDIR_P) $(DESTDIR)$(FISHCOMPDIR)
	$(INSTALL_DATA) $(DIST)/completions/$(BIN).fish \
	    $(DESTDIR)$(FISHCOMPDIR)/$(BIN).fish
	@if [ -n "$(PWSHCOMPDIR)" ]; then \
	    $(MKDIR_P) $(DESTDIR)$(PWSHCOMPDIR); \
	    $(INSTALL_DATA) $(DIST)/completions/_$(BIN).ps1 \
	        $(DESTDIR)$(PWSHCOMPDIR)/_$(BIN).ps1; \
	fi
	$(MKDIR_P) $(DESTDIR)$(DOCDIR)
	$(INSTALL_DATA) README.md $(DESTDIR)$(DOCDIR)/README.md
	$(INSTALL_DATA) CHANGELOG.md $(DESTDIR)$(DOCDIR)/CHANGELOG.md
	$(INSTALL_DATA) LICENSE-APACHE $(DESTDIR)$(DOCDIR)/LICENSE-APACHE
	$(INSTALL_DATA) LICENSE-MIT $(DESTDIR)$(DOCDIR)/LICENSE-MIT

.PHONY: install-strip
install-strip: ## Install, then strip the installed binary.
	$(MAKE) INSTALL_PROGRAM='$(INSTALL) -m 755 -s' install

# Removes exactly what install created, and prunes only the directories
# this package owns. Shared directories such as $(BINDIR) are left alone.
.PHONY: uninstall
uninstall: ## Remove everything `install` placed under $(DESTDIR)$(PREFIX).
	rm -f $(DESTDIR)$(BINDIR)/$(BIN)
	rm -f $(DESTDIR)$(MAN1DIR)/$(BIN).1
	rm -f $(DESTDIR)$(BASHCOMPDIR)/$(BIN)
	rm -f $(DESTDIR)$(ZSHCOMPDIR)/_$(BIN)
	rm -f $(DESTDIR)$(FISHCOMPDIR)/$(BIN).fish
	@if [ -n "$(PWSHCOMPDIR)" ]; then \
	    rm -f $(DESTDIR)$(PWSHCOMPDIR)/_$(BIN).ps1; \
	fi
	rm -rf $(DESTDIR)$(DOCDIR)

.PHONY: dist-clean
dist-clean: ## Remove the generated packaging tree.
	rm -rf $(DIST)

.PHONY: help
help: ## Display the packaging targets (see Makefile for dev targets).
	@echo "Usage: make [target]... [PREFIX=/usr] [DESTDIR=/tmp/stage]"
	@echo ""
	@echo "Packaging targets (GNUmakefile):"
	@awk 'BEGIN {FS = ":.*?##"} /^[a-zA-Z_-]+:.*?##/ \
	    {printf "  %-16s %s\n", $$1, $$2}' GNUmakefile
	@echo ""
	@echo "Developer targets (Makefile) — forwarded automatically:"
	@awk 'BEGIN {FS = ":.*?##"} /^[a-zA-Z_-]+:.*?##/ \
	    {printf "  %-16s %s\n", $$1, $$2}' Makefile

# Anything not defined above is a developer target: forward it to Makefile
# so this file's existence does not hide it. The rule is terminal (`::`)
# so make will not try to use it to remake the makefiles themselves, and
# the makefiles get explicit empty rules for the same reason.
GNUmakefile Makefile: ;
%:: ; @$(MAKE) --no-print-directory -f Makefile $@
