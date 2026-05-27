# Rùnaire

Rùnaire (Scottish Gaelic: "keeper of secrets") is an offline-first secrets manager built on a Rust core with thin cross-platform UIs. Vaults are stored in the KDBX (KeePass) format via the `keepass-rs` crate, so every Rùnaire vault is directly interoperable with KeePassXC, KeeWeb, KeePass2, and mobile KDBX clients. Default sync transport is git, with in-app three-way merge at the entry level.

**License:** MIT. **Status:** Greenfield (Phase 0 in progress).

## Planning docs (authoritative)

Engineering plans live outside this repo in the project notebook. Read these when scoping work — do not duplicate them here.

- **PRD:** `$HOME/Developer/sources/personal/notebook/projects/runaire/prd.md` — the product spec; section IDs (FR-xxx, NFR-xxx) are referenced throughout the plans.
- **Project index:** `$HOME/Developer/sources/personal/notebook/projects/runaire/index.md`
- **Verifications (user scenarios):** `…/runaire/verifications/` — every Must-Have FR maps to a US-xxx scenario.
- **Knowledge base:** `…/runaire/kb/` — KDBX format, three-way merge algorithm, memory hygiene, supply-chain policy, library notes.
- **Features:** `…/runaire/features/<slug>/` — per-feature implementation plan, design, task plan, follow-ups. First feature: `features/vault-core/`.

When implementing a feature, the corresponding `plans/implementation-plan.md` and `plans/design.md` are the source of truth for that feature's scope and architecture.

## Architecture posture

- **Rust core, thin UIs.** All business logic and crypto live in the Rust core library. CLI and TUI are presentation layers over the same core. Future Flutter UIs (Phase 1+) bind to the same core via FFI. No UI-layer logic leaks into the core.
- **TUI is the reference UX.** Every feature lands in the TUI with full keyboard parity before any GUI work begins. CLI provides one-shot scriptable access to every core operation.
- **Offline-first.** Every feature works without network. Sync is opt-in and operates on top of the offline core — never a precondition.
- **Sync trait abstraction.** Phase 0 git is one implementation; later transports (NFS, Samba, cloud) implement the same trait. CRUD and merge logic stay transport-agnostic.

## Engineering principles

1. **KDBX interop is a product promise, not an implementation detail.** A Rùnaire vault must round-trip through KeePassXC (≥2.7) with zero observable data loss. Any custom data written into a vault must be readable by a standards-compliant KDBX client. Round-trip CI tests gate every change to the vault layer.
2. **Test-forward.** Every Must-Have FR has at least one automated test AND one user-scenario verification in `notebook/.../verifications/`. The coverage matrix is a CI gate. Property-based / fuzz tests for the merge engine and fault-injection tests for atomic writes are first-class deliverables, not nice-to-haves.
3. **Simplicity over cleverness.** Prefer obvious code, narrow abstractions, and well-trodden patterns. Don't introduce generality for hypothetical future requirements. Three similar lines beats a premature abstraction.
4. **No data loss, ever.** During sync conflicts, the loser is preserved as a KDBX history entry under the same UUID. Atomic writes (write-then-rename) ensure a crash never leaves a corrupted vault. Pre-merge state is kept as `.kdbx.bak`.
5. **Honest about limits.** Document platform limitations (encrypted swap, `mlock` quotas, OS lock-event reliability) rather than papering over them. Cosmic-ray paranoia (Rowhammer, cold-boot DMA) is explicitly out of scope.
6. **Limited External Dependencies** Only add external dependencies when it is very clear they solve a major gap in functionality. For minor features it's worth evaluating writing the code directly in the repo (vs adding N more dependencies).

## Security rules (non-negotiable)

