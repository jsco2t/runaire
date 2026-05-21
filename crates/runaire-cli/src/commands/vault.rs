//! `runaire vault {create, open, list, set-lock}` dispatcher.
//!
//! `set-sync` is a Phase-4 slot (lives in `commands/sync.rs`'s style;
//! the verb here returns `NotImplemented` until `features/sync-git/`
//! ships its real body).
//!
//! ## State directory resolution
//!
//! The global `--registry <path>` flag is treated as the absolute path
//! to a `vaults.toml` file. We derive a [`RunairePaths`] rooted at the
//! file's parent directory. When the flag is omitted we fall back to
//! [`RunairePaths::from_env`] (the `$HOME/.local/state/runaire/` path).
//!
//! ## `[vault.lock]` schema
//!
//! `runaire-security` (where the typed `VaultLockConfig` API will
//! live) has not yet shipped. The `[vault.lock] idle_timeout_seconds`
//! TOML field is forward-compatible: this file writes it through the
//! `RegisteredVault::extra: toml::Table` field, and the security
//! crate will later read the same on-disk shape via its own typed
//! wrapper. When security-behaviors lands, this module switches to its
//! API; the registry data does not change.

use std::path::PathBuf;

use runaire_core::{
    KdfParams, Keyfile, NoRecoveryConfirmed, RegisteredVault, RunairePaths, Vault, VaultError,
    VaultRegistry,
};

use crate::agent::NoAgentClient;
use crate::cli::{
    Cli, VaultArgs, VaultCreateArgs, VaultOpenArgs, VaultSetLockArgs, VaultSetSyncArgs, VaultVerb,
};
use crate::exit::CliExit;
use crate::format::OutputFormatter;
use crate::prompt::{master_password, new_master_password_confirmed, PromptOpts};
use crate::views::vault::{
    VaultCreateKdfView, VaultCreateView, VaultListEntry, VaultListView, VaultOpenView,
    VaultSetLockView,
};

/// Sub-table name used inside `RegisteredVault::extra` for per-vault
/// lock config. The `[vault.lock] idle_timeout_seconds` shape is the
/// on-disk schema security-behaviors will later own.
const LOCK_TABLE_KEY: &str = "lock";
/// Field name inside the lock sub-table.
const IDLE_TIMEOUT_KEY: &str = "idle_timeout_seconds";

