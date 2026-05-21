//! `runaire sync` slot — design §3.6.
//!
//! The flag surface is parseable so users see the complete subcommand
//! tree in `runaire --help`; the body returns
//! [`CliExit::NotImplemented`] (exit 11) until `features/sync-git/`
//! ships the real implementation.

use crate::cli::{Cli, SyncArgs};
use crate::exit::CliExit;

/// Phase 1 (and through MVP) entry point — always returns `NotImplemented`.
///
/// # Errors
///
/// Returns [`CliExit::NotImplemented`] until `features/sync-git/` lands.
pub fn run(_cli: &Cli, _args: &SyncArgs) -> Result<(), CliExit> {
    Err(CliExit::NotImplemented(
        "runaire sync — implementation arrives in features/sync-git/",
    ))
}
