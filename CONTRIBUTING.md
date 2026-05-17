# Contributing to Rùnaire

Thanks for your interest. Rùnaire is a personal/OSS project that values
**simplicity over cleverness** and treats KDBX interop, durability, and
supply-chain hygiene as **non-negotiable**. Please read this document — and
[`CLAUDE.md`](CLAUDE.md) — before opening a PR.

Authoritative source documents (read these for scope):

- **Project PRD** and **engineering plans** live in the project notebook
  (paths referenced in [`CLAUDE.md`](CLAUDE.md) — section "Planning docs").
- **Supply-chain policy:** `notebook/.../runaire/kb/supply-chain-policy.md`.
- **Test plan:** the test conventions in this document are the canonical
  summary of Test Plan §8.1 in the vault-core implementation plan.

---

## Workspace layout

```
runaire/
├── Cargo.toml                # workspace manifest, resolver = "2"
├── Cargo.lock                # pinned, source of truth, checked in
├── rust-toolchain.toml       # pinned channel
├── deny.toml                 # cargo-deny config
├── .cargo/config.toml        # offline + vendored-sources
├── vendor/                   # all dependencies, checked in
├── .github/workflows/        # CI
├── crates/
│   ├── runaire-core/         # library — KDBX I/O, registry, atomic, locking
│   ├── runaire-cli/          # placeholder binary
│   ├── runaire-tui/          # placeholder binary
│   └── runaire-agent/        # placeholder binary
└── tools/
    └── interop-tests/        # bash harness, arrives in Phase 6
```

Only `runaire-core` is implemented in Phase 0. The three binary crates are
stubs reserved for later features and must not be deleted (the layout is
part of the workspace contract).

---

## Building and testing

**Everything goes through `make`** — `make` is the build system of record
(see [`CLAUDE.md`](CLAUDE.md) §"Build system"). Run `make` alone (or
`make help`) to list every target. The day-to-day loop:

```sh
make build         # build the workspace (offline, vendored)
make test          # default-parallel tests
make test-ignored  # serial run of env-mutating / fault-injection tests
make fmt           # auto-format
make lint          # clippy with `-D warnings`
make check         # fmt-check + lint + build + test — the full local CI gate
make verify        # check + ignored tests + docs + supply-chain checks + interop
make deny          # cargo-deny (license + advisory + bans)
make vendor        # re-vendor dependencies (the only target that needs network)
```

These targets wrap `cargo` with `--workspace --offline --locked`. A clean
clone produces identical results to CI because `Cargo.lock`, `vendor/`,
`rust-toolchain.toml`, and the `Makefile` itself are all committed and
authoritative.

**Adding a new workflow?** If you add a new lint, test category, code-gen
step, harness, or CI step, add the matching `make` target in the same
change. This is a project rule — see [`CLAUDE.md`](CLAUDE.md)
§"Keeping the Makefile up to date" for the rationale.

---

## Adding a dependency

Every new dependency goes through this five-step workflow (from the
supply-chain policy, Rule 4). Document each step in the PR description.

### 1. License check

The dependency's license **must be one of**: `MIT`, `Apache-2.0`,
`Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`,
`Zlib`, `Unicode-3.0`, `Unicode-DFS-2016`, `Unlicense`, `CC0-1.0`.

The following are **forbidden**, including via transitive deps:
`GPL-*`, `LGPL-*`, `AGPL-*`, `SSPL-*`, `Commons Clause`,
**including GPL-with-linking-exception** (the ambiguity isn't worth it —
e.g., `libgit2` is banned in favor of `gitoxide`).

`cargo deny check licenses` enforces this.

**Clarification — linked vs. invoked test drivers:** GPL software
**linked or loaded** into the build (linked as a C library, loaded via
FFI, statically included) is forbidden by the rules above. GPL software
**invoked as a subprocess in tests** (e.g., the `keepassxc-cli` binary
used for KDBX round-trip verification) is **permitted** — it does not
enter `Cargo.lock` and is not redistributed with the binary. This is
documented to avoid future ambiguity.

### 2. Maintenance signal

Confirm the crate is actively maintained: recent commits, recent
releases, responsive issue tracker, no flood of unaddressed security
advisories. If the crate is dormant but the code is small and obvious,
prefer vendoring with patches over depending on a stale upstream.

### 3. Popularity baseline

The crate should be in widespread use within the Rust ecosystem
(typically: pulled in transitively by `cargo`, `rustup`, popular
runtimes, or major application crates). Lightly-used crates require
extra scrutiny in step 5.

### 4. Vendoring + Cargo.lock review

Add the dependency to `Cargo.toml`, then:

```sh
make vendor   # CARGO_NET_OFFLINE=false cargo vendor — needs network access
```

Inspect the diff in `vendor/` and `Cargo.lock`:

- Confirm only expected new directories appear.
- Skim `Cargo.toml` of each new vendored crate — look for `build.rs`
  scripts that fetch from the network or shell out to external tools.
  **Build-script networking is forbidden.**
