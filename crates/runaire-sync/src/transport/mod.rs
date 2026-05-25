//! The [`SyncTransport`] trait (FR-046) and its content-addressable types.
//!
//! The trait deliberately exposes *content-addressable* primitives
//! (`local_head`, `remote_head`, `merge_base`, `read_vault_at`,
//! `commit_and_push`) rather than git-specific operations (ADR-002), so
//! Phase-4 transports (NFS, Samba, Drive, iCloud) map their native version
//! concept onto [`ContentVersion`] without faking git semantics.
//!
//! **Phase 1 scaffold.** The trait and its value types are defined here;
//! [`git::GitTransport`] is a stub implemented across Phases 2–3.

pub mod git;

/// An opaque, transport-defined content version identifier.
///
/// For [`git::GitTransport`] this wraps a commit hash; Phase-4 transports may
/// wrap a Drive revision ID, an `mtime`+hash, or a version vector. The trait
/// makes no assumption beyond: `==` defines version equality, and the
/// transport is the sole interpreter of the bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentVersion(pub Vec<u8>);

/// Metadata returned by [`SyncTransport::commit_and_push`].
#[derive(Debug, Clone)]
pub struct PushResult {
    /// The new head version after the push.
    pub new_head: ContentVersion,
}

/// The transport-agnostic sync contract (FR-046).
///
/// **Implementor's promise:** `fetch` is the only networked method;
/// `local_head` / `remote_head` / `merge_base` are stable between fetches;
/// `read_vault_at` returns the exact KDBX bytes committed at a version;
/// `commit_and_push` is atomic from the local side (or returns a
/// push-rejected error).
///
/// **Caller's promise:** never call any method but `fetch` concurrently, and
/// always establish a fetched view of `remote_head` before `commit_and_push`.
///
/// The trait is intentionally *not* object-safe in spirit: the orchestrator is
/// generic over `T: SyncTransport`, so each implementor chooses its own
/// associated [`SyncTransport::Error`] (mapped into [`crate::SyncError`] via an
/// `Into` bound) rather than sharing one enum.
pub trait SyncTransport {
    /// Transport-specific error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Refresh the local mirror of remote state. The only networked method.
    ///
    /// # Errors
    /// Returns [`Self::Error`] on network or protocol failure.
    fn fetch(&mut self) -> Result<(), Self::Error>;

    /// Current local tip of the tracked branch on this device.
    ///
    /// # Errors
    /// Returns [`Self::Error`] if the local ref cannot be read.
    fn local_head(&self) -> Result<ContentVersion, Self::Error>;

    /// Current remote tip as of the most recent successful [`Self::fetch`].
    ///
    /// # Errors
    /// Returns [`Self::Error`] if the remote-tracking ref cannot be read.
    fn remote_head(&self) -> Result<ContentVersion, Self::Error>;

    /// Latest common ancestor of `local` and `remote`; `Ok(None)` when the two
    /// have no common ancestor (orphan branches).
    ///
    /// # Errors
    /// Returns [`Self::Error`] if the ancestry query fails.
    fn merge_base(
        &self,
        local: &ContentVersion,
        remote: &ContentVersion,
    ) -> Result<Option<ContentVersion>, Self::Error>;

    /// Read the complete vault file bytes committed at `version`.
    ///
    /// # Errors
    /// Returns [`Self::Error`] if the version or its vault blob is missing.
    fn read_vault_at(&self, version: &ContentVersion) -> Result<Vec<u8>, Self::Error>;

    /// Create a commit containing `vault_bytes` with `message`, advance the
    /// local head to it, and push to the remote.
    ///
    /// # Errors
    /// Returns [`Self::Error`] (push-rejected variant) when the remote
    /// advanced concurrently; on any error the local state is left unchanged.
    fn commit_and_push(
        &mut self,
        vault_bytes: &[u8],
        message: &str,
    ) -> Result<PushResult, Self::Error>;

    /// Advance the local ref to `version` without creating a commit (used in
    /// fast-forward scenarios).
    ///
    /// # Errors
    /// Returns [`Self::Error`] if the ref update fails.
    fn advance_local_to(&mut self, version: &ContentVersion) -> Result<(), Self::Error>;
}
