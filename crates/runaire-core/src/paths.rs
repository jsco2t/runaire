//! Resolve the canonical Rùnaire state directory and the `vaults.toml`
//! location.
//!
//! Per design §2.2.1 and PRD Decision #16, the state directory is
//! `$HOME/.local/state/runaire/` on both macOS and Linux — deliberately
//! consistent across platforms, deliberately NOT following macOS
//! `Application Support/`. We resolve `$HOME` directly instead of using
//! the `dirs` crate because `dirs::state_dir()` returns
//! `~/Library/Application Support/...` on macOS, which would contradict
//! the project's "same path everywhere" promise and could silently
//! diverge from registries users created on Linux.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::error::VaultError;

/// Resolved paths for a Rùnaire installation.
///
/// All filesystem operations in `runaire-core` route their path
/// resolution through this type. Construct via [`Self::from_env`] in
/// production code, or [`Self::with_state_dir`] in tests.
#[derive(Debug, Clone)]
pub struct RunairePaths {
    state_dir: PathBuf,
}

impl RunairePaths {
    /// Resolve the state directory from `$HOME`.
    ///
    /// Returns [`VaultError::HomeUnresolvable`] if the `HOME` environment
    /// variable is absent or empty. The library must not panic in
    /// CI / containers / headless services where `$HOME` is missing.
    pub fn from_env() -> Result<Self, VaultError> {
        // Read once and delegate to the env-free helper so the body's
        // logic (empty-string filtering, path composition) is unit-
        // testable without mutating process env vars — env mutation is
        // `unsafe` in Rust 1.85+ and races concurrent `getenv` calls
        // anywhere else in the process.
        let home = std::env::var_os("HOME");
        Self::from_home_value(home.as_deref())
    }

    /// Resolve from a pre-fetched optional `HOME` value.
    ///
    /// `None` and `Some("")` both produce [`VaultError::HomeUnresolvable`].
    /// Crate-private so tests can exercise the resolution logic without
    /// touching process environment state.
    pub(crate) fn from_home_value(home: Option<&OsStr>) -> Result<Self, VaultError> {
        let home = home
            .filter(|v| !v.is_empty())
            .ok_or(VaultError::HomeUnresolvable)?;
        Ok(Self::from_home(home))
    }

    /// Compose the state directory path from a known-good `HOME` value.
    ///
    /// `home` must be non-empty; callers (`from_env`, `from_home_value`)
    /// enforce this. Crate-private — production callers go through
    /// `from_env`; tests use `from_home_value` or `with_state_dir`.
    fn from_home(home: &OsStr) -> Self {
        let mut state_dir = PathBuf::from(home);
        state_dir.push(".local");
        state_dir.push("state");
        state_dir.push("runaire");
        Self { state_dir }
    }

    /// Construct from an explicit state directory.
    ///
    /// Used by tests to root the state dir inside a `tempfile::TempDir`.
    /// Not for production use.
    pub fn with_state_dir(state_dir: PathBuf) -> Self {
        Self { state_dir }
    }

    /// The resolved state directory (`$HOME/.local/state/runaire`).
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// The vault-registry file path (`<state_dir>/vaults.toml`).
    pub fn vaults_toml(&self) -> PathBuf {
        self.state_dir.join("vaults.toml")
    }

