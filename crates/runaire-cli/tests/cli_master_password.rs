//! Phase 4 (T4.3) integration tests for the master-password collection
//! contract (FR-061 + design §2.6).
//!
//! All tests in this file are `#[ignore]`d because they either mutate
//! the spawned child's environment (`RUNAIRE_MASTER_PASSWORD`) or
//! exercise the `--master-password`-flag-doesn't-exist rule via the
//! parse layer — running them in parallel with the rest of the
//! integration suite would pollute observation of the env-var warning.
//! Invoked by `make test-ignored`.
//!
//! The TTY no-echo verification cannot be expressed in a portable
//! Rust test without a PTY harness; it is covered by the manual
//! verification document `verifications/01-local/03-master-password.md`
//! per design §6 row 9.

mod common;

use std::io::Write as _;
use std::process::{Command, Stdio};

use common::{runaire_bin, VaultsToml};

/// Build a `Command` for the `runaire` binary that **does not** strip
/// `RUNAIRE_MASTER_PASSWORD` from the spawned environment, so we can
/// verify the runtime warn-and-remove path. `common::runaire_cmd`
/// strips the env var for normal tests; this helper is the exception.
fn runaire_cmd_keep_env() -> Command {
    Command::new(runaire_bin())
}

/// Helper: create a fresh vault registered as `mp` with master password
/// `correct` inside `reg`. Returns the registry handle for further use.
fn create_fixture_vault(reg: &VaultsToml, password: &str) {
    let path = reg.tempdir.path().join("mp.kdbx");
    let mut cmd = Command::new(runaire_bin());
    cmd.env_remove("RUNAIRE_MASTER_PASSWORD")
        .env_remove("RUNAIRE_STATE_DIR")
        .args([
            "--registry",
            &reg.registry_arg(),
            "vault",
            "create",
            "--id",
            "mp",
            "--path",
            path.to_str().unwrap(),
            "--no-recovery-warning",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn vault create");
    {
        let stdin = child.stdin.as_mut().expect("captured stdin");
        // create prompts for the password twice (confirm pattern).
        let input = format!("{password}\n{password}\n");
        stdin
            .write_all(input.as_bytes())
            .expect("write fixture password to stdin");
    }
    let output = child.wait_with_output().expect("wait for vault create");
    assert!(
        output.status.success(),
        "fixture vault create failed: stderr={}\nstdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout),
    );
}

#[test]
#[ignore = "mutates RUNAIRE_MASTER_PASSWORD on the spawned child; run via `make test-ignored`"]
fn env_var_warns_to_stderr_then_is_ignored_for_open() {
    // The runtime contract: if RUNAIRE_MASTER_PASSWORD is set when the
    // CLI starts, we (a) emit a warning to stderr, (b) remove it from
    // the process environment, and (c) fall through to the secure
    // stdin prompt. Verify the end-to-end behaviour: a correct
    // password piped via stdin still unlocks the vault, the env-var
    // value is ignored, and the warning lands in stderr.
    let reg = VaultsToml::new();
    create_fixture_vault(&reg, "correct");

    let mut cmd = runaire_cmd_keep_env();
    cmd.env("RUNAIRE_MASTER_PASSWORD", "leak-me-if-you-can")
        .env_remove("RUNAIRE_STATE_DIR")
        .args([
            "--registry",
            &reg.registry_arg(),
            "vault",
            "open",
            "--id",
            "mp",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn runaire vault open");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"correct\n")
        .expect("write stdin");
    let output = child
        .wait_with_output()
        .expect("wait for runaire vault open");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "exit code should be 0; stderr was:\n{stderr}"
    );
    assert!(
        stderr.contains("RUNAIRE_MASTER_PASSWORD") && stderr.contains("ignored"),
        "expected warn-and-ignore line in stderr; got:\n{stderr}"
    );
    // Defence-in-depth: the warning text must NOT echo the env-var's
    // value — that would defeat the leak-prevention contract.
    assert!(
        !stderr.contains("leak-me-if-you-can"),
        "stderr leaked the env-var value: {stderr:?}"
    );
}

#[test]
#[ignore = "mutates RUNAIRE_MASTER_PASSWORD on the spawned child; run via `make test-ignored`"]
fn env_var_value_is_not_used_as_password() {
    // When RUNAIRE_MASTER_PASSWORD is set to a non-matching value AND
    // the stdin prompt receives the WRONG password, the env-var value
    // must not be used as a fallback. The expected behaviour is auth
    // failure (exit 2) for the wrong stdin value — the env var is
    // ignored, full stop.
    let reg = VaultsToml::new();
    create_fixture_vault(&reg, "correct");

    let mut cmd = runaire_cmd_keep_env();
    cmd.env("RUNAIRE_MASTER_PASSWORD", "correct")
        .env_remove("RUNAIRE_STATE_DIR")
        .args([
            "--registry",
            &reg.registry_arg(),
            "vault",
            "open",
            "--id",
            "mp",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn runaire vault open");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"wrong\n")
        .expect("write stdin");
    let output = child
        .wait_with_output()
        .expect("wait for runaire vault open");

    // If the env-var value were used as a fallback, this would
    // succeed (because `correct` does match). It must fail with
    // exit 2 (vault.locked).
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected auth-failure exit 2 — env var must not be used; \
         stderr was:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "grouped with the env-var tests in the test-ignored tier"]
fn no_master_password_flag_is_rejected_by_clap() {
    // FR-061 structural gate: there is no `--master-password` flag
    // anywhere. clap rejects the unknown argument with its standard
    // exit code 2 (per the design's two-meanings-for-exit-2 note in
    // §2.4.2 — distinct from CliExit::VaultLocked which also uses 2,
    // but produced by clap's parser rather than the runtime).
    let output = Command::new(runaire_bin())
        .env_remove("RUNAIRE_MASTER_PASSWORD")
        .env_remove("RUNAIRE_STATE_DIR")
        .args(["vault", "open", "--id", "x", "--master-password=bypass"])
        .output()
        .expect("spawn runaire");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(
        output.status.code(),
        Some(2),
        "clap should reject the unknown flag with exit 2; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("--master-password") || stderr.to_lowercase().contains("unexpected"),
        "stderr should name the rejected argument:\n{stderr}"
    );
    // Defence-in-depth: the value MUST NOT leak into stderr.
    assert!(
        !stderr.contains("bypass"),
        "stderr leaked the would-be password value: {stderr:?}"
    );
}

#[test]
#[ignore = "spawns the binary repeatedly; serialized with the env-var tests in test-ignored"]
fn rpassword_replacement_uses_no_echo_path() {
    // The TTY no-echo path runs inside `read_password_no_echo` (see
    // `crates/runaire-cli/src/prompt.rs`) and toggles termios via
    // `nix::sys::termios::tcsetattr`. Asserting the actual echo state
    // requires a PTY harness, which is out of scope for CI per design
    // §6 row 9. The unit tests in `prompt.rs::tests` exercise the
    // pipe-fallback path (no echo to toggle); the manual verification
    // document `verifications/01-local/03-master-password.md` covers
    // the real-TTY case.
    //
    // What we CAN verify here is the structural invariant: the prompt
    // function is reachable via the `vault open` happy path with
    // stdin piped. If this passes, the prompt path itself ran.
    let reg = VaultsToml::new();
    create_fixture_vault(&reg, "correct");

    let mut cmd = runaire_cmd_keep_env();
    cmd.env_remove("RUNAIRE_MASTER_PASSWORD")
        .env_remove("RUNAIRE_STATE_DIR")
        .args([
            "--registry",
            &reg.registry_arg(),
            "vault",
            "open",
            "--id",
            "mp",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn runaire vault open");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"correct\n")
        .expect("write stdin");
    let output = child
        .wait_with_output()
        .expect("wait for runaire vault open");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "happy-path open should succeed; stderr was:\n{stderr}"
    );
    // The prompt label written to stderr proves the prompt path ran.
    assert!(
        stderr.contains("Master password:"),
        "expected the prompt label to appear in stderr:\n{stderr}"
    );
}
