//! Characterization tests for the two-way merge adapter (`merge::reconcile`).
//!
//! `reconcile` delegates to `keepass-rs`'s `Database::merge` (the `_merge`
//! feature, enabled in `runaire-core`). Per design ADR-008 we deliberately do
//! NOT own that algorithm — but we DO own a contract with it. These tests pin
//! every merge behaviour Rùnaire relies on, so that an upgrade of the pinned
//! `keepass-rs` version which changes merge semantics fails CI loudly instead
//! of silently altering how vaults reconcile. Each test documents the FR-043
//! property (or the documented Phase-0 limitation) it locks.
//!
//! All databases are built through the public API. Entries live at the root
//! group; timestamps are set explicitly so the tests are deterministic and do
//! not depend on wall-clock granularity. `EntryId` is not re-exported by
//! `runaire-core`, so it is always obtained by inference (`entry.id()`) and
//! never named.

use chrono::NaiveDateTime;
use runaire_core::{fields, Database, GroupRef, Uuid};
use runaire_sync::merge::{reconcile, MergeError};

/// A deterministic timestamp `secs` seconds past a fixed base instant.
fn t(secs: i64) -> NaiveDateTime {
    chrono::DateTime::from_timestamp(1_700_000_000 + secs, 0)
        .expect("valid timestamp")
        .naive_utc()
}

/// Create a database with a single root-level entry titled `title` whose
/// `last_modification` is `at`. Returns the database and the entry's UUID.
fn db_with_entry(title: &str, at: NaiveDateTime) -> (Database, Uuid) {
    let mut db = Database::new();
    let uuid = {
        let mut root = db.root_mut();
        let mut entry = root.add_entry();
        entry.set_unprotected(fields::TITLE, title);
        entry.times.last_modification = Some(at);
        entry.id().uuid()
    };
    (db, uuid)
}

/// Set a root-level entry's title with history tracking (the prior value is
/// pushed to history, as Rùnaire's real edits do), then pin
/// `last_modification` to `at` for deterministic ordering.
fn edit_tracked(db: &mut Database, uuid: Uuid, title: &str, at: NaiveDateTime) {
    // `id` is the keepass `EntryId` (not nameable from this crate), kept as an
    // inferred concrete type so it can be passed to `entry_mut`. The immutable
    // `root()` borrow ends before the mutable `entry_mut` borrows begin.
    let id = db
        .root()
        .entries()
        .find(|e| e.id().uuid() == uuid)
        .map(|e| e.id())
        .expect("entry exists");
    db.entry_mut(id)
        .expect("entry exists")
        .edit_tracking(|e| e.set_unprotected(fields::TITLE, title));
    db.entry_mut(id)
        .expect("entry exists")
        .times
        .last_modification = Some(at);
}

/// Set a root-level entry's (protected) password with history tracking, then
/// pin `last_modification` to `at`.
fn edit_password_tracked(db: &mut Database, uuid: Uuid, password: &str, at: NaiveDateTime) {
    let id = db
        .root()
        .entries()
        .find(|e| e.id().uuid() == uuid)
        .map(|e| e.id())
        .expect("entry exists");
    db.entry_mut(id)
        .expect("entry exists")
        .edit_tracking(|e| e.set_protected(fields::PASSWORD, password));
    db.entry_mut(id)
        .expect("entry exists")
        .times
        .last_modification = Some(at);
}

/// Set a root-level entry's title WITHOUT history tracking (no prior value is
/// recorded), then pin `last_modification` to `at`.
fn edit_untracked(db: &mut Database, uuid: Uuid, title: &str, at: NaiveDateTime) {
    let id = db
        .root()
        .entries()
        .find(|e| e.id().uuid() == uuid)
        .map(|e| e.id())
        .expect("entry exists");
    let mut entry = db.entry_mut(id).expect("entry exists");
    entry.set_unprotected(fields::TITLE, title);
    entry.times.last_modification = Some(at);
}

// --- read helpers (UUID-keyed, so they survive the merge) ----------------

/// The current value of field `field` on the entry with `uuid`. Returns `None`
/// when the entry is absent; an entry present without the field reads as `""`.
fn current_field(db: &Database, uuid: Uuid, field: &str) -> Option<String> {
    fn walk(group: &GroupRef<'_>, uuid: Uuid, field: &str) -> Option<String> {
        for entry in group.entries() {
            if entry.id().uuid() == uuid {
                return Some(entry.get(field).unwrap_or_default().to_string());
            }
        }
        for sub in group.groups() {
            if let Some(found) = walk(&sub, uuid, field) {
                return Some(found);
            }
        }
        None
    }
    walk(&db.root(), uuid, field)
}

