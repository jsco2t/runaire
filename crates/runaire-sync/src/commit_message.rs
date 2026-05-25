//! Structured commit-message generator (design §2.2.9; grammar §2.6).
//!
//! Renders an [`EntryDelta`] plus a conflict count into a one-line,
//! machine-parseable message — `runaire: <events>[, <C> conflicts]` with a
//! fixed `added, updated, deleted` event order. **Phase 1 scaffold:**
//! signature only; the grammar implementation lands in Phase 5 (T5.2).

use crate::merge::EntryDelta;

/// Render an [`EntryDelta`] (and conflict count) into a one-line commit
/// message per the §2.6 grammar.
///
/// **Phase 1 stub** — implemented in Phase 5 (T5.2).
#[must_use]
pub fn structured(_delta: &EntryDelta, _conflicts: usize) -> String {
    unimplemented!("Phase 5 — task T5.2 (commit_message::structured)")
}
