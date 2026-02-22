use tauri::State;

use crate::connections::types::{SshConnection, SshConnectionConfig};
use crate::AppState;

// ─── Read ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_connections(state: State<'_, AppState>) -> Result<Vec<SshConnection>, String> {
    Ok(state.connections.list().await)
}

#[tauri::command]
pub async fn get_connection(
    id: String,
    state: State<'_, AppState>,
) -> Result<SshConnection, String> {
    state
        .connections
        .get(&id)
        .await
        .ok_or_else(|| format!("Connection '{id}' not found"))
}

// ─── Write ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn add_connection(
    config: SshConnectionConfig,
    state: State<'_, AppState>,
) -> Result<SshConnection, String> {
    Ok(state.connections.add(config).await)
}

#[tauri::command]
pub async fn remove_connection(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.connections.remove(&id).await
}

#[tauri::command]
pub async fn update_connection(
    id: String,
    config: SshConnectionConfig,
    state: State<'_, AppState>,
) -> Result<SshConnection, String> {
    state.connections.update(&id, config).await
}

// ─── Launch ───────────────────────────────────────────────────────────────────

/// Opens the SSH connection in the system's native terminal application.
#[tauri::command]
pub async fn launch_connection(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.connections.launch(&id).await
}
// ── Keychain ───────────────────────────────────────────────────────────

/// Stores a password in the OS keychain for the given connection.
/// Pass an empty string to erase the stored password.
#[tauri::command]
pub async fn save_connection_password(
    id: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if password.is_empty() {
        state.connections.delete_password(&id);
        Ok(())
    } else {
        state.connections.save_password(&id, &password)
    }
}

/// Removes a stored password from the OS keychain.
#[tauri::command]
pub async fn delete_connection_password(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.connections.delete_password(&id);
    Ok(())
}