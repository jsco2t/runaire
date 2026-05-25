//! Per-vault sync configuration ([`SyncConfig`]) and its `vaults.toml`
//! integration (design §2.2.1).
//!
//! `SyncConfig` is stored under `[vault.sync]` inside each `[[vault]]` entry
//! of `vaults.toml`. `runaire-core`'s registry preserves unknown keys via
//! `#[serde(flatten)]`, so this module reads and writes the `sync` sub-table
//! out of [`RegisteredVault::extra`] without `runaire-core` knowing the sync
//! schema (ADR-004 keeps the device's sync bookmark in `vaults.toml`).
//!
//! No `gix` types appear here: that would force this crate to depend on `gix`
//! from Phase 1 and defeat the layering. `last_synced_commit` is an opaque
//! hex string at this layer; conversion to a git object id (and hex
//! validation) happens in the transport.

use runaire_core::RegisteredVault;
use serde::{Deserialize, Serialize};

/// The `vaults.toml` key under which a vault's sync config is stored.
const SYNC_KEY: &str = "sync";

/// Per-vault git-sync configuration, serialized under `[vault.sync]`.
///
/// Derives `PartialEq` but not `Eq`: [`SyncConfig::extra`] is a `toml::Table`,
/// whose values may contain `f64`, which is not `Eq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Git remote URL (SSH or HTTPS).
    pub remote_url: String,

    /// Branch to sync against. Defaults to `"main"` when omitted.
    #[serde(default = "default_branch")]
    pub branch: String,

    /// The most recent commit this device successfully synced, as a hex
    /// object-id string. Omitted from the serialized form when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_synced_commit: Option<String>,

    /// Authentication strategy.
    pub auth: AuthKind,

    /// Forward-compat: unknown top-level keys under `[sync]` preserved
    /// verbatim across save+load. Matches the `toml::Table` convention used by
    /// [`RegisteredVault::extra`].
    #[serde(flatten)]
    pub extra: toml::Table,
}

/// Authentication strategy for the git remote.
///
/// Internally tagged by a `kind` field (`"ssh"` / `"https_stored"`). Unknown
/// tags fail deserialization, defending against typos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum AuthKind {
    /// Use the user's existing ssh-agent and `~/.ssh/config`.
    Ssh,

    /// HTTPS with a username and a ChaCha20-Poly1305-encrypted password.
    HttpsStored {
        /// HTTPS basic-auth username.
        username: String,
        /// Base64-encoded "RST-CRED-1" credential container (design §2.2.3).
        /// Opaque at this layer; decrypted just-in-time by `auth`.
        password_encrypted: String,
    },
}

/// The default sync branch.
fn default_branch() -> String {
    "main".to_string()
}

impl SyncConfig {
    /// The default sync branch, `"main"`.
    #[must_use]
    pub fn default_branch() -> String {
        default_branch()
    }

    /// Read the `[sync]` sub-table out of a registered vault entry.
    ///
    /// Returns `None` when the entry has no `sync` sub-table **or** when the
    /// sub-table fails to deserialize into a `SyncConfig` (e.g. a hand-edited,
    /// malformed block). Surfacing parse errors distinctly is a planned
    /// post-Phase-1 refinement, gated on adding a `ConfigMalformed` variant to
    /// [`crate::SyncError`] (whose surface is fixed in this phase, design §2.8).
    /// In normal operation the block is only ever written by
    /// [`SyncConfig::to_vault_entry`], so a round-trip always deserializes.
    #[must_use]
    pub fn from_vault_entry(entry: &RegisteredVault) -> Option<SyncConfig> {
        entry
            .extra
            .get(SYNC_KEY)
            .cloned()
            .and_then(|value| value.try_into().ok())
    }

