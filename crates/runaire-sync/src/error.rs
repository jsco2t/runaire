//! The crate's single public error type (design §2.8).
//!
//! No variant ever carries plaintext passwords, decrypted vault contents, or
//! KDBX bytes — the same discipline `runaire-core::VaultError` follows. The
//! enum is `#[non_exhaustive]` so later phases can add variants without a
//! `SemVer` break.

use crate::merge::MergeError;

/// Errors surfaced by the sync layer.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SyncError {
    /// The remote could not be reached (network failure, bad URL, DNS, …).
    #[error("remote unreachable: {url} ({source})")]
    RemoteUnreachable {
        /// The remote URL that could not be reached.
        url: String,
        /// The underlying transport error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Authentication against the remote failed.
    #[error("authentication failed for {url}: {reason}")]
    AuthFailed {
        /// The remote URL the auth attempt targeted.
        url: String,
        /// A non-secret reason string.
        reason: String,
    },

    /// The remote rejected the push because it advanced concurrently. The
    /// caller should re-run sync (the orchestrator retries automatically).
    #[error("push rejected by remote (remote has new commits); retry sync")]
    PushRejected,

    /// The vault directory is not a git working tree yet.
    #[error("vault directory is not a git repository (run `runaire vault set-sync` first)")]
    NotARepository,

    /// Local and remote share no common ancestor (orphan branches).
    #[error("no common ancestor between local and remote (orphan branches)")]
    MissingMergeBase,

    /// The master password or KDF parameters differ across the vaults being
    /// merged, so they cannot be combined.
    #[error("master password or KDF parameters differ between local and remote vault")]
    MasterPasswordMismatch,

    /// A merge could not be completed; the pre-merge state is preserved.
    #[error("merge cannot proceed: {reason}; pre-merge state preserved at {backup_path}")]
    Unresolvable {
        /// A non-secret reason string.
        reason: String,
        /// Path to the preserved `.kdbx.bak` snapshot.
        backup_path: std::path::PathBuf,
    },

    /// Writing the pre-merge `.kdbx.bak` snapshot failed.
    #[error("backup creation failed: {source}")]
    BackupFailed {
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The merge engine returned an error.
    #[error("merge engine error: {0}")]
    Merge(#[from] MergeError),

    /// A vault-layer (`runaire-core`) error occurred.
    #[error("vault error: {0}")]
    Vault(#[from] runaire_core::VaultError),

    /// A git transport error not mapped to a more specific variant.
    #[error("git transport error: {0}")]
    Gix(String),

    /// A stored HTTPS credential could not be decrypted (corrupt container or
    /// wrong master password — the two cases are intentionally indistinct).
    #[error("invalid credential container (corrupt or wrong master password)")]
    CredentialDecryption,
}
