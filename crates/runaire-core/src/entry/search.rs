//! Entry search over an unlocked KDBX database.
//!
//! Two modes per FR-014:
//!
//! - [`SearchMode::Substring`] (default): case-insensitive substring match.
//! - [`SearchMode::Wildcard`] (opt-in): case-insensitive whole-field match
//!   with `*` (any sequence, including empty) and `?` (exactly one
//!   character). To express "field contains FOO" use `*FOO*`. Literal
//!   `*` and `?` are not supported as search terms — see PRD §6.2 FR-014.
//!
//! The wildcard matcher is a small in-tree iterative two-pointer
//! implementation (`wildcard_match` below) — deliberately a few dozen
//! lines so we don't pull a regex/PCRE crate for an opt-in feature.

use keepass::db::{fields, EntryRef};
use keepass::Database;

use crate::entry::crud::is_entry_in_recycle_bin;
use crate::{Vault, VaultError, VaultReadOnly};

/// Ranking weight for title matches.
pub const RANK_TITLE: u32 = 8;
/// Ranking weight for username matches.
pub const RANK_USERNAME: u32 = 4;
/// Ranking weight for URL matches.
pub const RANK_URL: u32 = 2;
/// Ranking weight for notes matches.
pub const RANK_NOTES: u32 = 1;
/// Ranking weight for tag matches.
pub const RANK_TAGS: u32 = 4;

/// Search configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchOptions {
    query: String,
    mode: SearchMode,
    include_recycled: bool,
}

/// Search matching mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchMode {
    /// Case-insensitive substring match.
    Substring,
    /// Case-insensitive wildcard match anchored to the whole field. `*`
    /// matches any sequence of characters (including empty); `?` matches
    /// exactly one character. Every other character is matched literally.
    Wildcard,
}

/// One entry search hit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    /// Entry UUID.
    pub uuid: uuid::Uuid,
    /// Field-weighted score. Higher scores sort first.
    pub score: u32,
    /// Fields that matched the query.
    pub matched_fields: Vec<MatchedField>,
}

/// Entry field that matched a search query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchedField {
    /// KDBX `Title` field.
    Title,
    /// KDBX `UserName` field.
    Username,
    /// KDBX `URL` field.
    Url,
    /// KDBX `Notes` field.
    Notes,
    /// Entry tags.
    Tags,
}

impl SearchOptions {
    /// Construct default substring search options for `query`.
    ///
    /// Defaults to case-insensitive substring matching and excludes entries in
    /// the Recycle Bin.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            mode: SearchMode::Substring,
            include_recycled: false,
        }
    }

    /// Enable or disable wildcard mode.
    ///
    /// Wildcards (`*`, `?`) match the whole value — pad with `*` to express
    /// "contains."
    #[must_use]
    pub fn wildcard(mut self, on: bool) -> Self {
        self.mode = if on {
            SearchMode::Wildcard
        } else {
            SearchMode::Substring
        };
        self
    }

    /// Include or exclude Recycle Bin entries.
    #[must_use]
    pub fn include_recycled(mut self, on: bool) -> Self {
        self.include_recycled = on;
        self
    }

    /// Return the configured query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Return the configured search mode.
    pub fn mode(&self) -> SearchMode {
        self.mode
    }

    /// Return whether recycled entries are included.
    pub fn includes_recycled(&self) -> bool {
        self.include_recycled
    }
}

impl Vault {
    /// Search entries in this writable vault.
    pub fn search(&self, opts: SearchOptions) -> Result<Vec<SearchResult>, VaultError> {
        Ok(search_database(self.database(), opts))
    }
}

impl VaultReadOnly {
    /// Search entries in this read-only vault.
    pub fn search(&self, opts: SearchOptions) -> Result<Vec<SearchResult>, VaultError> {
        Ok(search_database(self.database(), opts))
    }
}

fn search_database(db: &Database, opts: SearchOptions) -> Vec<SearchResult> {
    if opts.query.is_empty() {
        return Vec::new();
    }

    let include_recycled = opts.include_recycled;
    let matcher = Matcher::new(opts);
    let mut results = Vec::new();

    for entry in db.iter_all_entries() {
        if !include_recycled && is_entry_in_recycle_bin(db, entry.id()) {
            continue;
        }

        if let Some(result) = score_entry(&entry, &matcher) {
            results.push(result);
        }
    }

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.uuid.as_bytes().cmp(right.uuid.as_bytes()))
    });

    results
}

