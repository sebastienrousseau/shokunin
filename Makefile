# Makefile using cargo for managing builds and dependencies in a Rust project.

# Default target executed when no arguments are given to make.
.PHONY: all
all: help ## Display this help.

# One-command bootstrap. Detects the host OS, installs missing toolchain
# components via rustup, and wires up the project's git hooks. Re-running
# this target on a fresh clone is a no-op for anything already in place.
.PHONY: init
init: ## Bootstrap a clean clone (rustfmt + clippy + hooks + first build).
	@echo "==> Detecting platform..."
	@printf "    host: " ; uname -s
	@command -v rustup >/dev/null 2>&1 || { \
	    echo "ERROR: rustup not found. Install from https://rustup.rs/"; \
	    exit 1; \
	}
	@rustup show active-toolchain
	@rustup component add rustfmt clippy 2>/dev/null || true
	@command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny
	@$(MAKE) -s hooks
	@cargo build --quiet
	@echo "==> Bootstrap complete. Run 'make test' to verify."

# Install the project-managed git hooks (signed-commit guard, etc).
.PHONY: hooks
hooks: ## Install the project's git hooks under .githooks/.
	@git config core.hooksPath .githooks
	@chmod +x .githooks/pre-commit 2>/dev/null || true
	@echo "✓ Git hooks installed (.githooks/)."

# Build the project including all workspace members.
.PHONY: build
build: ## Build the project.
	@echo "Building all project components..."
	@cargo build --all

# Remove build artifacts and stray logs from the working tree.
.PHONY: clean
clean: ## Remove build artifacts and stray logs.
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf examples/build examples/public public/ build/
	@rm -f site.log site_generation.log
	@echo "Done."

# Lint the project with stringent rules using Clippy, install Clippy if not present.
.PHONY: lint
lint: ensure-clippy ## Lint the project with Clippy.
	@echo "Linting with Clippy..."
	@cargo clippy --all-features --all-targets --all -- \
		--deny clippy::dbg_macro --deny clippy::unimplemented --deny clippy::todo --deny warnings \
		--deny missing_docs --deny broken_intra_doc_links --forbid unused_must_use --deny clippy::result_unit_err

# Run all unit and integration tests in the project.
.PHONY: test
test: ## Run tests for the project.
	@echo "Running tests..."
	@cargo test

# Check the project for errors without producing outputs.
.PHONY: check
check: ## Check the project for errors without producing outputs.
	@echo "Checking code formatting..."
	@cargo check

# Format all code in the project according to rustfmt's standards, install rustfmt if not present.
.PHONY: format
format: ensure-rustfmt ## Format the code.
	@echo "Formatting all project components..."
	@cargo fmt --all

# Check code formatting without making changes, with verbose output, install rustfmt if not present.
.PHONY: format-check-verbose
format-check-verbose: ensure-rustfmt ## Check code formatting with verbose output.
	@echo "Checking code format with verbose output..."
	@cargo fmt --all -- --check --verbose

# Apply fixes to the project using cargo fix, install cargo-fix if necessary.
.PHONY: fix
fix: ensure-cargo-fix ## Automatically fix Rust lint warnings using cargo fix.
	@echo "Applying cargo fix..."
	@cargo fix --all

# Use cargo-deny to check for security vulnerabilities, licensing issues, and more, install if not present.
.PHONY: deny
deny: ensure-cargo-deny ## Run cargo deny checks.
	@echo "Running cargo deny checks..."
	@cargo deny check

# Check for outdated dependencies only for the root package, install cargo-outdated if necessary.
.PHONY: outdated
outdated: ensure-cargo-outdated ## Check for outdated dependencies for the root package only.
	@echo "Checking for outdated dependencies..."
	@cargo outdated --root-deps-only

# Installation checks and setups
.PHONY: ensure-clippy ensure-rustfmt ensure-cargo-fix ensure-cargo-deny ensure-cargo-outdated
ensure-clippy:
	@cargo clippy --version || rustup component add clippy

ensure-rustfmt:
	@cargo fmt --version || rustup component add rustfmt

ensure-cargo-fix:
	@cargo version > /dev/null 2>&1 || (echo "cargo is required" && exit 1)

ensure-cargo-deny:
	@command -v cargo-deny || cargo install cargo-deny

ensure-cargo-outdated:
	@command -v cargo-outdated || cargo install cargo-outdated

# Help target to display callable targets and their descriptions.
.PHONY: help
help: ## Display this help.
	@echo "Usage: make [target]..."
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*?##"} /^[a-zA-Z_-]+:.*?##/ {printf "  %-30s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
