//! Pre-merge `.kdbx.bak` snapshot (design §2.2.10).
//!
//! Before any merge work the orchestrator copies the live `.kdbx` to a
//! sibling `.kdbx.bak` via `runaire_core::atomic::write_atomic`, so a bad
//! merge is always recoverable (`cp my.kdbx.bak my.kdbx`). The copied bytes
//! are already-encrypted KDBX — no plaintext is involved.
//!
//! **Phase 1 scaffold:** signature only; implemented in Phase 5 (T5.1).

use std::path::{Path, PathBuf};

use crate::error::SyncError;

/// Atomically copy `vault_path` to its sibling `*.kdbx.bak`, overwriting any
/// stale backup from a prior interrupted sync. Returns the backup path.
///
/// **Phase 1 stub** — implemented in Phase 5 (T5.1).
///
/// # Errors
/// Returns [`SyncError::BackupFailed`] on I/O failure (Phase 5).
pub fn snapshot_pre_merge(_vault_path: &Path) -> Result<PathBuf, SyncError> {
    unimplemented!("Phase 5 — task T5.1 (backup::snapshot_pre_merge)")
}
