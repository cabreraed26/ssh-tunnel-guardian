use tauri::{AppHandle, State};

use crate::tunnel::types::{LogEntry, TunnelConfig, TunnelInfo};
use crate::AppState;

// ─── Read ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_tunnels(state: State<'_, AppState>) -> Result<Vec<TunnelInfo>, String> {
    Ok(state.manager.get_tunnels().await)
}

#[tauri::command]
pub async fn get_tunnel(id: String, state: State<'_, AppState>) -> Result<TunnelInfo, String> {
    state
        .manager
        .get_tunnel(&id)
        .await
        .ok_or_else(|| format!("Tunnel {id} not found"))
}

#[tauri::command]
pub async fn get_tunnel_logs(
    id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntry>, String> {
    Ok(state.manager.get_logs(&id, limit.unwrap_or(100)).await)
}

// ─── Write ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn add_tunnel(
    config: TunnelConfig,
    state: State<'_, AppState>,
) -> Result<TunnelInfo, String> {
    Ok(state.manager.add_tunnel(config).await)
}

#[tauri::command]
pub async fn remove_tunnel(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.manager.remove_tunnel(&app, &id).await
}

#[tauri::command]
pub async fn update_tunnel(
    id: String,
    config: TunnelConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TunnelInfo, String> {
    state.manager.update_tunnel(&app, &id, config).await
}

// ─── Lifecycle ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_tunnel(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.manager.start_tunnel(&app, &id).await
}

#[tauri::command]
pub async fn stop_tunnel(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.manager.stop_tunnel(&app, &id).await
}

#[tauri::command]
pub async fn restart_tunnel(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.manager.restart_tunnel(&app, &id).await
}