/// Phase 2 entry point — dispatches to the verb handler.
///
/// # Errors
///
/// Any [`CliExit`] returned by the per-verb handlers.
pub fn run(cli: &Cli, args: &VaultArgs) -> Result<(), CliExit> {
    match &args.verb {
        Some(VaultVerb::Create(create)) => run_create(cli, create),
        Some(VaultVerb::Open(open)) => run_open(cli, open),
        Some(VaultVerb::List(_)) => run_list(cli),
        Some(VaultVerb::SetSync(set_sync)) => run_set_sync(set_sync),
        Some(VaultVerb::SetLock(setlock)) => run_set_lock(cli, setlock),
        None => Err(CliExit::UserError(
            "missing subcommand verb (try `runaire vault --help`)".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// vault create
// ---------------------------------------------------------------------------

fn run_create(cli: &Cli, args: &VaultCreateArgs) -> Result<(), CliExit> {
    if !args.no_recovery_warning {
        return Err(CliExit::UserError(
            "vault create requires --no-recovery-warning: there is no master-password recovery \
             in Rùnaire (data lost on forgotten password)"
                .to_string(),
        ));
    }

    // Load the registry FIRST so a duplicate-id error surfaces before
    // we run the expensive Argon2id KDF + write a .kdbx file that the
    // user can't easily clean up. Belt-and-suspenders: the
    // `VaultRegistry::register` call below still re-checks for
    // duplicates, so a race between two concurrent `vault create`
    // invocations can't silently succeed.
    let paths = resolve_paths(cli)?;
    let mut registry = VaultRegistry::load(paths).map_err(CliExit::from)?;
    if registry.get(&args.id).is_some() {
        return Err(CliExit::from(runaire_core::VaultError::AlreadyRegistered {
            name: args.id.clone(),
        }));
    }

    // Collect the password BEFORE touching disk so a user who Ctrl-Cs
    // out of the prompt doesn't leave a half-registered vault behind.
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut stderr = std::io::stderr().lock();
    let master = new_master_password_confirmed(&mut stdin, &mut stderr)?;
    let keyfile = args.keyfile.clone().map(Keyfile::Path);
    let kdf = KdfParams::default();

    let _vault = Vault::create(
        &args.path,
        &master,
        keyfile.as_ref(),
        kdf,
        NoRecoveryConfirmed::yes(),
    )
    .map_err(CliExit::from)?;
    // Drop the Vault handle here — Phase 2's `vault create` returns
    // immediately after registration; the file is on disk and the
    // registry will be updated below. The handle's exclusive lock is
    // released as it drops.

    registry
        .register(RegisteredVault {
            name: args.id.clone(),
            path: args.path.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            keyfile_path: args.keyfile.clone(),
            extra: toml::Table::new(),
        })
        .map_err(CliExit::from)?;
    registry.save().map_err(CliExit::from)?;

    let view = VaultCreateView {
        id: &args.id,
        path: &args.path,
        keyfile: args.keyfile.as_deref(),
        kdf: VaultCreateKdfView {
            algorithm: "argon2id",
            memory_kib: kdf.memory_kib,
            iterations: kdf.iterations,
            parallelism: kdf.parallelism,
        },
    };
    write_success(cli, &view)
}

// ---------------------------------------------------------------------------
// vault open (probe)
// ---------------------------------------------------------------------------

fn run_open(cli: &Cli, args: &VaultOpenArgs) -> Result<(), CliExit> {
    let paths = resolve_paths(cli)?;
    let registry = VaultRegistry::load(paths).map_err(CliExit::from)?;
    let record = registry
        .get(&args.id)
        .ok_or_else(|| {
            CliExit::from(VaultError::NotRegistered {
                name: args.id.clone(),
            })
        })?
        .clone();

    let agent = NoAgentClient;
    let opts = PromptOpts {
        vault: &args.id,
        agent: &agent,
        prompt_label: "Master password: ",
    };
    let master = master_password(&opts)?;

    let keyfile = record.keyfile_path.clone().map(Keyfile::Path);
    // The probe: open the vault, immediately drop it. The open call
    // produces `VaultError::AuthenticationFailed` (exit 2) on a wrong
    // master password, which is the whole point.
    let _vault = Vault::open(&record.path, &master, keyfile.as_ref()).map_err(CliExit::from)?;

    let view = VaultOpenView {
        id: &args.id,
        status: "unlocked-ok",
    };
    write_success(cli, &view)
}

// ---------------------------------------------------------------------------
// vault list
// ---------------------------------------------------------------------------

fn run_list(cli: &Cli) -> Result<(), CliExit> {
    let paths = resolve_paths(cli)?;
    let registry = VaultRegistry::load(paths).map_err(CliExit::from)?;

    let entries: Vec<VaultListEntry<'_>> = registry
        .list()
        .map(|r| VaultListEntry {
            id: &r.name,
            path: &r.path,
            keyfile: r.keyfile_path.as_deref(),
            created_at: &r.created_at,
            idle_timeout_seconds: read_idle_timeout(r),
        })
        .collect();
    let view = VaultListView { vaults: entries };
    write_success(cli, &view)
}

// ---------------------------------------------------------------------------
// vault set-sync (slot — body in features/sync-git/)
// ---------------------------------------------------------------------------

fn run_set_sync(_args: &VaultSetSyncArgs) -> Result<(), CliExit> {
    Err(CliExit::NotImplemented(
        "runaire vault set-sync — implementation arrives in features/sync-git/",
    ))
}

// ---------------------------------------------------------------------------
// vault set-lock
// ---------------------------------------------------------------------------

fn run_set_lock(cli: &Cli, args: &VaultSetLockArgs) -> Result<(), CliExit> {
    // Exactly one of --timeout / --clear is meaningful. clap's
    // `conflicts_with` blocks both being present; we enforce
    // "exactly one" here (neither set is also a user error).
    if args.timeout.is_none() && !args.clear {
        return Err(CliExit::UserError(
            "vault set-lock requires either --timeout <seconds> or --clear".to_string(),
        ));
    }
    if let Some(0) = args.timeout {
        // When security-behaviors lands it owns the canonical
        // validation. Until then, reject 0 at the CLI layer rather
        // than writing an obviously-broken value to the registry.
        return Err(CliExit::UserError(
            "--timeout must be at least 1 second".to_string(),
        ));
    }

    let paths = resolve_paths(cli)?;
    let mut registry = VaultRegistry::load(paths).map_err(CliExit::from)?;

    // The registry's API returns `&RegisteredVault`. We need owning
    // mutation. Take ownership of the vector via `into_records` is not
    // available; instead, re-read by name, mutate a clone, deregister,
    // re-register. That round-trip preserves all `extra` keys.
    let original = registry
        .get(&args.id)
        .ok_or_else(|| {
            CliExit::from(VaultError::NotRegistered {
                name: args.id.clone(),
            })
        })?
        .clone();

    let mut updated = original;
    apply_idle_timeout(&mut updated, args.timeout);

    registry
        .deregister(&args.id, false)
        .map_err(CliExit::from)?;
    registry.register(updated).map_err(CliExit::from)?;
    registry.save().map_err(CliExit::from)?;

    let view = VaultSetLockView {
        id: &args.id,
        idle_timeout_seconds: args.timeout,
    };
    write_success(cli, &view)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn resolve_paths(cli: &Cli) -> Result<RunairePaths, CliExit> {
    if let Some(registry) = cli.registry.as_deref() {
        let state_dir = registry
            .parent()
            .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf);
        return Ok(RunairePaths::with_state_dir(state_dir));
    }
    RunairePaths::from_env().map_err(CliExit::from)
}

/// Read the `[vault.lock] idle_timeout_seconds` value from the
/// `extra` table, if present. Quiet on malformed data: a non-integer
/// or missing key reads as `None`, never panics.
fn read_idle_timeout(record: &RegisteredVault) -> Option<u64> {
    record
        .extra
        .get(LOCK_TABLE_KEY)
        .and_then(toml::Value::as_table)
        .and_then(|t| t.get(IDLE_TIMEOUT_KEY))
        .and_then(toml::Value::as_integer)
        .and_then(|v| u64::try_from(v).ok())
}

/// Apply the requested change to the `[vault.lock]` sub-table.
///
/// - `Some(s)`: set `idle_timeout_seconds = s`.
/// - `None`: remove the `idle_timeout_seconds` key. If the lock
///   sub-table becomes empty afterwards, drop the table itself so the
///   registry stays clean.
fn apply_idle_timeout(record: &mut RegisteredVault, timeout: Option<u64>) {
    let lock = record
        .extra
        .entry(LOCK_TABLE_KEY.to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    let toml::Value::Table(lock) = lock else {
        // Someone wrote a non-table value here; replace it with a
        // fresh table. Forward-compat tolerance — better than
        // panicking.
        *lock = toml::Value::Table(toml::Table::new());
        let toml::Value::Table(lock) = lock else {
            unreachable!()
        };
        return apply_idle_timeout_inner(lock, timeout);
    };
    apply_idle_timeout_inner(lock, timeout);

    // Tidy up: if the lock table is now empty, drop it from `extra`.
    if record
        .extra
        .get(LOCK_TABLE_KEY)
        .and_then(toml::Value::as_table)
        .is_some_and(toml::Table::is_empty)
    {
        record.extra.remove(LOCK_TABLE_KEY);
    }
}

fn apply_idle_timeout_inner(lock: &mut toml::Table, timeout: Option<u64>) {
    match timeout {
        Some(s) => {
            // `u64` -> `i64` is lossy at >i64::MAX. The CLI accepts
            // `u64` via clap; values that exceed i64::MAX (~9e18) are
            // not real-world idle timeouts. Clamp defensively.
            let as_i64 = i64::try_from(s).unwrap_or(i64::MAX);
            lock.insert(IDLE_TIMEOUT_KEY.to_string(), toml::Value::Integer(as_i64));
        }
        None => {
            lock.remove(IDLE_TIMEOUT_KEY);
        }
    }
}

fn write_success<V>(cli: &Cli, view: &V) -> Result<(), CliExit>
where
    V: serde::Serialize + crate::format::HumanFormat,
{
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut formatter = OutputFormatter::new(stdout.lock(), stderr.lock(), cli.format);
    formatter
        .write(view)
        .map_err(|e| CliExit::Internal(format!("failed to write output: {e}")))
}
