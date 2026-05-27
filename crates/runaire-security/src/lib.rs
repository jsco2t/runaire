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
//! - `#![cfg_attr(not(test), deny(unsafe_code))]` — the crate is
//!   `unsafe`-free except for one locally-`#[allow]`ed module,
//!   [`os_events::macos`] (Phase 5b `IoKitSource`), which makes the
//!   `IOKit` / `NSWorkspace` FFI calls. This mirrors the pattern
//!   `runaire-core` reserves for its future `mlock` block: `deny`
//!   crate-wide, `#[allow(unsafe_code)]` on exactly the one audited
//!   module. The flip from MVP's `forbid` to `deny` is irreversible
//!   (design §3.12, Phase 5 risk #6); a `forbid` would make the
//!   module-level `#[allow]` a hard error.
//! - No async runtime; the controller is a pure-sync state machine
//!   driven by `tick(now)` from the frontend's event loop.
//! - One public error enum (`SecurityError`). No variant ever carries
//!   secret material; the design's mapping rule is enforced by code
//!   review (see `error.rs`).

#![cfg_attr(not(test), deny(unsafe_code))]
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
#[cfg(all(target_os = "macos", feature = "iokit"))]
pub use os_events::IoKitSource;
#[cfg(all(target_os = "linux", feature = "logind"))]
pub use os_events::LogindSource;
#[cfg(unix)]
pub use os_events::SigstopSource;
pub use os_events::{NoopSource, OsLockEventSource, ShutdownHandle};
pub use vault_lock::VaultLockConfig;
