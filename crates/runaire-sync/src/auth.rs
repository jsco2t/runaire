//! Authentication strategies and the encrypted-credential entry point.
//!
//! SSH defers to the user's ssh-agent; HTTPS decrypts a master-password-keyed
//! "RST-CRED-1" container just-in-time. **Phase 1 scaffold:** only the public
//! [`encrypt_credential`] entry point exists, as a stub. Credential
//! resolution, the gix credentials provider, and the ChaCha20-Poly1305 /
//! Argon2id container land in Phase 3 (T3.5), which vendors `argon2` and
//! `chacha20poly1305`.

use runaire_core::MasterPassword;

use crate::error::SyncError;

/// Encrypt a freshly-typed HTTPS password into a base64 "RST-CRED-1"
/// container for storage in `vaults.toml`.
///
/// **Phase 1 stub** — implemented in Phase 3 (T3.5).
///
/// # Errors
/// Returns [`SyncError`] if key derivation or encryption fails (Phase 3).
pub fn encrypt_credential(
    _plaintext: &str,
    _master_password: &MasterPassword,
) -> Result<String, SyncError> {
    unimplemented!("Phase 3 — task T3.5 (auth::encrypt_credential / RST-CRED-1)")
}
