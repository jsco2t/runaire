//! Shared helpers for `runaire-cli` integration tests.
//!
//! Uses `std::process::Command` directly with the `CARGO_BIN_EXE_runaire`
//! env var Cargo sets for tests, avoiding the `assert_cmd` +
//! `predicates` dependency tree (~15 transitives).

#![allow(dead_code)] // helpers, not all tests use every one

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use tempfile::TempDir;

/// Path to the `runaire` test binary Cargo built for us.
///
/// `CARGO_BIN_EXE_runaire` is set by Cargo's test harness; it does not
/// require the binary to be on `PATH`.
pub fn runaire_bin() -> &'static str {
    env!("CARGO_BIN_EXE_runaire")
}

/// Build a `Command` for the `runaire` binary with an isolated env so
/// host `RUNAIRE_*` settings cannot leak into the test.
pub fn runaire_cmd() -> Command {
    let mut cmd = Command::new(runaire_bin());
    // `env_clear` would lose `PATH` which clap doesn't need but
    // libraries occasionally do — instead, strip only the runaire-
    // specific vars and leave everything else alone.
    cmd.env_remove("RUNAIRE_MASTER_PASSWORD");
    cmd.env_remove("RUNAIRE_STATE_DIR");
    cmd
}

/// Run a `runaire` invocation with the given args and return
/// `(exit_code, stdout, stderr)`.
pub fn run_args(args: &[&str]) -> (i32, String, String) {
    let output = runaire_cmd()
        .args(args)
        .output()
        .expect("failed to spawn runaire binary");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (code, stdout, stderr)
}

/// A tempdir holding a `vaults.toml` path Rùnaire can be pointed at via
/// the global `--registry` flag.
///
/// `_tempdir` is held to keep the directory alive for the lifetime of
/// the test. `vaults_toml` is the path to pass with `--registry`.
pub struct VaultsToml {
    pub tempdir: TempDir,
    pub vaults_toml: PathBuf,
}

impl VaultsToml {
    /// Create a fresh tempdir + plan the `vaults.toml` path inside it.
    /// The file itself is not created; the CLI will create it lazily
    /// on first registry save.
    pub fn new() -> Self {
        let tempdir = TempDir::new().expect("create tempdir");
        let vaults_toml = tempdir.path().join("vaults.toml");
        Self {
            tempdir,
            vaults_toml,
        }
    }

    /// `--registry <path>` argument as a `String` (so it can be
    /// borrowed by `Command::arg`).
    pub fn registry_arg(&self) -> String {
        self.vaults_toml.display().to_string()
    }
}

/// Run `runaire` with `--registry <path>` pointed at `reg`, feeding
/// `stdin_input` on stdin. Returns `(exit_code, stdout, stderr)`.
pub fn run_with_stdin(reg: &VaultsToml, args: &[&str], stdin_input: &str) -> (i32, String, String) {
    let mut cmd = runaire_cmd();
    cmd.args(["--registry", &reg.registry_arg()])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn runaire binary");
    if !stdin_input.is_empty() {
        let stdin = child.stdin.as_mut().expect("captured stdin");
        stdin
            .write_all(stdin_input.as_bytes())
            .expect("write stdin");
    }
    let output = child.wait_with_output().expect("wait for runaire");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (code, stdout, stderr)
}

/// Convenience: run with `--registry` pointed at `reg`, no stdin input.
pub fn run_in(reg: &VaultsToml, args: &[&str]) -> (i32, String, String) {
    run_with_stdin(reg, args, "")
}