enum Matcher {
    Substring { needle: String },
    Wildcard { pattern: Vec<char> },
}

impl Matcher {
    fn new(opts: SearchOptions) -> Self {
        let SearchOptions { query, mode, .. } = opts;
        match mode {
            SearchMode::Substring => Self::Substring {
                needle: query.to_lowercase(),
            },
            SearchMode::Wildcard => Self::Wildcard {
                pattern: query.to_lowercase().chars().collect(),
            },
        }
    }

    fn is_match(&self, value: &str) -> bool {
        match self {
            Self::Substring { needle } => value.to_lowercase().contains(needle.as_str()),
            Self::Wildcard { pattern } => {
                let value: Vec<char> = value.to_lowercase().chars().collect();
                wildcard_match(pattern, &value)
            }
        }
    }
}

/// Whole-string wildcard match: `*` matches any sequence (including empty),
/// `?` matches exactly one character, all other characters match literally.
///
/// Iterative two-pointer algorithm with backtrack on `*` — O(n+m) for
/// typical inputs and O(n*m) worst case, well within budget for the
/// 5,000-entry / ≤200-char field workload search is benchmarked against.
fn wildcard_match(pattern: &[char], value: &[char]) -> bool {
    let mut p = 0;
    let mut v = 0;
    let mut star_p: Option<usize> = None;
    let mut star_v: usize = 0;

    while v < value.len() {
        if p < pattern.len() && pattern[p] == '*' {
            star_p = Some(p);
            star_v = v;
            p += 1;
        } else if p < pattern.len() && (pattern[p] == '?' || pattern[p] == value[v]) {
            p += 1;
            v += 1;
        } else if let Some(saved) = star_p {
            p = saved + 1;
            star_v += 1;
            v = star_v;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == '*' {
        p += 1;
    }
    p == pattern.len()
}

fn score_entry(entry: &EntryRef<'_>, matcher: &Matcher) -> Option<SearchResult> {
    let mut score = 0;
    let mut matched_fields = Vec::new();

    score_field(
        entry.get(fields::TITLE).unwrap_or(""),
        matcher,
        RANK_TITLE,
        MatchedField::Title,
        &mut score,
        &mut matched_fields,
    );
    score_field(
        entry.get(fields::USERNAME).unwrap_or(""),
        matcher,
        RANK_USERNAME,
        MatchedField::Username,
        &mut score,
        &mut matched_fields,
    );
    score_field(
        entry.get(fields::URL).unwrap_or(""),
        matcher,
        RANK_URL,
        MatchedField::Url,
        &mut score,
        &mut matched_fields,
    );
    score_field(
        entry.get(fields::NOTES).unwrap_or(""),
        matcher,
        RANK_NOTES,
        MatchedField::Notes,
        &mut score,
        &mut matched_fields,
    );

    let tags = entry.tags.join(";");
    score_field(
        &tags,
        matcher,
        RANK_TAGS,
        MatchedField::Tags,
        &mut score,
        &mut matched_fields,
    );

    (score > 0).then(|| SearchResult {
        uuid: entry.id().uuid(),
        score,
        matched_fields,
    })
}

fn score_field(
    value: &str,
    matcher: &Matcher,
    weight: u32,
    field: MatchedField,
    score: &mut u32,
    matched_fields: &mut Vec<MatchedField>,
) {
    if matcher.is_match(value) {
        *score += weight;
        matched_fields.push(field);
    }
}

#[cfg(test)]
mod tests {
    use keepass::db::Value;

    use super::*;

    fn search_db() -> (Database, uuid::Uuid, uuid::Uuid) {
        let mut db = Database::new();
        let title = db
            .root_mut()
            .add_entry()
            .edit(|entry| {
                entry.set_unprotected(fields::TITLE, "Example");
                entry.set_unprotected(fields::USERNAME, "alice");
                entry.set_unprotected(fields::URL, "https://title.test");
                entry.set_unprotected(fields::NOTES, "no keyword here");
                entry.tags.push("primary".to_string());
            })
            .id()
            .uuid();
        let notes = db
            .root_mut()
            .add_entry()
            .edit(|entry| {
                entry.set_unprotected(fields::TITLE, "Secondary");
                entry.set_unprotected(fields::USERNAME, "bob");
                entry.set_unprotected(fields::URL, "https://notes.test");
                entry.set_unprotected(fields::NOTES, "example appears only here");
                entry.tags.push("archive".to_string());
                entry.add_attachment("ignored.txt", Value::unprotected(b"example".to_vec()));
            })
            .id()
            .uuid();
        (db, title, notes)
    }

    #[test]
    fn substring_search_matches_standard_fields_and_tags() {
        let (mut db, _, _) = search_db();
        db.root_mut().add_entry().edit(|entry| {
            entry.set_unprotected(fields::TITLE, "Tagged");
            entry.tags.push("ExampleTag".to_string());
        });

        let results = search_database(&db, SearchOptions::new("example"));

        assert_eq!(results.len(), 3);
        assert!(results
            .iter()
            .any(|result| result.matched_fields.contains(&MatchedField::Tags)));
    }

    #[test]
    fn substring_search_is_case_insensitive() {
        let (db, _, _) = search_db();

        let lower = search_database(&db, SearchOptions::new("example"));
        let upper = search_database(&db, SearchOptions::new("EXAMPLE"));

        assert_eq!(lower, upper);
    }

    #[test]
    fn title_match_ranks_above_notes_match() {
        let (db, title_uuid, notes_uuid) = search_db();

        let results = search_database(&db, SearchOptions::new("example"));

        assert_eq!(results[0].uuid, title_uuid);
        assert_eq!(results[1].uuid, notes_uuid);
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn wildcard_mode_anchors_to_whole_field() {
        let (db, title_uuid, _) = search_db();

        // Anchored: "Example" matches only the entry whose title is exactly
        // "Example" (and no other field on any other entry contains it
        // standalone).
        let results = search_database(&db, SearchOptions::new("example").wildcard(true));

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uuid, title_uuid);
    }

    #[test]
    fn wildcard_star_matches_any_substring() {
        let (db, title_uuid, notes_uuid) = search_db();

        let results = search_database(&db, SearchOptions::new("*example*").wildcard(true));

        // Both the title and the notes entry should match, since *example*
        // matches anywhere in a field.
        let uuids: Vec<_> = results.iter().map(|result| result.uuid).collect();
        assert!(uuids.contains(&title_uuid));
        assert!(uuids.contains(&notes_uuid));
    }

    #[test]
    fn wildcard_question_mark_matches_single_character() {
        let mut db = Database::new();
        let one = db
            .root_mut()
            .add_entry()
            .edit(|entry| entry.set_unprotected(fields::TITLE, "cat"))
            .id()
            .uuid();
        db.root_mut().add_entry().edit(|entry| {
            entry.set_unprotected(fields::TITLE, "cart");
        });

        let results = search_database(&db, SearchOptions::new("c?t").wildcard(true));

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uuid, one);
    }

    #[test]
    fn empty_query_returns_no_results() {
        let (db, _, _) = search_db();

        let results = search_database(&db, SearchOptions::new(""));

        assert!(results.is_empty());
    }

    #[test]
    fn wildcard_match_basics() {
        // Direct unit tests on the matcher to lock in the contract.
        let cases: &[(&str, &str, bool)] = &[
            ("", "", true),
            ("*", "", true),
            ("*", "anything", true),
            ("?", "", false),
            ("?", "a", true),
            ("?", "ab", false),
            ("a*b", "ab", true),
            ("a*b", "axxb", true),
            ("a*b", "axxc", false),
            ("foo*", "foobar", true),
            ("*bar", "foobar", true),
            ("*foo*", "xfoox", true),
            ("a?c", "abc", true),
            ("a?c", "ac", false),
            ("**", "abc", true),
        ];
        for (pattern, value, want) in cases {
            let pattern: Vec<char> = pattern.chars().collect();
            let value: Vec<char> = value.chars().collect();
            let got = wildcard_match(&pattern, &value);
            assert_eq!(
                got, *want,
                "pattern={pattern:?} value={value:?} want={want}"
            );
        }
    }
}
