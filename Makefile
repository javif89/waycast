# Waycast - Makefile for development convenience
.PHONY: help build run test clean install dev release check fmt lint fix deps docker

# Default target
help: ## Show this help message
	@echo "Waycast Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@echo ""

# Development
run: ## Run waycast GUI
	cargo build -p waycast-ui --release
	./target/release/waycast

clean-run: clean build-release
	./target/release/waycast

build-flake:
	rm -rf result
	nix build .#default

run-flake:
	./result/bin/waycast

# Building
build: ## Build waycast GUI (debug)
	cargo build -p waycast-ui

# Release builds
build-release: ## Build waycast GUI (optimized)
	cargo build -p waycast-ui --release

# Testing & Quality
test: ## Run all tests
	cargo test --workspace

check: ## Quick compile check
	cargo check --workspace

fmt: ## Format all code
	cargo fmt --all

lint: ## Run clippy lints
	cargo clippy --workspace --all-targets --all-features --pedantic

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
	cargo flamegraph -p waycast-ui
	brave flamegraph.svg

size: ## Check binary size
	@echo "Binary sizes:"
	@ls -lh target/release/waycast 2>/dev/null || echo "Run 'make release' first"

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