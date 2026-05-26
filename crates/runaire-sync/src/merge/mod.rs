//! Two-way KDBX merge adapter (FR-043).
//!
//! # Why an adapter, not a hand-written engine
//!
//! `keepass-rs` ships a UUID-keyed, timestamp-ordered database merge
//! (`Database::merge`, gated behind its `_merge` feature, which
//! `runaire-core` enables). It already implements the FR-043 reconciliation
//! we need — entry identity by UUID, newer-`last_modification` wins, and the
//! loser of a collision preserved as a KDBX history entry under the same UUID
//! — against the crate's real (flat, id-keyed) data model. Re-deriving that
//! against the same data structures is impossible from this crate: every
//! UUID-preserving insert API (`Group::add_entry_with_id`, `EntryId::from_uuid`)
//! is `pub(crate)`. So we wrap the upstream merge behind the single chokepoint
//! [`reconcile`] and pin its observable behaviour with a defensive
//! characterization suite (`tests/merge_semantics.rs`).
//!
//! This supersedes the original three-way design (ADR-005); see design
//! **ADR-008** and `kb/three-way-merge.md`.
//!
//! # No-data-loss guarantee
//!
//! [`reconcile`] folds `remote` into `local` in place. `Database::merge` alone
//! is asymmetric — it preserves the *destination*'s loser as history only when
//! that entry already carries history, and never preserves the *source*'s
//! losing current value. Because a silently dropped secret version is often
//! unrecoverable, [`reconcile`] adds a defensive backfill pass that enforces a
//! single invariant: **every pre-merge entry version, from either side, that is
//! not the merged current value survives as a history entry under the same
//! UUID.** Collisions are still resolved by newer `last_modification` (the
//! winner becomes current); the loser — whichever side it is, and regardless of
//! whether it carried prior history — is recovered into history. The backfill
//! is idempotent (a version already present is detected and not re-added, so
//! repeated syncs do not grow history).
//!
//! ## Remaining Phase-0 limitations (acceptable; tracked as follow-ups)
//!
//! - **Attachments do not merge.** Upstream replaces a winning entry's fields
//!   but leaves attachments untouched (`merge.rs` has an explicit `TODO`), so
//!   an attachment-only edit does not propagate to the merged current value.
//!   Attachment data is not lost — the losing side's full entry (attachments
//!   included) is captured in history by the backfill. (The backfill's
//!   content comparison cannot inspect the attachment pool — the relevant
//!   fields are `pub(crate)` — so attachment-bearing entries may gain an extra
//!   history entry; this fails safe, i.e. it over-preserves, never drops.)
//! - **Same-second divergence is unresolvable.** Two devices editing the same
//!   entry within KDBX's one-second timestamp granularity, with differing
//!   content, cannot be auto-ordered — surfaced as [`MergeError::Unresolvable`]
//!   with the pre-merge `.kdbx.bak` preserved. Vanishingly rare for a single
//!   user; fails safe (no merge is written).
//!
//! Every behaviour above is asserted in the characterization suite
//! (`tests/merge_semantics.rs`) so a future `keepass-rs` bump that changes it
//! fails CI loudly.

use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use runaire_core::{Database, Entry, GroupRef, Times, Uuid};

/// Entries added / removed / modified on `local` by a [`reconcile`] call.
#[derive(Debug, Default, Clone)]
pub struct EntryDelta {
    /// UUIDs present after the merge but not before (folded in from `remote`).
    pub added: Vec<Uuid>,
    /// UUIDs present before the merge but not after (removed via a `remote`
    /// deletion tombstone).
    pub removed: Vec<Uuid>,
    /// UUIDs present both before and after whose `last_modification` changed
    /// (i.e. `remote` won a contested entry).
    pub modified: Vec<Uuid>,
}

impl EntryDelta {
    /// `true` when the merge changed no entries on `local`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }
}

/// Outcome of a successful [`reconcile`].
#[derive(Debug, Default, Clone)]
pub struct MergeSummary {
    /// How `local` changed as a result of folding in `remote`.
    pub delta: EntryDelta,
}

/// Errors returned by [`reconcile`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MergeError {
    /// The two vault snapshots diverged in a way the merge cannot
    /// auto-resolve. In practice this is the same entry edited on two devices
    /// within KDBX's one-second timestamp granularity, leaving differing
    /// content with identical modification times. The orchestrator preserves
    /// the pre-merge state (`.kdbx.bak`) and surfaces this to the user.
    #[error("merge cannot be auto-resolved: {reason}")]
    Unresolvable {
        /// Non-secret description of the conflict. Carries only entry/group
        /// UUIDs and timing facts — never field values.
        reason: String,
    },
}

