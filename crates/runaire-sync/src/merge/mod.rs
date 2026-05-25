//! Three-way merge engine (FR-043).
//!
//! The merge is a *pure function* over three in-memory [`Database`] values
//! (ADR-005): no I/O, deterministic, and idempotent. This shape makes the
//! four documented invariants (no data loss, determinism, commutativity over
//! disjoint changes, idempotence) testable with `proptest` and exercisable by
//! a fuzz target with no per-iteration setup.
//!
//! **Phase 1 scaffold.** Only the public types and the [`three_way`]
//! signature exist here; the algorithm and its submodules (`entry`, `group`,
//! `metadata`, `delta`) land in Phase 4 (tasks T4.1–T4.5). See design §2.2.7.

use chrono::{DateTime, Utc};
use runaire_core::{Database, Uuid};

/// The merge engine's output.
///
/// No `Debug` derive: [`Database`] is large and its debug output would risk
/// surfacing entry material in logs. The merged database is installed via
/// `Vault::replace_database`; the deltas drive commit-message generation.
pub struct MergedDatabase {
    /// The merged database, ready to install via `Vault::replace_database`.
    pub database: Database,
    /// What changed on the local side relative to the base.
    pub local_delta: EntryDelta,
    /// What changed on the remote side relative to the base.
    pub remote_delta: EntryDelta,
    /// Count of same-entry collisions resolved by the newer-wins rule. Each
    /// collision contributed exactly one loser entry to history.
    pub conflicts: usize,
}

/// Entries added / removed / modified on one side relative to the base.
#[derive(Debug, Default, Clone)]
pub struct EntryDelta {
    /// UUIDs present in the side but not in the base.
    pub added: Vec<Uuid>,
    /// UUIDs present in the base but deleted in the side.
    pub removed: Vec<Uuid>,
    /// UUIDs present in both with changed content.
    pub modified: Vec<Uuid>,
}

/// Errors the merge engine can return.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MergeError {
    /// KDF parameters or master key differ between base, local, or remote.
    #[error("KDF parameters / master-key differ between base, local, or remote")]
    KdfMismatch,
    /// An entry appears as "new" in both local and remote — implies UUID reuse,
    /// which violates the entry-management UUID-stability contract.
    #[error(
        "entry {uuid} appears as 'new' in both local and remote — collision implies UUID reuse"
    )]
    UuidReuse {
        /// The reused entry UUID.
        uuid: Uuid,
    },
    /// A merge-engine internal invariant was violated.
    #[error("merge engine internal invariant violated: {0}")]
    InvariantViolation(String),
}

/// Pure three-way merge over three in-memory databases.
///
/// `now` stamps merged-metadata modification times only; entry timestamps are
/// never overwritten (winners keep theirs; losers placed in history keep
/// theirs). Deterministic and idempotent — see design §2.2.7 / §3.5.
///
/// **Phase 1 stub** — implemented in Phase 4 (T4.5).
///
/// # Errors
///
/// Returns [`MergeError`] when the three databases are not mergeable (KDF
/// mismatch, UUID reuse, or an internal invariant violation).
pub fn three_way(
    _base: &Database,
    _local: &Database,
    _remote: &Database,
    _now: DateTime<Utc>,
) -> Result<MergedDatabase, MergeError> {
    unimplemented!("Phase 4 — task T4.5 (merge::three_way orchestration)")
}