    /// Write this config into a registered vault entry's `[sync]` sub-table,
    /// overwriting any existing one. Unknown keys captured in
    /// [`SyncConfig::extra`] are written back verbatim.
    ///
    /// Serialization is infallible: `SyncConfig` contains only strings, an
    /// optional string, a tagged enum, and a TOML table — all unconditionally
    /// representable as a TOML value.
    ///
    /// # Panics
    ///
    /// Never in practice. The `expect` guards a `toml::Value` serialization
    /// that cannot fail for `SyncConfig`'s field types; a panic here would
    /// signal a bug in `toml`, not bad input.
    pub fn to_vault_entry(&self, entry: &mut RegisteredVault) {
        let value = toml::Value::try_from(self)
            .expect("SyncConfig is always representable as a TOML value");
        entry.extra.insert(SYNC_KEY.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runaire_core::{RegisteredVault, RunairePaths, VaultRegistry};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn ssh_config() -> SyncConfig {
        SyncConfig {
            remote_url: "git@github.com:me/vault.git".to_string(),
            branch: "main".to_string(),
            last_synced_commit: Some("a".repeat(40)),
            auth: AuthKind::Ssh,
            extra: toml::Table::new(),
        }
    }

    fn https_config() -> SyncConfig {
        SyncConfig {
            remote_url: "https://example.com/me/vault.git".to_string(),
            branch: "main".to_string(),
            last_synced_commit: None,
            auth: AuthKind::HttpsStored {
                username: "alice".to_string(),
                password_encrypted: "UkMwMQ-base64-blob".to_string(),
            },
            extra: toml::Table::new(),
        }
    }

    // -- TC-CONFIG-001 ------------------------------------------------------
    #[test]
    fn sync_config_round_trips_ssh_auth() {
        let cfg = ssh_config();
        let serialized = toml::to_string(&cfg).expect("serialize");
        let back: SyncConfig = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(cfg, back);
    }

    // -- TC-CONFIG-002 ------------------------------------------------------
    #[test]
    fn sync_config_round_trips_https_stored() {
        let cfg = https_config();
        let serialized = toml::to_string(&cfg).expect("serialize");
        let back: SyncConfig = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(cfg, back);
    }

    // -- TC-CONFIG-003 ------------------------------------------------------
    #[test]
    fn sync_config_default_branch_is_main() {
        let toml_without_branch = r#"
remote_url = "git@github.com:me/vault.git"

[auth]
kind = "ssh"
"#;
        let cfg: SyncConfig = toml::from_str(toml_without_branch).expect("deserialize");
        assert_eq!(cfg.branch, "main");
        assert_eq!(SyncConfig::default_branch(), "main");
    }

    // -- TC-CONFIG-004 ------------------------------------------------------
    #[test]
    fn sync_config_extra_preserves_unknown_top_level_keys() {
        let with_unknown = r#"
remote_url = "git@github.com:me/vault.git"
branch = "main"
crazy_future_field = "x"

[auth]
kind = "ssh"
"#;
        let cfg: SyncConfig = toml::from_str(with_unknown).expect("deserialize");
        assert_eq!(
            cfg.extra
                .get("crazy_future_field")
                .and_then(toml::Value::as_str),
            Some("x"),
            "unknown top-level key captured in extra"
        );

        let serialized = toml::to_string(&cfg).expect("serialize");
        let reparsed: toml::Table = toml::from_str(&serialized).expect("reparse");
        assert_eq!(
            reparsed
                .get("crazy_future_field")
                .and_then(toml::Value::as_str),
            Some("x"),
            "unknown top-level key survives the round-trip"
        );
    }

    // -- TC-CONFIG-005 ------------------------------------------------------
    // The architecture preserves unknown *nested* keys via `SyncConfig.extra`
    // (a `toml::Table`), which can hold sub-tables. NOTE: this verifies an
    // unknown sub-table under `[sync]` (e.g. a future feature's block), not
    // keys inside `[sync.auth]`: `AuthKind` is a closed tagged enum by design
    // (§2.2.1) with no per-variant catch-all, so keys inside `[auth]` are not
    // a forward-compat surface. (Deviation from impl-plan §8.2.1's `[sync.auth]`
    // example — recorded in the task report.)
    #[test]
    fn sync_config_extra_preserves_unknown_nested_keys() {
        let with_unknown_subtable = r#"
remote_url = "git@github.com:me/vault.git"
branch = "main"

[auth]
kind = "ssh"

[future_feature]
enabled = true
threshold = 42
"#;
        let cfg: SyncConfig = toml::from_str(with_unknown_subtable).expect("deserialize");
        let future = cfg
            .extra
            .get("future_feature")
            .and_then(toml::Value::as_table)
            .expect("unknown nested sub-table captured in extra");
        assert_eq!(
            future.get("enabled").and_then(toml::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            future.get("threshold").and_then(toml::Value::as_integer),
            Some(42)
        );

        let serialized = toml::to_string(&cfg).expect("serialize");
        let reparsed: SyncConfig = toml::from_str(&serialized).expect("reparse");
        assert_eq!(reparsed, cfg, "nested sub-table survives the round-trip");
    }

    // -- TC-CONFIG-006 ------------------------------------------------------
    #[test]
    fn sync_config_last_synced_commit_omitted_when_none() {
        let cfg = https_config(); // last_synced_commit = None
        assert!(cfg.last_synced_commit.is_none());
        let serialized = toml::to_string(&cfg).expect("serialize");
        assert!(
            !serialized.contains("last_synced_commit"),
            "None must be omitted from the serialized form, got:\n{serialized}"
        );
    }

    // -- TC-CONFIG-007 ------------------------------------------------------
    #[test]
    fn sync_config_invalid_kind_tag_errors() {
        let bad_kind = r#"
remote_url = "git@github.com:me/vault.git"
branch = "main"

[auth]
kind = "telepathy"
"#;
        let result: Result<SyncConfig, _> = toml::from_str(bad_kind);
        assert!(
            result.is_err(),
            "unknown auth kind must fail to deserialize"
        );
    }

    // -- vaults.toml integration helpers ------------------------------------

    fn paths_in(dir: &TempDir) -> RunairePaths {
        RunairePaths::with_state_dir(dir.path().join("state"))
    }

    fn seed_vaults_toml(paths: &RunairePaths, body: &str) {
        paths.ensure_exists().expect("ensure state dir");
        std::fs::write(paths.vaults_toml(), body).expect("seed vaults.toml");
    }

    // -- TC-CONFIG-008 ------------------------------------------------------
    #[test]
    fn vaults_toml_load_with_sync_block_succeeds() {
        let tmp = TempDir::new().expect("tempdir");
        let paths = paths_in(&tmp);
        seed_vaults_toml(
            &paths,
            r#"
version = 1

[[vault]]
name = "work"
path = "/tmp/work.kdbx"
created_at = "2026-05-20T10:00:00-06:00"

[vault.sync]
remote_url = "git@github.com:me/work.git"
branch = "main"

[vault.sync.auth]
kind = "ssh"
"#,
        );

        let registry = VaultRegistry::load(paths).expect("load registry");
        let entry = registry.get("work").expect("work vault present");
        let cfg = SyncConfig::from_vault_entry(entry).expect("sync config present");
        assert_eq!(cfg.remote_url, "git@github.com:me/work.git");
        assert_eq!(cfg.branch, "main");
        assert_eq!(cfg.auth, AuthKind::Ssh);
    }

    // -- TC-CONFIG-009 ------------------------------------------------------
    #[test]
    fn vaults_toml_save_then_load_preserves_sync_block() {
        let tmp = TempDir::new().expect("tempdir");
        let paths = paths_in(&tmp);

        // Carry a non-empty `extra` so this also proves the forward-compat
        // guarantee through the *write* path (`to_vault_entry`'s
        // `Value::try_from` serializer + registry save), not just the
        // `from_str`/`to_string` path that TC-CONFIG-004/005 cover.
        let mut cfg = ssh_config();
        cfg.extra.insert(
            "future_feature".to_string(),
            toml::Value::Table({
                let mut t = toml::Table::new();
                t.insert("enabled".to_string(), toml::Value::Boolean(true));
                t
            }),
        );

        // Write: register a vault carrying the sync block, then save.
        {
            let mut registry = VaultRegistry::with_paths(paths.clone());
            let mut entry = RegisteredVault {
                name: "personal".to_string(),
                path: PathBuf::from("/tmp/personal.kdbx"),
                created_at: "2026-05-20T10:00:00-06:00".to_string(),
                keyfile_path: None,
                extra: toml::Table::new(),
            };
            cfg.to_vault_entry(&mut entry);
            registry.register(entry).expect("register");
            registry.save().expect("save");
        }

        // Read back in a fresh registry instance.
        let registry = VaultRegistry::load(paths).expect("reload");
        let entry = registry.get("personal").expect("personal vault present");
        let loaded = SyncConfig::from_vault_entry(entry).expect("sync config present");
        assert_eq!(loaded, cfg, "sync block survives save + reload");
    }

    // -- TC-CONFIG-010 ------------------------------------------------------
    #[test]
    fn vaults_toml_load_without_sync_block_returns_none() {
        let tmp = TempDir::new().expect("tempdir");
        let paths = paths_in(&tmp);
        seed_vaults_toml(
            &paths,
            r#"
version = 1

[[vault]]
name = "plain"
path = "/tmp/plain.kdbx"
created_at = "2026-05-20T10:00:00-06:00"
"#,
        );

        let registry = VaultRegistry::load(paths).expect("load registry");
        let entry = registry.get("plain").expect("plain vault present");
        assert!(
            SyncConfig::from_vault_entry(entry).is_none(),
            "a vault without a [sync] block has no SyncConfig"
        );
    }
}