/// Fold `remote` into `local` in place, UUID-keyed and timestamp-ordered.
///
/// On success `local` contains the union of both sides' entries, with
/// per-entry collisions resolved by newer `last_modification` (loser preserved
/// as history, subject to the preconditions in the module docs). `remote` is
/// left unchanged.
///
/// # Errors
///
/// Returns [`MergeError::Unresolvable`] when the underlying merge cannot decide
/// a winner (same-second divergence). `local` may be left partially modified on
/// error, which is why the orchestrator always snapshots `.kdbx.bak` before
/// calling this.
pub fn reconcile(local: &mut Database, remote: &Database) -> Result<MergeSummary, MergeError> {
    let before = entry_times(local);

    // Snapshot both sides' live entries *before* the merge, so we can enforce
    // the no-data-loss invariant afterwards (see `backfill_lost_versions`).
    let local_pre = collect_live_entries(local);
    let remote_pre = collect_live_entries(remote);

    // Relies on entry-management's UUID-stability contract (US-011): entries
    // never silently regenerate their UUIDs, so UUID identity equals
    // logical-entry identity for the merge. The upstream `Database::merge`
    // (keepass `_merge` feature) performs the reconciliation; we only map its
    // error, enforce no-data-loss, and summarize the change.
    local.merge(remote).map_err(|e| MergeError::Unresolvable {
        reason: e.to_string(),
    })?;

    // Defensive no-data-loss pass. `Database::merge` is asymmetric: it
    // preserves the *destination*'s losing value as history only when that
    // entry already carries history, and never preserves the *source*'s losing
    // current value at all. For a secrets manager a silently dropped version is
    // often unrecoverable, so we enforce the invariant directly: every
    // pre-merge entry version (from either side) that is not the merged current
    // value must survive as a history entry under the same UUID.
    backfill_lost_versions(local, &local_pre);
    backfill_lost_versions(local, &remote_pre);

    let after = entry_times(local);
    Ok(MergeSummary {
        delta: delta_between(&before, &after),
    })
}

/// Clone every live entry reachable from the database root. Used to capture a
/// side's pre-merge state for the no-data-loss backfill.
fn collect_live_entries(db: &Database) -> Vec<Entry> {
    fn walk(group: &GroupRef<'_>, out: &mut Vec<Entry>) {
        for entry in group.entries() {
            out.push((*entry).clone());
        }
        for sub in group.groups() {
            walk(&sub, out);
        }
    }
    let mut out = Vec::new();
    walk(&db.root(), &mut out);
    out
}

/// For each `pre`-merge entry, if the merged database still holds that UUID but
/// its current value differs from `pre` and `pre`'s version is not already in
/// the merged entry's history, append `pre` to history. This recovers any
/// losing version that `Database::merge` dropped, in either direction.
///
/// Entries absent from the merged database (e.g. resolved deletions via a
/// remote tombstone) are intentionally skipped — those are deletions, not
/// dropped content. Idempotent: a version already preserved is detected by the
/// dedup check and not re-added, so repeated syncs do not grow history.
fn backfill_lost_versions(local: &mut Database, pre_entries: &[Entry]) {
    for pre in pre_entries {
        let id = pre.id();
        let should_backfill = match local.entry(id) {
            Some(merged) => content_diverged(&merged, pre) && !history_has_version(&merged, pre),
            None => false,
        };
        if should_backfill {
            if let Some(mut merged) = local.entry_mut(id) {
                merged
                    .history
                    .get_or_insert_default()
                    .add_entry(pre.clone());
            }
        }
    }
}

/// An entry with its `times` and `history` cleared, for content-only equality.
/// `id` and `parent` are retained, so a group move (parent change) counts as a
/// genuine content change worth preserving.
fn content_only(entry: &Entry) -> Entry {
    let mut copy = entry.clone();
    copy.times = Times::default();
    copy.history = None;
    copy
}

/// Whether two entries differ in content (ignoring timestamps and history).
fn content_diverged(a: &Entry, b: &Entry) -> bool {
    content_only(a) != content_only(b)
}

/// Whether `candidate`'s version is already present in `entry`'s history,
/// matched by both `last_modification` and content (second-precision
/// timestamps mean two same-second-but-different versions can legitimately
/// coexist, so neither key alone is sufficient).
fn history_has_version(entry: &Entry, candidate: &Entry) -> bool {
    entry.history.as_ref().is_some_and(|history| {
        history.get_entries().iter().any(|past| {
            past.times.last_modification == candidate.times.last_modification
                && content_only(past) == content_only(candidate)
        })
    })
}

/// Collect every entry's UUID → `last_modification` time by walking the group
/// tree through the public read API. Used to summarize what a merge changed.
fn entry_times(db: &Database) -> BTreeMap<Uuid, Option<NaiveDateTime>> {
    fn walk(group: &GroupRef<'_>, out: &mut BTreeMap<Uuid, Option<NaiveDateTime>>) {
        for entry in group.entries() {
            out.insert(entry.id().uuid(), entry.times.last_modification);
        }
        for sub in group.groups() {
            walk(&sub, out);
        }
    }
    let mut out = BTreeMap::new();
    walk(&db.root(), &mut out);
    out
}

/// Classify the difference between two UUID → time snapshots of `local`.
fn delta_between(
    before: &BTreeMap<Uuid, Option<NaiveDateTime>>,
    after: &BTreeMap<Uuid, Option<NaiveDateTime>>,
) -> EntryDelta {
    let mut delta = EntryDelta::default();
    for (uuid, after_time) in after {
        match before.get(uuid) {
            None => delta.added.push(*uuid),
            Some(before_time) if before_time != after_time => delta.modified.push(*uuid),
            Some(_) => {}
        }
    }
    for uuid in before.keys() {
        if !after.contains_key(uuid) {
            delta.removed.push(*uuid);
        }
    }
    delta
}
