# Waycast - Makefile for development convenience
.PHONY: help build run test clean install dev release check fmt lint fix deps docker

# Default target
help: ## Show this help message
	@echo "Waycast Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Examples:"
	@echo "  make dev          # Start development"
	@echo "  make build-all    # Build everything"
	@echo "  make install      # Install to system"

# Development
run: ## Run waycast GUI
	cargo run -p waycast-gtk

run-daemon: ## Run waycast daemon (when implemented)
	@echo "Daemon not yet implemented"
	# cargo run -p waycast-daemon

# Building
build: ## Build waycast GUI (debug)
	cargo build -p waycast-gtk

build-all: ## Build all crates (debug)
	cargo build --workspace

build-core: ## Build core library only
	cargo build -p waycast-core

build-plugins: ## Build plugins only
	cargo build -p waycast-plugins

# Release builds
release: ## Build waycast GUI (optimized)
	cargo build -p waycast-gtk --release

release-all: ## Build all crates (optimized)
	cargo build --workspace --release

# Testing & Quality
test: ## Run all tests
	cargo test --workspace

test-core: ## Run core tests only
	cargo test -p waycast-core

check: ## Quick compile check
	cargo check --workspace

fmt: ## Format all code
	cargo fmt --all

lint: ## Run clippy lints
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fix: ## Auto-fix linting issues
	cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
	cargo fmt --all

# Dependencies
deps: ## Update dependencies
	cargo update

deps-audit: ## Check for security vulnerabilities
	cargo audit

deps-unused: ## Check for unused dependencies (requires cargo-machete)
	cargo machete

# Installation & Packaging
install: release ## Install waycast to system
	cargo install --path waycast-gtk --force

install-deps: ## Install required system dependencies (Debian/Ubuntu)
	sudo apt update
	sudo apt install -y build-essential libgtk-4-dev libadwaita-1-dev pkg-config

uninstall: ## Remove waycast from system
	cargo uninstall waycast

# Development Environment
setup: install-deps ## Set up development environment
	rustup update
	rustup component add rustfmt clippy
	cargo install cargo-watch cargo-audit cargo-machete
	@echo "Development environment ready!"

# Cleaning
clean: ## Clean build artifacts
	cargo clean

clean-all: clean ## Deep clean (including cache)
	rm -rf target/
	rm -rf ~/.cargo/registry/cache/

# Future: Packaging & Distribution
deb: release ## Build Debian package (future)
	@echo "Debian packaging not yet implemented"
	# Use cargo-deb when ready

rpm: release ## Build RPM package (future)
	@echo "RPM packaging not yet implemented"
	# Use cargo-rpm when ready

flatpak: release ## Build Flatpak (future)
	@echo "Flatpak packaging not yet implemented"

# Future: System Integration
enable-daemon: ## Enable waycast daemon service (future)
	@echo "Daemon service not yet implemented"
	# systemctl --user enable waycast-daemon

disable-daemon: ## Disable waycast daemon service (future)
	@echo "Daemon service not yet implemented"
	# systemctl --user disable waycast-daemon

# Docker (for CI/testing)
docker-build: ## Build in Docker container
	docker build -t waycast-build .

docker-test: ## Test in Docker container
	docker run --rm waycast-build make test

# Performance & Profiling
bench: ## Run benchmarks
	cargo bench --workspace

profile: ## Profile the application (requires cargo-flamegraph)
	cargo flamegraph --bin waycast

size: ## Check binary size
	@echo "Binary sizes:"
	@ls -lh target/release/waycast 2>/dev/null || echo "Run 'make release' first"

# Development workflow shortcuts
quick: fmt check ## Quick development check (format + compile)

full: clean fmt lint test build-all ## Full development check

ci: fmt lint test build-all ## CI pipeline simulation

# Git hooks
hooks: ## Install git hooks
	@echo "#!/bin/sh" > .git/hooks/pre-commit
	@echo "make fmt lint" >> .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Git hooks installed!"

# Documentation
docs: ## Build and open documentation
	cargo doc --workspace --open

# Development tools installation
tools: ## Install useful development tools
	cargo install cargo-watch cargo-audit cargo-machete cargo-flamegraph cargo-deb cargo-outdated
	@echo "Development tools installed!"

devicon-theme: DEVICON_DIR = ./assets/icons/devicons
devicon-theme: 
	rm -rf $(DEVICON_DIR)
	mkdir -p $(DEVICON_DIR) 
	devicon remix -t framework,language -o $(DEVICON_DIR) --variant original --fallback plain
	devicon get nixos -o $(DEVICON_DIR)
	devicon get bash -o $(DEVICON_DIR)
	devicon get ansible -o $(DEVICON_DIR)
	cp $(DEVICON_DIR)/nixos.svg $(DEVICON_DIR)/nix.svg
	cp $(DEVICON_DIR)/bash.svg $(DEVICON_DIR)/shell.svg

install-icons: ## Install icons to XDG data directory
	@XDG_DATA_HOME=$${XDG_DATA_HOME:-$$HOME/.local/share} && \
	ICON_DIR="$$XDG_DATA_HOME/waycast/icons" && \
	mkdir -p "$$ICON_DIR" && \
	cp -r ./assets/icons/* "$$ICON_DIR/" && \
	echo "Icons installed to $$ICON_DIR"