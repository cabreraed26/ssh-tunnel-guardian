//! Thin wrapper around the OS keychain (macOS Keychain, Linux Secret Service,
//! Windows Credential Store) via the `keyring` crate.
//!
//! Passwords for SSH connections are stored under the service name
//! `stg-connections` with the connection ID as the username key.

const SERVICE: &str = "ssh-tunnel-guardian-connections";

/// Saves (or overwrites) a password for the given connection ID.
pub fn save(conn_id: &str, password: &str) -> Result<(), String> {
    keyring::Entry::new(SERVICE, conn_id)
        .map_err(|e| format!("Keychain error: {e}"))?
        .set_password(password)
        .map_err(|e| format!("Keychain write error: {e}"))
}

/// Retrieves the stored password, if any.
pub fn get(conn_id: &str) -> Option<String> {
    keyring::Entry::new(SERVICE, conn_id)
        .ok()?
        .get_password()
        .ok()
}

/// Deletes the stored password. Ignores "not found" errors silently.
pub fn delete(conn_id: &str) {
    if let Ok(entry) = keyring::Entry::new(SERVICE, conn_id) {
        let _ = entry.delete_credential();
    }
}

/// Returns true if a password exists for the given connection ID.
pub fn exists(conn_id: &str) -> bool {
    get(conn_id).is_some()
}
