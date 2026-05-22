//! Per-subcommand serializable view structs — JSON schema contract.
//!
//! Phase 1 lands the module shape only. Phases 2–4 fill in:
//!
//! - [`vault`]: `VaultCreateView`, `VaultListView`, `VaultOpenView`, `VaultSetLockView`.
//! - [`entry`]: `EntryGetView`, `EntryListView`, `EntrySearchView`, `EntryAddView`, `EntryEditView`, `EntryRmView`.
//! - [`gen`]: `PasswordGenView`, `PassphraseGenView`.

pub mod entry;
pub mod gen;
pub mod vault;
