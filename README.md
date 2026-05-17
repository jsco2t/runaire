# Rùnaire

Rùnaire (Scottish Gaelic: _"keeper of secrets"_) is an offline-first secrets
manager built on a Rust core with thin cross-platform UIs. Vaults are stored in
the [KDBX (KeePass) format](https://keepass.info/help/kb/kdbx_4.html) via the
[`keepass-rs`](https://crates.io/crates/keepass) crate, so every Rùnaire vault
is directly interoperable with KeePassXC, KeeWeb, KeePass2, and other
standards-compliant KDBX clients. Default sync transport (Phase 1+) is git,
with in-app three-way merge at the entry level.

**License:** MIT. **Status:** Phase 0 — vault-core and entry-management
implemented; downstream CLI/TUI/sync/security-behaviours/password-generation
features are still in development.

The core crate provides the durability-sensitive vault layer: KDBX4 create,
open, save, rekey, registry management, atomic write replacement, and
cross-process advisory locking — plus, as of entry-management, the entry CRUD
surface, KDBX-native history, groups + tags, free-text + wildcard search,
RFC-6238 TOTP, file attachments, and per-entry expiration. It is intentionally
a small library surface; sync, generated passwords, and user interfaces
consume this crate from later features.

## What works today

`runaire-core` ships the entire offline data layer:

- **Vault lifecycle:** `Vault::create` / `Vault::open` / `Vault::save` / `Vault::change_master_password`, with Argon2id KDF and atomic-write replacement.
- **Vault registry:** `VaultRegistry` over `~/.local/state/runaire/vaults.toml`.
- **Concurrent-access safety:** advisory file locking (`ExclusiveLock` / `SharedLock`); concurrent readers OK, writers serialised.
- **Entry CRUD** with stable KDBX-native UUIDs and automatic history:
  - `Vault::add_entry(group, EntryBuilder::credential|secure_note|totp(...).build())`
  - `Vault::update_entry(uuid, |entry| { entry.set_password("..."); Ok(()) })` — closure-based; appends to history exactly once per call; rolls back on `Err`.
  - `Vault::delete_entry` (honours `RecycleBinEnabled`) / `Vault::purge_entry` (unconditional permanent delete).
  - `Vault::move_entry`, `Vault::get_entry`, `Vault::get_entry_mut`.
- **Groups + tags:** `Vault::create_group` / `rename_group` / `move_group` / `delete_group(behavior)`; `EntryViewMut::add_tag` / `remove_tag` / `set_tags`; `Vault::list_tags()`.
- **Search (FR-014):** `Vault::search(SearchOptions::new("q"))` for case-insensitive substring; `.wildcard(true)` for case-insensitive `*` / `?` matching anchored to the whole field (pad with `*` for "contains").
- **TOTP (FR-015):** `EntryBuilder::totp("Title", otpauth_uri)` + `Vault::totp(uuid) → (code, remaining)`. Phase 0 supports HMAC-SHA1 (RFC 6238 §1.2; every mainstream authenticator app).
- **Attachments (FR-016):** `Vault::add_attachment` / `get_attachment` / `list_attachments` / `remove_attachment`, with a configurable per-attachment size cap (default 5 MiB, max 100 MiB) stored in KDBX `MetaData::custom_data`.
- **Expiration (FR-017):** `Vault::set_expiration(uuid, when)` / `clear_expiration` / `is_expired(uuid, now)`.
- **KDBX interop:** every entry-level feature is verified to round-trip through `keepassxc-cli` ≥ 2.7 — see `tools/interop-tests/entry_us-*.sh`.
- **Crate-level docs:** `cargo doc --no-deps --offline` produces a warning-free API reference with runnable examples for every public surface.

## !!WARNING!! PLEASE READ

This repository is a hobby project of a single person (or at best a few people). The author(s) of
this project wish to make the following **VERY** clear:

- This software is provided "AS IS", without warranty of any kind. (See **License** for further
  clarification)

- This software is not designed as enterprise grade security software (note the _hobby project_
  statement above). Users should not expect any sort of specific quality bar or reliability
  with this software.

- **Use at your own risk**. The authors of this software are in no way responsible for your use of
  the software and/or the security of data you store with this software.

- This software is opinionated in how it works. What that mostly means is that it was designed to
  solve the needs of the original author. It may or may not fit your needs.

## Workspace layout

| Crate           | Purpose                                                      |
| --------------- | ------------------------------------------------------------ |
| `runaire-core`  | Library: KDBX I/O, vault registry, atomic writes, file locks |
| `runaire-cli`   | (Placeholder) one-shot scriptable command-line surface       |
| `runaire-tui`   | (Placeholder) interactive terminal UI (`ratatui`)            |
| `runaire-agent` | (Placeholder) optional long-running unlock agent             |

Only `runaire-core` is implemented in Phase 0; the three binary crates are
stubs reserved for later features.

## Supported platforms

Phase 0 targets macOS and Linux on x86_64 and aarch64. Windows, FreeBSD, and
network filesystems are not supported targets for the vault-core reliability
claims.

## Build from source

`make` is the canonical build interface — every workflow has a target. Run
`make` (or `make help`) to list them. The most common:

```sh
make build              # cargo build --workspace --offline --locked
make test               # default-parallel tests
make check              # fmt-check + lint + build + test — the full local CI gate
make verify             # full core verification: check + ignored tests + docs + supply chain + interop
make interop            # vault-core KeePassXC interop tests; requires keepassxc-cli >= 2.7
make interop-entry      # entry-management KeePassXC interop tests (TOTP cross-checks with oathtool if present)
make bench-search       # informational entry-search benchmark
make bench-search-gate  # NFR-002 gate: fails if search exceeds the latency budget
```

All dependencies are vendored in `vendor/`; offline is the only supported
build mode. From a clean clone, do not run `cargo update`; use the checked-in
`Cargo.lock` and vendored sources. To intentionally add or upgrade a Rust
dependency, follow the vendoring workflow in [CONTRIBUTING.md](CONTRIBUTING.md)
and run `make vendor` as the only networked dependency step.

## Security and supply chain

See [`CLAUDE.md`](CLAUDE.md) for the non-negotiable security and supply-chain
rules: zeroize on drop, no plaintext to disk, atomic writes, permissive-license
deps only, vendored + pinned, no telemetry.

## Planning docs

The product and engineering docs live in the project notebook, not in this
repository:

- PRD: `/home/jason/Developer/sources/personal/notebook/projects/runaire/prd.md`
- Vault-core design:
  `/home/jason/Developer/sources/personal/notebook/projects/runaire/features/vault-core/plans/design.md`
- Vault-core implementation plan:
  `/home/jason/Developer/sources/personal/notebook/projects/runaire/features/vault-core/plans/implementation-plan.md`
- Contributor workflow: [CONTRIBUTING.md](CONTRIBUTING.md)

## AI Coding Policy

This repository leverages AI Coding solutions as part of its development processes. The authors of
this repository recognize that some individuals will find that objectionable. We recognize and
appreciate that perspective and wish such individuals the very best in finding a codebase that
fit's their needs.
