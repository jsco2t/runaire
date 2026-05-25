//! [`GitTransport`] — the production `gix`-backed [`SyncTransport`].
//!
//! All `gix::*` types are owned by this submodule; nothing outside it imports
//! from `gix`. **Phase 1 scaffold:** this is a fieldless stub. The `gix`
//! dependency, the repository handle, the ref/blob reads, and the
//! fetch/push wiring land in Phases 2–3 (T3.2–T3.4). The `_phantom`-free unit
//! shape keeps the scaffold dependency-free while still satisfying the
//! `Sync<GitTransport>` bound.

use std::path::Path;

use runaire_core::MasterPassword;

use crate::config::SyncConfig;
use crate::error::SyncError;
use crate::transport::{ContentVersion, PushResult, SyncTransport};

/// Production [`SyncTransport`] backed by `gix`.
pub struct GitTransport;

impl GitTransport {
    /// Open (or initialize) `vault_dir` as a git working tree and resolve the
    /// credentials implied by `cfg.auth` using `master_password`.
    ///
    /// **Phase 1 stub** — implemented in Phase 3 (T3.2).
    ///
    /// # Errors
    /// Returns [`SyncError`] if the directory cannot be opened/initialized or
    /// credentials cannot be resolved (Phase 3).
    pub fn open(
        _vault_dir: &Path,
        _cfg: &SyncConfig,
        _master_password: &MasterPassword,
    ) -> Result<Self, SyncError> {
        unimplemented!("Phase 3 — task T3.2 (GitTransport::open)")
    }
}

impl SyncTransport for GitTransport {
    type Error = SyncError;

    fn fetch(&mut self) -> Result<(), Self::Error> {
        unimplemented!("Phase 3 — task T3.3 (GitTransport::fetch)")
    }

    fn local_head(&self) -> Result<ContentVersion, Self::Error> {
        unimplemented!("Phase 3 — task T3.2 (GitTransport::local_head)")
    }

    fn remote_head(&self) -> Result<ContentVersion, Self::Error> {
        unimplemented!("Phase 3 — task T3.2 (GitTransport::remote_head)")
    }

    fn merge_base(
        &self,
        _local: &ContentVersion,
        _remote: &ContentVersion,
    ) -> Result<Option<ContentVersion>, Self::Error> {
        unimplemented!("Phase 3 — task T3.2 (GitTransport::merge_base)")
    }

    fn read_vault_at(&self, _version: &ContentVersion) -> Result<Vec<u8>, Self::Error> {
        unimplemented!("Phase 3 — task T3.2 (GitTransport::read_vault_at)")
    }

    fn commit_and_push(
        &mut self,
        _vault_bytes: &[u8],
        _message: &str,
    ) -> Result<PushResult, Self::Error> {
        unimplemented!("Phase 3 — task T3.4 (GitTransport::commit_and_push)")
    }

    fn advance_local_to(&mut self, _version: &ContentVersion) -> Result<(), Self::Error> {
        unimplemented!("Phase 3 — task T3.4 (GitTransport::advance_local_to)")
    }
}
