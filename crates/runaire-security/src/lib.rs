//! Rùnaire runtime safety behaviours.
//!
//! This crate owns the three runtime safety behaviours every Rùnaire
//! frontend depends on:
//!
//! - **Idle auto-lock** (FR-051) — [`auto_lock::AutoLockController`]
//!   tracks user activity and produces an [`auto_lock::LockState`] the
//!   frontend reacts to. Landed in Phase 2.
//! - **Clipboard auto-clear** (FR-053) — [`clipboard::Clipboard`]
//!   places a secret on the system clipboard, then a background timer
//!   compares-and-clears after a configurable TTL. Landed in Phase 3.
//! - **OS lock-event sources** (FR-052) — [`os_events::OsLockEventSource`]
//!   trait + concrete `NoopSource` / `SigstopSource` implementations.
//!   Arrives in Phase 4.
//!
//! ## Crate posture
//!
//! - `#![cfg_attr(not(test), forbid(unsafe_code))]` — MVP. The
//!   post-MVP `IoKitSource` (Phase 5b) will flip this to
//!   `deny(unsafe_code)` plus a single locally-`#[allow]`ed module.
//! - No async runtime; the controller is a pure-sync state machine
//!   driven by `tick(now)` from the frontend's event loop.
//! - One public error enum (`SecurityError`). No variant ever carries
//!   secret material; the design's mapping rule is enforced by code
//!   review (see `error.rs`).

#![cfg_attr(not(test), forbid(unsafe_code))]
#![deny(missing_docs)]

pub mod auto_lock;
pub mod clipboard;
pub mod clock;
pub mod error;
pub mod os_events;
pub(crate) mod secret;
pub mod vault_lock;

pub use auto_lock::{
    AutoLockConfig, AutoLockController, LockState, OsLockReason, SecurityEvent,
    DEFAULT_IDLE_TIMEOUT,
};
pub use clipboard::{AutoClearGuard, Clipboard};
pub use clock::{Clock, SystemClock};
pub use error::SecurityError;
#[cfg(all(target_os = "linux", feature = "logind"))]
pub use os_events::LogindSource;
#[cfg(unix)]
pub use os_events::SigstopSource;
pub use os_events::{NoopSource, OsLockEventSource, ShutdownHandle};
pub use vault_lock::VaultLockConfig;
