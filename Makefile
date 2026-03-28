CARGO  ?= cargo
APP_ID := com.github.zongflow
RUST_LOG_DEBUG := zongflow=debug

# ── Debug run (full logging) ────────────────────────────────────────

.PHONY: run
run: ## Build & run in debug mode with all logs enabled
	RUST_LOG=$(RUST_LOG_DEBUG) $(CARGO) run --features debug

.PHONY: run-release
run-release: ## Build & run in release mode with all logs enabled
	RUST_LOG=$(RUST_LOG_DEBUG) $(CARGO) run --release

# ── cargo watch ─────────────────────────────────────────────────────

.PHONY: watch-check
watch-check: ## Continuously run `cargo check` on changes
	$(CARGO) watch -x check

.PHONY: watch-test
watch-test: ## Continuously run `cargo test` on changes
	$(CARGO) watch -x test

.PHONY: watch-clippy
watch-clippy: ## Continuously run clippy on changes
	$(CARGO) watch -x 'clippy -- -W deprecated'

.PHONY: watch-run
watch-run: ## Rebuild & rerun on changes (full logging)
	$(CARGO) watch -s 'RUST_LOG=$(RUST_LOG_DEBUG) cargo run --features debug'

# ── Standard targets ────────────────────────────────────────────────

.PHONY: build
build: ## Build in debug mode
	$(CARGO) build

.PHONY: build-release
build-release: ## Build in release mode
	$(CARGO) build --release

.PHONY: check
check: ## Run `cargo check`
	$(CARGO) check

.PHONY: test
test: ## Run all tests
	$(CARGO) test

.PHONY: clippy
clippy: ## Run clippy with deprecation warnings
	$(CARGO) clippy -- -W deprecated

.PHONY: fmt
fmt: ## Format code
	$(CARGO) fmt

.PHONY: fmt-check
fmt-check: ## Check formatting without changes
	$(CARGO) fmt -- --check

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean

# ── Help ────────────────────────────────────────────────────────────

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
