//! The sync orchestrator: [`Sync`], [`SyncOptions`], and [`SyncOutcome`].
//!
//! [`Sync`] is generic over `T: SyncTransport` so the fetch/merge/push state
//! machine never sees git-specific types; the only production transport is
//! [`GitTransport`]. **Phase 1 scaffold:** the public surface is declared
//! here, with the `sync_now` state machine and the `configure_remote` /
//! `clone_from` flows landing in Phase 5 (T5.3–T5.4).

use std::path::Path;

use runaire_core::{MasterPassword, Vault, VaultRegistry};

use crate::config::{AuthKind, SyncConfig};
use crate::error::SyncError;
use crate::transport::git::GitTransport;
use crate::transport::SyncTransport;

/// The sync orchestrator. Wires a [`SyncTransport`] to the merge engine.
pub struct Sync<T: SyncTransport> {
    // Populated by `new_git` (Phase 3) and consumed by `sync_now` (Phase 5).
    // Held now so the public type is stable; allowed dead until the orchestrator
    // body lands.
    #[allow(dead_code)]
    transport: T,
    #[allow(dead_code)]
    config: SyncConfig,
}

/// The outcome of a successful [`Sync::sync_now`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SyncOutcome {
    /// Local and remote already agree; nothing to do.
    AlreadyInSync,
    /// Local was strictly behind; fast-forwarded to the remote tip.
    FastForwarded {
        /// Number of commits the local ref advanced by.
        ahead_by: usize,
    },
    /// Local was strictly ahead; pushed local commits to the remote.
    Pushed {
        /// Number of commits pushed.
        ahead_by: usize,
    },
    /// Both sides diverged; a three-way merge was performed and pushed.
    Merged {
        /// Same-entry collisions resolved (each preserved a loser in history).
        conflicts: usize,
        /// Push retries consumed before the push succeeded.
        retries: u8,
    },
}

/// Options controlling a [`Sync::sync_now`] call.
///
/// **Deferred field:** design §2.2.11 also carries
/// `activity: Option<&AutoLockController>` to suppress idle auto-lock during
/// long syncs. It is intentionally *not* present in the Phase 1 scaffold:
/// `runaire-security`'s `register_activity(&mut self, …)` takes `&mut self`,
/// so whether the controller belongs on this struct (forcing a lifetime
/// parameter) or as a `sync_now` argument must be settled when the
/// activity-pinger is wired (T5.2/T5.3) — not guessed here. Adding it now
/// would couple this scaffold to `runaire-security` and bake in a shared-`&`
/// shape the live API rejects. See the `activity` module's notes.
pub struct SyncOptions {
    /// Maximum push retries on [`SyncError::PushRejected`]. Defaults to 3
    /// (design §3.4).
    pub max_push_retries: u8,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            max_push_retries: 3,
        }
    }
}

impl Sync<GitTransport> {
    /// Construct a git-backed orchestrator for the vault directory.
    ///
    /// **Phase 1 stub** — implemented alongside `GitTransport` in Phase 3.
    ///
    /// # Errors
    /// Returns [`SyncError`] if the git working tree cannot be opened or the
    /// credentials cannot be resolved (Phase 3).
    pub fn new_git(
        _vault_dir: &Path,
        _cfg: SyncConfig,
        _master_password: &MasterPassword,
    ) -> Result<Self, SyncError> {
        unimplemented!("Phase 3 — Sync::new_git (GitTransport wiring)")
    }
}

impl<T: SyncTransport> Sync<T>
where
    T::Error: Into<SyncError>,
{
    /// Configure or change the remote for a vault, initializing the vault
    /// directory as a git working tree if needed, and persisting the
    /// [`SyncConfig`] into `vaults.toml`.
    ///
    /// **Phase 1 stub** — implemented in Phase 5 (T5.4).
    ///
    /// # Errors
    /// Returns [`SyncError`] on git or registry failure (Phase 5).
    pub fn configure_remote(
        _vault: &mut Vault,
        _registry: &mut VaultRegistry,
        _vault_name: &str,
        _remote_url: String,
        _auth: AuthKind,
        _branch: Option<String>,
    ) -> Result<(), SyncError> {
        unimplemented!("Phase 5 — task T5.4 (Sync::configure_remote)")
    }

    /// Clone an existing remote into a new local vault directory.
    ///
    /// **Phase 1 stub** — implemented in Phase 5 (T5.4).
    ///
    /// # Errors
    /// Returns [`SyncError`] on clone failure (Phase 5).
    pub fn clone_from(
        _url: &str,
        _local_dir: &Path,
        _auth: AuthKind,
        _branch: Option<String>,
    ) -> Result<(), SyncError> {
        unimplemented!("Phase 5 — task T5.4 (Sync::clone_from)")
    }

    /// The main operation: fetch, compare, possibly merge, and push.
    ///
    /// **Phase 1 stub** — implemented in Phase 5 (T5.3).
    ///
    /// # Errors
    /// Returns [`SyncError`] on any fetch/merge/push failure (Phase 5).
    pub fn sync_now(
        _vault: &mut Vault,
        _registry: &VaultRegistry,
        _password: &MasterPassword,
        _opts: SyncOptions,
    ) -> Result<SyncOutcome, SyncError> {
        unimplemented!("Phase 5 — task T5.3 (Sync::sync_now)")
    }
}