/// The current title of the entry with `uuid`, or `None` if absent.
fn current_title(db: &Database, uuid: Uuid) -> Option<String> {
    current_field(db, uuid, fields::TITLE)
}

/// The values of field `field` recorded in the history of the entry with `uuid`.
fn history_field_values(db: &Database, uuid: Uuid, field: &str) -> Vec<String> {
    fn walk(group: &GroupRef<'_>, uuid: Uuid, field: &str) -> Option<Vec<String>> {
        for entry in group.entries() {
            if entry.id().uuid() == uuid {
                let values = entry
                    .history
                    .as_ref()
                    .map(|h| {
                        h.get_entries()
                            .iter()
                            .map(|he| he.get(field).unwrap_or_default().to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                return Some(values);
            }
        }
        for sub in group.groups() {
            if let Some(found) = walk(&sub, uuid, field) {
                return Some(found);
            }
        }
        None
    }
    walk(&db.root(), uuid, field).unwrap_or_default()
}

/// The titles recorded in the history of the entry with `uuid`.
fn history_titles(db: &Database, uuid: Uuid) -> Vec<String> {
    history_field_values(db, uuid, fields::TITLE)
}

// --- FR-043 / limitation characterization tests --------------------------

#[test]
fn remote_only_entry_is_added_under_its_own_uuid() {
    // FR-043: a UUID present only on the remote is folded into local under the
    // SAME UUID (UUID stability — entries are never re-keyed by the merge).
    let (base, shared_uuid) = db_with_entry("shared", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    let new_uuid = {
        let mut root = remote.root_mut();
        let mut e = root.add_entry();
        e.set_unprotected(fields::TITLE, "remote-new");
        e.times.last_modification = Some(t(5));
        e.id().uuid()
    };

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(
        current_title(&local, new_uuid).as_deref(),
        Some("remote-new")
    );
    assert!(summary.delta.added.contains(&new_uuid));
    // The pre-existing shared entry is undisturbed by the addition.
    assert_eq!(
        current_title(&local, shared_uuid).as_deref(),
        Some("shared")
    );
}

#[test]
fn remote_only_entry_in_a_subgroup_is_added() {
    // Exercises the recursive delta walk: an entry added to a remote-only
    // subgroup must be reported in `delta.added` and present after the merge.
    let (base, _) = db_with_entry("shared", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    let sub_entry_uuid = {
        let mut root = remote.root_mut();
        let mut group = root.add_group();
        group.name = "subgroup".into();
        let mut e = group.add_entry();
        e.set_unprotected(fields::TITLE, "in-subgroup");
        e.times.last_modification = Some(t(5));
        e.id().uuid()
    };

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(
        current_title(&local, sub_entry_uuid).as_deref(),
        Some("in-subgroup"),
        "an entry in a remote-only subgroup must be folded in"
    );
    assert!(
        summary.delta.added.contains(&sub_entry_uuid),
        "the subgroup entry must be reported as added (asserts the recursive delta walk)"
    );
}

#[test]
fn local_only_entry_is_preserved() {
    // FR-043 no-data-loss: an entry that exists only on local (never seen by
    // remote, no remote tombstone) is kept untouched.
    let (base, _) = db_with_entry("shared", t(0));
    let mut local = base.clone();
    let remote = base.clone();

    let local_uuid = {
        let mut root = local.root_mut();
        let mut e = root.add_entry();
        e.set_unprotected(fields::TITLE, "local-new");
        e.times.last_modification = Some(t(5));
        e.id().uuid()
    };

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(
        current_title(&local, local_uuid).as_deref(),
        Some("local-new")
    );
    assert!(
        summary.delta.is_empty(),
        "merging in an older remote changes nothing on local"
    );
}

#[test]
fn remote_newer_wins_and_local_loser_is_preserved_in_history() {
    // FR-043 collision rule: same UUID edited on both sides; remote's edit is
    // newer, so remote wins and local's prior value is preserved as history.
    // Edits are history-tracked, matching Rùnaire's real edit path.
    let (base, uuid) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    edit_tracked(&mut local, uuid, "local-edit", t(10));
    edit_tracked(&mut remote, uuid, "remote-edit", t(20));

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(current_title(&local, uuid).as_deref(), Some("remote-edit"));
    let history = history_titles(&local, uuid);
    assert!(
        history.contains(&"local-edit".to_string()),
        "the losing local value must be preserved as history; got {history:?}"
    );
    assert!(summary.delta.modified.contains(&uuid));
}

#[test]
fn disjoint_edits_to_two_entries_both_survive() {
    // FR-043 headline property (US-043): non-overlapping edits merge cleanly.
    // Local edits entry A; remote edits a different entry B. After the merge,
    // local must reflect BOTH edits.
    let mut db = Database::new();
    let (uuid_a, uuid_b) = {
        let mut root = db.root_mut();
        let a = {
            let mut e = root.add_entry();
            e.set_unprotected(fields::TITLE, "A-v0");
            e.times.last_modification = Some(t(0));
            e.id().uuid()
        };
        let b = {
            let mut e = root.add_entry();
            e.set_unprotected(fields::TITLE, "B-v0");
            e.times.last_modification = Some(t(0));
            e.id().uuid()
        };
        (a, b)
    };
    let mut local = db.clone();
    let mut remote = db.clone();

    edit_tracked(&mut local, uuid_a, "A-local", t(10));
    edit_tracked(&mut remote, uuid_b, "B-remote", t(10));

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(
        current_title(&local, uuid_a).as_deref(),
        Some("A-local"),
        "local's own edit to A is retained"
    );
    assert_eq!(
        current_title(&local, uuid_b).as_deref(),
        Some("B-remote"),
        "remote's edit to B is folded in"
    );
    assert!(
        summary.delta.modified.contains(&uuid_b),
        "B is reported as modified by the merge"
    );
    assert!(
        !summary.delta.modified.contains(&uuid_a),
        "A was already current on local; the merge does not re-touch it"
    );
}

#[test]
fn winner_replaces_secret_field_and_loser_secret_is_kept_in_history() {
    // For a secrets manager the load-bearing property is that the PASSWORD
    // follows the merge winner and the loser's password is preserved in
    // history — asserting only on TITLE would miss a regression that mangled
    // protected fields.
    let mut db = Database::new();
    let uuid = {
        let mut root = db.root_mut();
        let mut e = root.add_entry();
        e.set_unprotected(fields::TITLE, "site");
        e.set_protected(fields::PASSWORD, "pw-v0");
        e.times.last_modification = Some(t(0));
        e.id().uuid()
    };
    let mut local = db.clone();
    let mut remote = db.clone();

    edit_password_tracked(&mut local, uuid, "pw-local", t(10));
    edit_password_tracked(&mut remote, uuid, "pw-remote", t(20));

    reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(
        current_field(&local, uuid, fields::PASSWORD).as_deref(),
        Some("pw-remote"),
        "the winner's password must replace the loser's"
    );
    assert!(
        history_field_values(&local, uuid, fields::PASSWORD).contains(&"pw-local".to_string()),
        "the losing local password must be preserved in history"
    );
}

#[test]
fn local_newer_keeps_local_value() {
    // FR-043: same UUID, local's edit is newer, so local wins and keeps its
    // value (remote's older edit loses).
    let (base, uuid) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    edit_tracked(&mut local, uuid, "local-edit", t(20));
    edit_tracked(&mut remote, uuid, "remote-edit", t(10));

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(current_title(&local, uuid).as_deref(), Some("local-edit"));
    assert!(
        summary.delta.is_empty(),
        "local already holds the winning value, so the merge changes nothing on local"
    );
}

#[test]
fn local_newer_retains_remote_loser_in_history_via_backfill() {
    // No-data-loss: when local wins, `Database::merge` alone drops remote's
    // losing current value. `reconcile`'s backfill recovers it into history.
    // (This is the data-loss gap the backfill exists to close.)
    let (base, uuid) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    edit_tracked(&mut local, uuid, "local-edit", t(20));
    edit_tracked(&mut remote, uuid, "remote-edit", t(10));

    reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(current_title(&local, uuid).as_deref(), Some("local-edit"));
    let history = history_titles(&local, uuid);
    assert!(
        history.contains(&"remote-edit".to_string()),
        "remote's losing value must be recovered into history by the backfill; got {history:?}"
    );
}

#[test]
fn first_collision_without_prior_history_still_preserves_loser_via_backfill() {
    // No-data-loss without prior history: `Database::merge` preserves a loser
    // only if it already carried history, so a first collision on an untracked
    // entry would drop the loser. The backfill recovers it regardless — here
    // remote wins and local's history-less losing value is still preserved.
    let (base, uuid) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    edit_untracked(&mut local, uuid, "local-edit", t(10));
    edit_untracked(&mut remote, uuid, "remote-edit", t(20));

    reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(current_title(&local, uuid).as_deref(), Some("remote-edit"));
    assert!(
        history_titles(&local, uuid).contains(&"local-edit".to_string()),
        "the history-less losing value must still be recovered by the backfill"
    );
}

#[test]
fn reconcile_is_idempotent_after_backfill() {
    // A second identical reconcile after a backfill-triggering merge must be a
    // no-op: no duplicate history, no further change. (Backfill dedup canary.)
    let (base, uuid) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    edit_tracked(&mut local, uuid, "local-edit", t(20));
    edit_tracked(&mut remote, uuid, "remote-edit", t(10));

    reconcile(&mut local, &remote).expect("first merge");
    let after_first = local.clone();

    let summary = reconcile(&mut local, &remote).expect("second merge");

    assert!(
        summary.delta.is_empty(),
        "a repeated reconcile reports no further changes"
    );
    assert_eq!(
        local, after_first,
        "a repeated reconcile is a no-op (no duplicate backfilled history)"
    );
}

#[test]
fn repeated_reconcile_keeps_history_length_stable() {
    // Stronger idempotence canary: three reconciles must not grow history past
    // the first. Catches a dedup bug that would accumulate history every sync.
    let (base, uuid) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    edit_tracked(&mut local, uuid, "local-edit", t(20));
    edit_tracked(&mut remote, uuid, "remote-edit", t(10));

    reconcile(&mut local, &remote).expect("merge 1");
    let len_after_first = history_titles(&local, uuid).len();
    reconcile(&mut local, &remote).expect("merge 2");
    reconcile(&mut local, &remote).expect("merge 3");

    assert_eq!(
        history_titles(&local, uuid).len(),
        len_after_first,
        "history must not grow on repeated syncs"
    );
}

#[test]
fn same_second_divergence_is_unresolvable() {
    // FR-043 conflict path: identical modification timestamps but divergent
    // content cannot be auto-resolved. The orchestrator turns this into a
    // user-visible Unresolvable error with `.kdbx.bak` preserved.
    let (base, uuid) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    edit_tracked(&mut local, uuid, "local-edit", t(10));
    edit_tracked(&mut remote, uuid, "remote-edit", t(10));

    let result = reconcile(&mut local, &remote);
    assert!(
        matches!(result, Err(MergeError::Unresolvable { .. })),
        "same-second divergence must be Unresolvable; got {result:?}"
    );
}

#[test]
fn remote_deletion_tombstone_removes_local_entry() {
    // FR-043: an entry deleted on remote (after its last local modification)
    // is removed from local on merge.
    let (base, uuid) = db_with_entry("doomed", t(0));
    let mut local = base.clone();
    let mut remote = base.clone();

    // Delete the entry in remote with tracking, so a deletion tombstone (with
    // a now() timestamp, later than t(0)) is recorded.
    {
        let id = remote
            .root()
            .entries()
            .find(|e| e.id().uuid() == uuid)
            .map(|e| e.id())
            .expect("entry exists");
        let mut entry = remote.entry_mut(id).expect("entry exists");
        entry.track_changes().remove();
    }

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert_eq!(
        current_title(&local, uuid),
        None,
        "tombstoned entry must be removed"
    );
    assert!(summary.delta.removed.contains(&uuid));
}

#[test]
fn merging_identical_databases_is_idempotent() {
    // FR-043 idempotence: merging a database with an identical copy of itself
    // changes nothing.
    let (base, _) = db_with_entry("v0", t(0));
    let mut local = base.clone();
    let remote = base.clone();
    let expected = local.clone();

    let summary = reconcile(&mut local, &remote).expect("merge succeeds");

    assert!(
        summary.delta.is_empty(),
        "idempotent merge reports no changes"
    );
    assert_eq!(
        local, expected,
        "idempotent merge leaves the database unchanged"
    );
}
