//! `runaire` binary — thin shim over [`runaire_cli::run`].

fn main() -> std::process::ExitCode {
    runaire_cli::run()
}
