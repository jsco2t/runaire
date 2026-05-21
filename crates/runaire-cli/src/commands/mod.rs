//! Per-subcommand dispatch functions.
//!
//! Phase 1 ships stub bodies — each `run` returns
//! [`crate::exit::CliExit::NotImplemented`]. Phases 2–4 replace these
//! bodies in place; the module shape is frozen now so the integration
//! test surface (and any external links into `runaire_cli::commands::*`)
//! does not churn across phases.

pub mod completions;
pub mod entry;
pub mod gen;
pub mod ssh;
pub mod sync;
pub mod vault;
