default: ## Show help
    @just -l

# Development
[group('Development')]
run: 
	cargo build -p waycast-ui --release
	./target/release/waycast

run-nocomp:
	./target/release/waycast

daemon:
	cargo run -p waycast-daemon

[group('Development')]
clean-run: clean build-release
	./target/release/waycast

[group('build')]
build-flake:
	rm -rf result
	nix build .#default

[group('build')]
run-flake:
	./result/bin/waycast

[group('build')]
build: ## Build waycast GUI (debug)
	cargo build -p waycast-ui

[group('build')]
build-release: ## Build waycast GUI (optimized)
	cargo build -p waycast-ui --release

# Testing & Quality
[group('Testing and checking')]
test: ## Run all tests
	cargo test --workspace

[group('Testing and checking')]
check: ## Quick compile check
	cargo check --workspace

[group('Testing and checking')]
fmt: ## Format all code
	cargo fmt --all

[group('Testing and checking')]
lint: ## Run clippy lints
	cargo clippy --workspace --all-targets --all-features --fix

# Dependencies
deps: ## Update dependencies
	cargo update

deps-audit: ## Check for security vulnerabilities
	cargo audit

deps-unused: ## Check for unused dependencies (requires cargo-machete)
	cargo machete

[group('Development')]
clean: ## Clean build artifacts
	cargo clean

[group('Development')]
clean-all: clean ## Deep clean (including cache)
	rm -rf target/
	rm -rf ~/.cargo/registry/cache/

# Performance & Profiling
[group('Profiling')]
bench: ## Run benchmarks
	cargo bench --workspace

[group('Profiling')]
profile: ## Profile the application (requires cargo-flamegraph)
	cargo flamegraph -p waycast-ui
	brave flamegraph.svg

[group('Profiling')]
size: ## Check binary size
	@echo "Binary sizes:"
	@ls -lh target/release/waycast 2>/dev/null || echo "Run 'make release' first"

devicon_dir := "./assets/icons/devicons"
[group('Development')]
devicon-theme: 
	rm -rf $(devicon_dir)
	mkdir -p $(devicon_dir) 
	devicon remix -t framework,language -o $(devicon_dir) --variant original --fallback plain
	devicon get nixos -o $(devicon_dir)
	devicon get bash -o $(devicon_dir)
	devicon get ansible -o $(devicon_dir)
	cp $(devicon_dir)/nixos.svg $(DEVICON_DIR)/nix.svg
	cp $(devicon_dir)/bash.svg $(DEVICON_DIR)/shell.svg

[group('Development')]
install-icons: ## Install icons to XDG data directory
	@XDG_DATA_HOME=$${XDG_DATA_HOME:-$$HOME/.local/share} && \
	ICON_DIR="$$XDG_DATA_HOME/waycast/icons" && \
	mkdir -p "$$ICON_DIR" && \
	cp -r ./assets/icons/* "$$ICON_DIR/" && \
	echo "Icons installed to $$ICON_DIR"

# Release Management
[group('Releasing & publishing')]
plan-release:
    dist plan

[group('Releasing & publishing')]
tag-release version:
    @echo "Tagging release {{version}}"
    git add -A
    git commit -m "chore(release): v{{version}}"
    git tag -fa v{{version}} -m "Release v{{version}}"
    @echo "Release v{{version}} created!"

[group('Releasing & publishing')]
push-release version:
    git push origin master
    git push --force origin v{{version}}
    @echo "Release pushed to origin"

[group('Releasing & publishing')]
release version: 
    just tag-release {{version}}
    just push-release {{version}}

[group('DB')]
reset-db:
	rm waycast.db -f
	rm waycast.db-shm -f
	rm waycast.db-wal -f
	touch waycast.db
	sqlx migrate run --source ./waycast-data/migrations --database-url sqlite://waycast.db

[group('IPC')]
ipc-show:
	printf "show\n" | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/waycast.sock