- **Zeroize on drop** for every type holding sensitive bytes (master keys, derived keys, entry passwords, master-password input buffers). Use `zeroize::ZeroizeOnDrop`.
- **`mlock` the master key** where the platform supports it (best-effort; quotas are small, so don't lock everything).
- **No plaintext to disk, ever.** The unlocked KDBX is never serialized in plaintext. Never log entry contents. Never include secret material in error messages.
- **Master password collection: secure stdin prompt only.** Never accept on the command line, never accept as an env var (leaks via shell history / `ps`).
- **No master-password recovery.** Warn at vault creation; there is no escrow, backdoor, or recovery service.
- **Atomic writes.** Always write to a sibling temp file and `rename(2)`. Never truncate the live vault.
- **Advisory file locking** coordinates concurrent CLI/TUI/agent processes. Concurrent reads OK; writes serialized.
- **Disable core dumps** for the agent and TUI (`setrlimit(RLIMIT_CORE, 0)`).
- **No telemetry, no analytics, no crash reporting, no update checks.** The only network calls are user-configured sync.
- **No hand-rolled crypto.** Cryptography comes from `keepass-rs` and the RustCrypto family (Argon2, AES, HMAC, SHA). KDBX4 KDF is Argon2id, tuned to ~1s on target hardware.
- **CSPRNG only.** Password generation uses `OsRng` from `rand`.

## Supply chain rules (non-negotiable)

- **Permissive licenses only:** MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, Unlicense, CC0-1.0, Unicode-3.0, Unicode-DFS-2016.
- **Forbidden:** GPL-2.0, GPL-3.0, LGPL-2.1/3.0, AGPL-3.0, SSPL, Commons Clause, anything copyleft. **GPL-with-linking-exception is also forbidden** (e.g., libgit2 — the ambiguity isn't worth it; use `gitoxide`).
- **Pinned exact versions.** `Cargo.lock` is the source of truth and is committed.
- **Vendored dependency tree.** All crates vendored at `vendor/` via `cargo vendor`. Builds run `--offline` / `CARGO_NET_OFFLINE=true`.
- **No build-script networking.** Dependencies' `build.rs` must not fetch anything or shell out to undocumented executables.
- **Dependency-add/upgrade checklist** (documented in PR description): license check, upstream maintenance signal, popularity baseline, diff review of vendored sources.
- **Enforcement:** `cargo deny` for license + advisory + banned crates; `cargo audit` for RustSec advisories. Both are CI gates.

## Technical constraints

- **Language:** Rust. No `unsafe` outside well-justified, locally-audited blocks. No FFI in the core (FFI lives at UI boundaries only).
- **Target platforms (Phase 0):** macOS (aarch64 + x86_64) and Linux (x86_64 + aarch64). Windows is not a Phase 0 target.
- **Storage format:** KDBX4 (write); KDBX3 (read, for migration). No proprietary format.
- **State / vault default directory:** `$HOME/.local/state/runaire/` on both macOS and Linux (deliberately consistent — not following macOS `Application Support/`). Per-vault override allowed.
- **Vault registration:** TOML at `$HOME/.local/state/runaire/vaults.toml`.
- **Key libraries (locked in):**
  - `keepass-rs` (MIT) — KDBX read/write.
  - `gitoxide` / `gix-*` (MIT OR Apache-2.0) — git transport.
  - `zeroize` (MIT/Apache-2.0) — memory hygiene.
  - `rand` / `OsRng` (MIT/Apache-2.0) — CSPRNG.
  - `clap` (MIT/Apache-2.0) — CLI parsing.
  - `ratatui` (MIT) — TUI framework.
  - RustCrypto family — crypto primitives.

## Build system

**`make` is the build system of record.** Every developer-facing workflow — build, test, lint, format, vendor, supply-chain audit, docs — has a target in the top-level [`Makefile`](Makefile). Developer workstations and CI run identical commands by going through `make`; raw `cargo` invocations are reserved for ad-hoc exploration. Run `make` (or `make help`) to list every target.

The canonical targets:

| Target              | What it does                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------- |
| `make toolchain`    | One-time dev bootstrap: Rust (via `rustup`), `keepassxc-cli`, `cargo-deny`/`audit` (x-plat, idempotent) |
| `make build`        | `cargo build --workspace --offline --locked`                                                            |
| `make test`         | Default-parallel tests (`--offline --locked`)                                                           |
| `make test-ignored` | `#[ignore]`d tests, serial (`--test-threads=1`) — env-mutating tests, etc.                              |
| `make test-all`     | Both of the above                                                                                       |
| `make fmt`          | Auto-format the workspace                                                                               |
| `make fmt-check`    | Format check (CI gate)                                                                                  |
| `make lint`         | `cargo clippy ... -- -D warnings` (CI gate)                                                             |
| `make lint-fix`     | Apply safe clippy suggestions                                                                           |
| `make check`        | `fmt-check` + `lint` + `build` + `test` (the full local CI gate)                                        |
| `make deny`         | `cargo deny check` (license + advisory + bans)                                                          |
| `make audit`        | `cargo audit` (RustSec advisories)                                                                      |
| `make doc`          | Generate API docs (`cargo doc --no-deps --offline`)                                                     |
| `make vendor`       | Re-vendor deps (the only target that needs network)                                                     |
| `make clean`        | Remove build artifacts                                                                                  |

### Keeping the Makefile up to date — a project rule

**When new functionality introduces a new developer or CI command, add a corresponding `make` target in the same change.** This includes:

- A new lint, formatter, or static-analysis pass.
- A new test runner or test category (benchmarks, fuzzers, property-based, integration harnesses).
- A new code-generation step.
- A new shell-script harness (e.g., Phase-6 `tools/interop-tests/`).
- A new release / packaging / signing step.

Two reasons this rule is non-negotiable:

1. **Discoverability.** Every workflow lives in `make help`; no command exists only inside someone's shell history, a CI YAML, or a commit message.
2. **CI / dev parity.** CI invokes `make`, so anything CI does is reproducible locally with the same target name. The day a CI step diverges from a `make` target is the day "works on my machine" becomes possible.

Concretely: if a PR adds a `run: cargo ...` (or any shell command) to a CI workflow, the equivalent `make` target must land in the same PR. The Phase-6 tasks T6.2 (interop-tests harness), T6.3 (CI matrix completion), and T6.4 (KDF benchmark) will each add Makefile targets.

If a target's command becomes long or grows multiple cases, prefer adding flag variables (`CLIPPY_FLAGS`, etc.) over inlining; keeps the recipe readable and the variation surface visible.

## CLI conventions

- One-shot subcommands for every core operation.
- `--format json` for machine-readable output; secrets emitted only with explicit flags (e.g., `entry get --show-password`).
- Stable, documented exit codes: `0` success, `1` user error, `2` vault locked / auth failure, `3` sync conflict requiring user, `10+` internal errors.
- Shell completions for bash, zsh, fish produced by the build.

## Performance targets

- Vault open (Argon2id decrypt + parse, ≤500 entries) <500ms on M1 Mac / mid-tier x86_64 Linux.
- Search across an unlocked vault <50ms for ≤5,000 entries.

## Repo state

**Phase 0 progress (as of 2026-05-17):**

- **Phase 1 — Foundation: complete.** Cargo workspace + four member crates, supply-chain machinery (`deny.toml`, `.cargo/config.toml`, vendored deps, CI workflow), `Makefile`, `CONTRIBUTING.md`, and the no-I/O foundation modules `error::VaultError` / `paths::RunairePaths` / `secret::{MasterPassword, Keyfile}` with their tests. Tasks T1.1–T1.5 all checked.
- **Phase 2 — Storage primitives: complete.** `atomic::write_atomic` + `atomic::write_atomic_with`, `locking::{acquire_exclusive, acquire_shared}` over `std::fs::File`'s 1.89 lock API (zero third-party deps), and the `lock_holder` test-only binary. Tasks T2.1 + T2.2 checked.
- **Phase 3 — Registry: complete.** `VaultRegistry` over `vaults.toml` with `serde(flatten)` forward-compat for unknown top-level and per-vault keys; atomic-write save; three TOML fixtures. Task T3.1 checked.
- **Phase 4 — Vault operations: not started.** Next task is **T4.1 (`unlock` module + `keepass-rs` vendor + version pin)**. See `…/features/vault-core/tasks/04-vault-operations.md`.

See `…/features/vault-core/tasks/task-plan.md` for the rollup view and `…/features/vault-core/tasks/index.md` for the live task-tracking table.

## Shell command style

Prefer running commands as seperate Bash tool calls rather than chaining them with `&&`, `||`, `;`,
or pipes. Each command should be its own invocation so the permission matcher can authorize them
individual.

Exceptions where chaining is fine:

- Pipes that are part of a single logical operation (`grep ... | wc -l`, `cat foo | jq .bar`) - these
  only make sense as one command.

- `cd <dir> && <cmd>` when the directory change must scope to that one command and not persist.

When in doubt, run them separately.
