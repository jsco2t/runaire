//! Rùnaire git-transport sync.
//!
//! This crate owns FR-040..046: per-vault git sync configuration, a
//! transport-agnostic [`SyncTransport`] trait (FR-046), the gix-backed
//! [`GitTransport`], and a two-way KDBX [`merge`] adapter (FR-043) that
//! reconciles a local and remote vault by entry UUID, preserving the loser of
//! a collision as a KDBX history entry. The [`Sync`] orchestrator wires those
//! together.
//!
//! Per ADR-001 the dependency arrow points only into `runaire-core`: this
//! crate consumes [`runaire_core::Vault`] / [`runaire_core::Database`] but
//! `runaire-core` never depends on sync, keeping `gix-*` out of every
//! non-syncing consumer's dependency tree.
//!
//! # Phase status
//!
//! This is the **Phase 1 scaffold**. The [`config`] module (sync config
//! serde + `vaults.toml` integration) is fully implemented; every other
//! public function is a documented `unimplemented!()` stub naming the phase
//! and task that fills it in. See
//! `notebook/.../features/sync-git/tasks/index.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod activity;
pub mod auth;
pub mod backup;
pub mod commit_message;
pub mod config;
pub mod error;
pub mod merge;
pub mod sync;
pub mod transport;

pub use auth::encrypt_credential;
pub use config::{AuthKind, SyncConfig};
pub use error::SyncError;
pub use merge::{reconcile, EntryDelta, MergeError, MergeSummary};
pub use sync::{Sync, SyncOptions, SyncOutcome};
pub use transport::git::GitTransport;
pub use transport::{ContentVersion, PushResult, SyncTransport};
