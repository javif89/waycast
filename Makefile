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

ice:
	cargo build -p waycast-iced --release
	./target/release/waycast-iced

run-daemon: 
	cargo run -p waycast-daemon

call-daemon:
	busctl --user call dev.waycast.Daemon /dev/waycast/Daemon dev.waycast.Daemon $(METHOD) $(PARAMS)

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
build-release: ## Build waycast GUI (optimized)
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
	cargo clippy --workspace --all-targets --all-features

lint-fix: ## Auto-fix linting issues
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

# Performance & Profiling
bench: ## Run benchmarks
	cargo bench --workspace

profile: ## Profile the application (requires cargo-flamegraph)
	cargo flamegraph
	brave flamegraph.svg

size: ## Check binary size
	@echo "Binary sizes:"
	@ls -lh target/release/waycast 2>/dev/null || echo "Run 'make release' first"

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

# Release Management
release: 
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release VERSION=0.0.2"; \
		exit 1; \
	fi
	@git add -A
	@git commit -m "chore(release): v$(VERSION)"
	git tag -fa v$(VERSION) -m "Release v$(VERSION)"
	@git push origin master
	git push --force origin v$(VERSION)
	@echo "✅ Release v$(VERSION) created!"
	@echo "🔗 Go to your Gitea instance to add release notes"