- Confirm no banned crates (`openssl-sys`, `git2`, `libgit2-sys`,
  `native-tls`) appear in the dep graph.
- Run `make deny` locally — it must pass.

### 5. PR description checklist

The PR description must include a checklist of:

- [ ] License of each new (direct + transitive) crate — confirmed
      permissive
- [ ] Upstream maintenance status — last release, last commit, issue
      backlog
- [ ] Popularity — download count and notable users
- [ ] Vendored diff reviewed — no surprises
- [ ] `make deny` clean
- [ ] `make build` clean from a fresh clone

CI gates merge on `make deny` (license + advisory + bans) and
`make audit` (RustSec advisories).

---

## Testing conventions

Established by Phase 0 (vault-core T1.4 / T1.5) and inherited by all
later features.

### Framework

- **Rust standard `#[test]`.** No third-party test framework (no
  `rstest`, no `cucumber`).
- **Assertions:** `assert!`, `assert_eq!`, `assert_ne!`, `assert_matches!`
  (stable Rust 1.82+). No `unwrap()` inside tests — use
  `expect("clear failure message")` so failures point at the right line.
  No `?` operator inside tests — let panics print the right location.

### Mocks

- **None.** Vault-core has no mock-worthy boundary; its dependencies
  are the filesystem and `keepass-rs`. Both are used real, in
  `tempfile::TempDir`-isolated sandboxes. Tests that need a "second
  process" (cross-process locking, fault injection) spawn one via
  `std::process::Command` against a tiny in-repo helper binary.

### Layout

- **Unit tests** live in a `#[cfg(test)] mod tests` block at the bottom
  of the module file they exercise.
- **Integration tests** live in `crates/runaire-core/tests/`, one file
  per user scenario, named for the US identifier (`us_001_create_vault.rs`,
  etc.).
- **Shared helpers** live in `crates/runaire-core/tests/common/mod.rs`
  (Cargo convention). Phase 0 establishes [`TestEnv`].
- **Helper binaries** (`lock_holder`, `fault_helper`, `vault_holder`,
  `runaire-test-driver` — Phase 4+) are declared as `[[bin]]` entries
  in `Cargo.toml`. Tests discover them via
  `env!("CARGO_BIN_EXE_<name>")` — the standard Rust idiom — not via
  `cargo run --bin`.

### Table-driven tests

Where multiple inputs exercise the same behavior, use a
`cases: &[Case]` slice with a loop. Each case has a
`name: &'static str` field; assertion messages include the case name so
failures are unambiguous.

### Cleanup

- **Every test that touches the filesystem uses `tempfile::TempDir`**
  (or `NamedTempFile`); the directory is auto-deleted on drop.
- **No test writes to `$HOME`, `/tmp` directly, or any path resolved
  from the user's real environment.** The `TestEnv` helper enforces
  this by constructing a `RunairePaths` whose `state_dir` is inside the
  tempdir.

### `unsafe` in tests

Two specific test categories require `unsafe`:

1. **Zeroize verification** — `std::ptr::read_volatile` to inspect the
   underlying buffer of a `MasterPassword` / `Keyfile::Bytes` after
   calling `.zeroize()`. One test per sensitive type, with a documented
   block-level SAFETY comment.
2. **Environment mutation** — `std::env::set_var` / `remove_var` are
   `unsafe` in Rust 1.85+ due to libc-getenv thread-safety semantics.
   Tests that mutate env vars use an `EnvGuard` RAII helper that
   serializes via a module-local `Mutex<()>`.

Production code in `runaire-core` is `#![cfg_attr(not(test), forbid(unsafe_code))]`.
The two test exceptions above are the only `unsafe` allowed in the
crate at Phase 0. `mlock` of the master key — the only other
candidate for `unsafe` — is deferred (design §3.9).

---

## Style

- `make fmt-check` is enforced by CI (wraps `cargo fmt --all --check`).
- `make lint` is enforced by CI (wraps
  `cargo clippy --workspace --all-targets -- -D warnings`). The
  workspace allows `pedantic` Clippy as a warning baseline; three lint
  families (`module_name_repetitions`, `must_use_candidate`,
  `missing_errors_doc`) are allowed because they fire on idiomatic API
  shapes.
- Public items in `runaire-core` have rustdoc comments. CI runs
  `make doc` (wraps `cargo doc --no-deps --offline`) and (later) treats
  warnings as errors.

---

## Related documents

- [`CLAUDE.md`](CLAUDE.md) — non-negotiable rules (security, supply
  chain, technical constraints).
- [`README.md`](README.md) — project overview.
- **PRD** and **engineering plans** — referenced from `CLAUDE.md`.
- **Supply-chain policy:** `notebook/.../runaire/kb/supply-chain-policy.md`.
- **Test plan:** Test Plan §8.1 in
  `notebook/.../features/vault-core/plans/implementation-plan.md`.
