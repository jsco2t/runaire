# Rùnaire — Makefile
#
# This is the canonical build / test / lint interface for the project.
# Every developer-facing workflow has a target here so that local runs
# and CI run identical commands. Raw `cargo` invocations are reserved
# for ad-hoc exploration; anything that's part of the project's normal
# loop lives here.
#
# Run `make help` (or `make` alone) to list every target.
#
# All builds are offline against the vendored dependency tree. The
# project's `.cargo/config.toml` already enforces `[net] offline = true`;
# the explicit `--offline --locked` flags below are belt-and-suspenders
# so the intent is visible at the call site.

CARGO          := cargo
CARGO_FLAGS    := --workspace --offline --locked
CLIPPY_FLAGS   := --workspace --all-targets --offline --locked -- -D warnings

# `make` with no args shows the help screen — friendlier for first-run.
# Devs who know what they want type `make build`, `make test`, `make check`.
.DEFAULT_GOAL := help

.PHONY: help build test test-ignored test-all fmt fmt-check lint lint-fix \
        check verify interop interop-entry bench bench-search bench-search-gate \
        vendor deny audit doc clean

help:  ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; print "Usage: make <target>\n\nTargets:"} /^[a-zA-Z_-]+:.*##/ {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---------------------------------------------------------------------------
# Core developer loop — these match CI exactly.
# ---------------------------------------------------------------------------

build:  ## Build the workspace (offline, vendored).
	$(CARGO) build $(CARGO_FLAGS)

test:  ## Run default-parallel tests (offline, vendored).
	$(CARGO) test $(CARGO_FLAGS)

test-ignored:  ## Run #[ignore]d tests serially (env-mutating tests, etc).
	$(CARGO) test $(CARGO_FLAGS) -- --ignored --test-threads=1

test-all: test test-ignored  ## Run both default and #[ignore]d tests.

fmt:  ## Auto-format the workspace.
	$(CARGO) fmt --all

fmt-check:  ## Verify formatting without modifying files (CI gate).
	$(CARGO) fmt --all --check

lint:  ## Run clippy with `-D warnings` (CI gate).
	$(CARGO) clippy $(CLIPPY_FLAGS)

lint-fix:  ## Apply clippy auto-fixes where safe.
	$(CARGO) clippy --workspace --all-targets --offline --locked --fix --allow-dirty

# ---------------------------------------------------------------------------
# Full local CI gate — run before pushing.
# ---------------------------------------------------------------------------

check: fmt-check lint build test  ## fmt-check + lint + build + test (the CI gate).

verify: check test-ignored doc deny audit interop interop-entry  ## Run the full core verification gate.

interop:  ## Run vault-core KeePassXC interop shell tests (requires keepassxc-cli).
	$(CARGO) build -p runaire-core --bin runaire-test-driver --offline --locked
	RUNAIRE_TEST_DRIVER=target/debug/runaire-test-driver sh tools/interop-tests/us_090_rust_to_kpxc.sh
	RUNAIRE_TEST_DRIVER=target/debug/runaire-test-driver sh tools/interop-tests/us_091_kpxc_to_rust.sh
	RUNAIRE_TEST_DRIVER=target/debug/runaire-test-driver sh tools/interop-tests/us_092_round_trip.sh

interop-entry:  ## Run entry-management KeePassXC interop shell tests (requires keepassxc-cli; oathtool optional).
	$(CARGO) build -p runaire-core --bin runaire-test-driver --offline --locked
	@for script in tools/interop-tests/entry_us-010.sh \
	               tools/interop-tests/entry_us-012.sh \
	               tools/interop-tests/entry_us-013.sh \
	               tools/interop-tests/entry_us-014.sh \
	               tools/interop-tests/entry_us-016.sh \
	               tools/interop-tests/entry_us-018.sh; do \
		echo "==> $$script"; \
		RUNAIRE_TEST_DRIVER=target/debug/runaire-test-driver "$$script" || exit $$?; \
	done

bench:  ## Run informational benchmarks.
	$(CARGO) bench -p runaire-core --bench vault_open --offline --locked

bench-search:  ## Run the informational entry-search benchmark.
	$(CARGO) bench -p runaire-core --bench bench_search --offline --locked

bench-search-gate:  ## NFR-002 gate: fail if entry-search exceeds the latency budget (BUDGET_MS overridable).
	tools/bench/bench_search_gate.sh

# ---------------------------------------------------------------------------
# Supply-chain and docs.
# ---------------------------------------------------------------------------

deny:  ## Run cargo-deny license + advisory + ban checks (requires cargo-deny installed).
	$(CARGO) deny check

audit:  ## Run cargo-audit against RustSec advisories (requires cargo-audit installed).
	$(CARGO) audit

doc:  ## Generate API docs locally.
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --offline

# ---------------------------------------------------------------------------
# Dependency vendoring — the only target that needs network access.
# See CONTRIBUTING.md "Adding a dependency" for the full 5-step workflow.
# ---------------------------------------------------------------------------

vendor:  ## Re-vendor dependencies into vendor/. REQUIRES NETWORK ACCESS.
	CARGO_NET_OFFLINE=false $(CARGO) vendor

# ---------------------------------------------------------------------------
# Housekeeping.
# ---------------------------------------------------------------------------

clean:  ## Remove build artifacts (target/).
	$(CARGO) clean
