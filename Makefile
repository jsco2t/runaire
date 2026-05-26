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

.PHONY: help toolchain build test test-ignored test-all test-clipboard test-os-events \
        fmt fmt-check lint lint-fix \
        check verify interop interop-entry bench bench-search bench-search-gate \
        vendor deny audit doc clean completions completions-check

help:  ## Show this help.
	@awk 'BEGIN {FS = ":.*##"; print "Usage: make <target>\n\nTargets:"} /^[a-zA-Z_-]+:.*##/ {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---------------------------------------------------------------------------
# One-time environment setup — run this first on a fresh machine.
# ---------------------------------------------------------------------------

toolchain:  ## Install all dev tooling (Rust toolchain, keepassxc-cli, cargo-deny/audit). Run first on a new machine.
	# Cross-platform bootstrap (Linux + macOS). Idempotent — re-running
	# skips already-installed tools. Tool versions are pinned to match CI
	# (.github/workflows/ci.yml). After this, `make check` works offline.
	# Invoked directly (not via `sh`) so its bash shebang is honoured —
	# the script uses `set -o pipefail`, which POSIX `sh`/dash lacks.
	tools/dev/install-toolchain.sh

# ---------------------------------------------------------------------------
# Core developer loop — these match CI exactly.
# ---------------------------------------------------------------------------

build:  ## Build the workspace (offline, vendored).
	$(CARGO) build $(CARGO_FLAGS)

check-macos:  ## Type-check runaire-security against macOS targets (Level-1 cross-compile gate; no link, no SDK).
	# `cargo check` does macro expansion + type/borrow checking but
	# does NOT invoke the linker, so the absence of Apple frameworks
	# (IOKit, AppKit, Foundation) on Linux does not block this gate.
	# Catches the realistic refactor-breakage class for Phase 5 T5.2's
	# `os_events/macos.rs`: trait-impl drift, signature changes,
	# `objc2` macro errors, missing-arm matches. Does NOT catch
	# wrong-extern-symbol-name or behavioural bugs — for those you
	# need a real macOS host (see CONTRIBUTING.md "macOS verification").
	#
	# `--all-features` is intentionally NOT passed: `logind` is
	# Linux-only (target-cfg-gated dep on `zbus`) and `cargo check`
	# refuses to enable a feature whose deps don't resolve on the
	# target. We check the security crate with default features only,
	# which is what a downstream macOS consumer would see.
	$(CARGO) check --target aarch64-apple-darwin --offline --locked -p runaire-security
	$(CARGO) check --target x86_64-apple-darwin  --offline --locked -p runaire-security

test:  ## Run default-parallel tests (offline, vendored).
	$(CARGO) test $(CARGO_FLAGS)

test-ignored:  ## Run #[ignore]d tests serially (env-mutating + signal-handler tests, etc).
	# `--features test-binaries` enables runaire-security's `sigstop_helper`
	# test binary, needed by `tests/us_052_sigstop_lock.rs`. Other workspace
	# members don't define the feature; cargo silently no-ops on them.
	$(CARGO) test $(CARGO_FLAGS) --features test-binaries -- --ignored --test-threads=1

test-all: test test-ignored  ## Run both default and #[ignore]d tests.

test-clipboard:  ## Run runaire-security clipboard tests (requires real display; wrap in xvfb-run on CI).
	$(CARGO) test -p runaire-security --offline --locked --test us_053_clipboard_autoclear -- --ignored --test-threads=1

test-os-events:  ## Run runaire-security OS-event integration tests (Phase 5: logind on Linux, IOKit on macOS).
	# `--features test-binaries,logind` enables the `logind_helper`
	# binary and the `logind` cargo feature that compiles
	# `os_events/logind.rs` and its `zbus` dep tree. Tests are
	# `#[ignore]`d so the default `make test` skips them; they require
	# a logind-enabled host + `dbus-send` + `busctl` to drive the
	# signals from outside the helper process. Selects test binaries
	# by `--test <name>` so only the post-MVP files run (not e.g.
	# the clipboard or SIGTSTP integration tests).
	$(CARGO) test -p runaire-security --offline --locked --features test-binaries,logind --test us_052_post_mvp_logind -- --ignored --test-threads=1

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

check: fmt-check lint build test check-macos  ## fmt-check + lint + build + test + macOS type-check (the CI gate).

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
# Shell completions (FR-064) — re-generated by `make completions`, checked
# into shell-completions/ for packaging + the CI drift gate.
# ---------------------------------------------------------------------------

completions:  ## Re-generate shell-completions/runaire.{bash,fish} + _runaire.
	$(CARGO) run --example gen_completions -p runaire-cli --offline --locked

completions-check: completions  ## Re-generate completions and fail the build if shell-completions/ drifted (CI gate).
	@drift="$$(git status --porcelain -- shell-completions/)"; \
	if [ -n "$$drift" ]; then \
		echo "error: shell-completions/ drifted. Run \`make completions\` and commit the result." >&2; \
		echo "$$drift" >&2; \
		git --no-pager diff -- shell-completions/ >&2; \
		exit 1; \
	fi

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