    /// Ensure the state directory exists with mode `0700` (POSIX).
    ///
    /// Idempotent — safe to call from multiple call sites (registry
    /// load, vault create). If the directory already exists, its
    /// permissions are not modified (re-tightening on every load would
    /// surprise users who deliberately loosened it for, e.g., a backup
    /// process; loosened state dirs are flagged elsewhere, not silently
    /// reset).
    pub fn ensure_exists(&self) -> Result<(), VaultError> {
        if self.state_dir.exists() {
            return Ok(());
        }

        std::fs::create_dir_all(&self.state_dir).map_err(|source| VaultError::Io {
            source,
            path: self.state_dir.clone(),
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&self.state_dir, perms).map_err(|source| VaultError::Io {
                source,
                path: self.state_dir.clone(),
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use tempfile::TempDir;

    // -------------------------------------------------------------------
    // Pure resolution-logic tests — no env mutation, no `unsafe`.
    //
    // These exercise `from_home_value` directly. Together they cover the
    // three branches of `from_env`'s body (HOME set, HOME absent, HOME
    // empty) without touching process env state, which is `unsafe` in
    // Rust 1.85+ and would race concurrent `getenv` calls elsewhere in
    // the test process (notably `tempfile::TempDir::new` reading
    // `TMPDIR`).
    // -------------------------------------------------------------------

    #[test]
    fn from_home_value_returns_state_dir_under_home_when_present() {
        let paths =
            RunairePaths::from_home_value(Some(OsStr::new("/tmp/jasontest"))).expect("HOME set");
        assert_eq!(
            paths.state_dir(),
            Path::new("/tmp/jasontest/.local/state/runaire"),
        );
    }

    #[test]
    fn from_home_value_returns_err_when_none() {
        let err = RunairePaths::from_home_value(None).expect_err("HOME absent");
        assert!(matches!(err, VaultError::HomeUnresolvable));
    }

    #[test]
    fn from_home_value_returns_err_when_empty() {
        let err = RunairePaths::from_home_value(Some(OsStr::new(""))).expect_err("HOME empty");
        assert!(matches!(err, VaultError::HomeUnresolvable));
    }

    #[test]
    fn vaults_toml_is_state_dir_slash_vaults_toml() {
        let paths = RunairePaths::with_state_dir(PathBuf::from("/some/state/dir"));
        assert_eq!(
            paths.vaults_toml(),
            PathBuf::from("/some/state/dir/vaults.toml")
        );
    }

    #[cfg(unix)]
    #[test]
    fn ensure_exists_creates_with_mode_0700() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().expect("create tempdir");
        let state = tmp.path().join("state-dir-fresh");
        let paths = RunairePaths::with_state_dir(state.clone());

        paths.ensure_exists().expect("ensure_exists succeeds");
        assert!(state.is_dir(), "state dir should exist");

        let mode = std::fs::metadata(&state)
            .expect("stat state dir")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700, "state dir must be created with mode 0700");
    }

    #[test]
    fn ensure_exists_is_idempotent() {
        let tmp = TempDir::new().expect("create tempdir");
        let state = tmp.path().join("state-dir-idempotent");
        let paths = RunairePaths::with_state_dir(state.clone());

        paths.ensure_exists().expect("first call");
        paths.ensure_exists().expect("second call is a no-op");
        assert!(state.is_dir(), "state dir still exists after two calls");
    }

    // -------------------------------------------------------------------
    // Single env-mutating smoke test for `from_env` itself.
    //
    // The unit tests above prove the resolution logic; this one confirms
    // that `from_env` correctly reads `HOME` and passes the value
    // through. It is marked `#[ignore]` because env mutation in `cargo
    // test`'s default parallel mode is racy against any other test in
    // the process that reads env vars (e.g., `TempDir::new` reading
    // `TMPDIR`). Run it explicitly when needed:
    //
    //     cargo test -- --ignored --test-threads=1
    //
    // The `EnvGuard` helper below restores `HOME` on drop so this test
    // is safe to run repeatedly and does not leak state.
    // -------------------------------------------------------------------

    /// RAII guard that saves the current value of `HOME`, replaces it,
    /// and restores the original on drop.
    ///
    /// SAFETY note for future maintainers: `std::env::set_var` is
    /// `unsafe` in Rust 1.85+ because it races concurrent `getenv` calls
    /// anywhere in the process. There is no in-process synchronization
    /// that can fully eliminate this race — `getenv` callers in other
    /// crates (notably libc-level callers and `tempfile`'s `TMPDIR`
    /// resolution) do not consult any application-level mutex. Marking
    /// the single test that uses this helper `#[ignore]` keeps it out
    /// of the default-parallel test run; explicit serial runs
    /// (`--test-threads=1`) provide the safety guarantee.
    #[allow(unsafe_code)] // documented above; only used by an #[ignore]d test
    struct EnvGuard {
        var: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        #[allow(unsafe_code)]
        fn set(var: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(var);
            // SAFETY: see struct-level note. Caller is responsible for
            // ensuring the test is run with `--test-threads=1`.
            unsafe {
                std::env::set_var(var, value);
            }
            Self { var, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            #[allow(unsafe_code)]
            // SAFETY: see struct-level note.
            unsafe {
                match self.previous.take() {
                    Some(v) => std::env::set_var(self.var, v),
                    None => std::env::remove_var(self.var),
                }
            }
        }
    }

    #[test]
    #[ignore = "mutates HOME; run with `cargo test -- --ignored --test-threads=1`"]
    fn from_env_smoke_reads_home_from_process_environment() {
        let _guard = EnvGuard::set("HOME", "/tmp/jasontest-smoke");
        let paths = RunairePaths::from_env().expect("HOME set by guard");
        assert_eq!(
            paths.state_dir(),
            Path::new("/tmp/jasontest-smoke/.local/state/runaire"),
        );
    }
}
