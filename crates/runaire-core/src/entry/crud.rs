//! Entry CRUD operations on unlocked vaults.

use keepass::db::{fields, Entry, EntryId, GroupId, History, Times};
use keepass::Database;

use crate::{EntryDraft, EntryView, EntryViewMut, Tag, Vault, VaultError};

const DEFAULT_MAX_HISTORY_PER_ENTRY: usize = 10;
const RECYCLE_BIN_NAME: &str = "Recycle Bin";

impl Vault {
    /// Add `draft` to `group` and return the freshly allocated entry UUID.
    ///
    /// The entry exists only in the in-memory vault until [`Self::save`] is
    /// called.
    pub fn add_entry(
        &mut self,
        group: uuid::Uuid,
        draft: EntryDraft,
    ) -> Result<uuid::Uuid, VaultError> {
        let group_id = find_group_id(self.database(), group)?;
        let mut group = self
            .database_mut()
            .group_mut(group_id)
            .ok_or(VaultError::GroupNotFound { uuid: group })?;
        let mut entry = group.add_entry();
        populate_entry_from_draft(&mut entry, draft);
        Ok(entry.id().uuid())
    }

    /// Return a read-only view of an entry.
    pub fn get_entry(&self, uuid: uuid::Uuid) -> Result<EntryView<'_>, VaultError> {
        let entry_id = find_entry_id(self.database(), uuid)?;
        self.database()
            .entry(entry_id)
            .map(EntryView::new)
            .ok_or(VaultError::EntryNotFound { uuid })
    }

    /// Return a mutable view of an entry.
    ///
    /// Prefer [`Self::update_entry`] for normal edits so KDBX history is
    /// appended exactly once for the logical update.
    pub fn get_entry_mut(&mut self, uuid: uuid::Uuid) -> Result<EntryViewMut<'_>, VaultError> {
        let entry_id = find_entry_id(self.database(), uuid)?;
        self.database_mut()
            .entry_mut(entry_id)
            .map(EntryViewMut::new)
            .ok_or(VaultError::EntryNotFound { uuid })
    }

    /// Update an entry, appending one history snapshot for the committed edit.
    ///
    /// If `f` returns an error, the entry is restored to its pre-call state and
    /// no history snapshot remains.
    pub fn update_entry<F>(&mut self, uuid: uuid::Uuid, f: F) -> Result<(), VaultError>
    where
        F: FnOnce(&mut EntryViewMut<'_>) -> Result<(), VaultError>,
    {
        let entry_id = find_entry_id(self.database(), uuid)?;
        let max_history = self.max_history_per_entry();
        let snapshot = {
            let mut entry = self
                .database_mut()
                .entry_mut(entry_id)
                .ok_or(VaultError::EntryNotFound { uuid })?;
            let snapshot = entry.clone();
            let mut historical = snapshot.clone();
            historical.history = None;
            entry.history.get_or_insert_default().add_entry(historical);
            prune_history(&mut entry, max_history);
            snapshot
        };

        let result = {
            let entry = self
                .database_mut()
                .entry_mut(entry_id)
                .ok_or(VaultError::EntryNotFound { uuid })?;
            let mut view = EntryViewMut::new(entry);
            f(&mut view)
        };

        match result {
            Ok(()) => {
                let mut entry = self
                    .database_mut()
                    .entry_mut(entry_id)
                    .ok_or(VaultError::EntryNotFound { uuid })?;
                entry.times.last_modification = Some(Times::now());
                Ok(())
            }
            Err(err) => {
                let mut entry = self
                    .database_mut()
                    .entry_mut(entry_id)
                    .ok_or(VaultError::EntryNotFound { uuid })?;
                *entry = snapshot;
                Err(err)
            }
        }
    }

    /// Delete an entry using the vault's recycle-bin setting.
    ///
    /// When the recycle bin is enabled, the entry is moved there with its UUID
    /// preserved. When disabled, this permanently removes the entry.
    pub fn delete_entry(&mut self, uuid: uuid::Uuid) -> Result<(), VaultError> {
        if !self.database().meta.recyclebin_enabled.unwrap_or(true) {
            return self.purge_entry(uuid);
        }

        let entry_id = find_entry_id(self.database(), uuid)?;
        let bin_id = recycle_bin_group_id_or_create(self.database_mut());
        let parent_id = self
            .database()
            .entry(entry_id)
            .ok_or(VaultError::EntryNotFound { uuid })?
            .parent()
            .id();

        if parent_id == bin_id {
            return Ok(());
        }

        self.database_mut()
            .entry_mut(entry_id)
            .ok_or(VaultError::EntryNotFound { uuid })?
            .move_to(bin_id)
            .map_err(|_| VaultError::GroupNotFound {
                uuid: bin_id.uuid(),
            })?;
        Ok(())
    }

    /// Permanently remove an entry regardless of recycle-bin settings.
    pub fn purge_entry(&mut self, uuid: uuid::Uuid) -> Result<(), VaultError> {
        let entry_id = find_entry_id(self.database(), uuid)?;
        self.database_mut()
            .entry_mut(entry_id)
            .ok_or(VaultError::EntryNotFound { uuid })?
            .remove();
        Ok(())
    }

    /// Move an entry to another group while preserving the entry UUID.
    pub fn move_entry(
        &mut self,
        uuid: uuid::Uuid,
        new_group: uuid::Uuid,
    ) -> Result<(), VaultError> {
        let entry_id = find_entry_id(self.database(), uuid)?;
        let group_id = find_group_id(self.database(), new_group)?;
        self.database_mut()
            .entry_mut(entry_id)
            .ok_or(VaultError::EntryNotFound { uuid })?
            .move_to(group_id)
            .map_err(|_| VaultError::GroupNotFound { uuid: new_group })?;
        Ok(())
    }

    /// Return the max history snapshots kept per entry.
    pub fn max_history_per_entry(&self) -> usize {
        self.database()
            .meta
            .history_max_items
            .and_then(|value| usize::try_from(value).ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_HISTORY_PER_ENTRY)
    }

    /// Set the max history snapshots kept per entry.
    pub fn set_max_history_per_entry(&mut self, n: usize) -> Result<(), VaultError> {
        self.database_mut().meta.history_max_items =
            Some(
                isize::try_from(n).map_err(|_| VaultError::InvalidGroupTarget {
                    reason: "history cap exceeds supported range",
                })?,
            );
        Ok(())
    }
}

pub(crate) fn find_entry_id(db: &Database, uuid: uuid::Uuid) -> Result<EntryId, VaultError> {
    db.iter_all_entries()
        .find(|entry| entry.id().uuid() == uuid)
        .map(|entry| entry.id())
        .ok_or(VaultError::EntryNotFound { uuid })
}

pub(crate) fn find_group_id(db: &Database, uuid: uuid::Uuid) -> Result<GroupId, VaultError> {
    db.iter_all_groups()
        .find(|group| group.id().uuid() == uuid)
        .map(|group| group.id())
        .ok_or(VaultError::GroupNotFound { uuid })
}

pub(crate) fn recycle_bin_group_id_or_create(db: &mut Database) -> GroupId {
    if let Some(bin) = db.recycle_bin() {
        return bin.id();
    }

    let bin_id = db
        .root_mut()
        .add_group()
        .edit(|group| {
            group.name = RECYCLE_BIN_NAME.to_string();
            group.times.creation = Some(Times::now());
            group.times.last_modification = Some(Times::now());
        })
        .id();
    db.meta.recyclebin_uuid = Some(bin_id.uuid());
    db.meta.recyclebin_enabled = Some(true);
    db.meta.recyclebin_changed = Some(Times::now());
    bin_id
}

pub(crate) fn is_entry_in_recycle_bin(db: &Database, entry_id: EntryId) -> bool {
    db.recycle_bin()
        .is_some_and(|bin| group_contains_entry(&bin, entry_id))
}

fn group_contains_entry(group: &keepass::db::GroupRef<'_>, entry_id: EntryId) -> bool {
    group.entry(entry_id).is_some()
        || group
            .groups()
            .any(|child| group_contains_entry(&child, entry_id))
}

fn populate_entry_from_draft(entry: &mut keepass::db::EntryMut<'_>, draft: EntryDraft) {
    entry.set_unprotected(fields::TITLE, draft.title);
    if let Some(username) = draft.username {
        entry.set_unprotected(fields::USERNAME, username);
    }
    if let Some(password) = draft.password {
        entry.set_protected(fields::PASSWORD, password.to_string());
    }
    if let Some(url) = draft.url {
        entry.set_unprotected(fields::URL, url);
    }
    if let Some(notes) = draft.notes {
        entry.set_unprotected(fields::NOTES, notes);
    }
    if let Some(expires_at) = draft.expires_at {
        entry.times.expires = Some(true);
        entry.times.expiry = Some(expires_at.naive_utc());
    }
    entry.tags = draft.tags.into_iter().map(Tag::into_inner).collect();
    for field in draft.custom_fields {
        if field.protected {
            entry.set_protected(field.name, field.value);
        } else {
            entry.set_unprotected(field.name, field.value);
        }
    }
    entry.times.creation = Some(Times::now());
    entry.times.last_modification = Some(Times::now());
}

fn prune_history(entry: &mut Entry, max: usize) {
    if let Some(history) = &mut entry.history {
        if history.get_entries().len() > max {
            let retained: Vec<_> = history.get_entries().iter().take(max).cloned().collect();
            let mut pruned = History::default();
            for historical in retained.into_iter().rev() {
                pruned.add_entry(historical);
            }
            entry.history = Some(pruned);
        }
    }
}
