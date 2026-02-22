mod commands;
mod tunnel;

use tauri::Manager;
use commands::tunnel_commands::*;
use tunnel::manager::TunnelManager;

/// Shared application state injected into Tauri commands.
pub struct AppState {
    pub manager: TunnelManager,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState {
                manager: TunnelManager::new(data_dir),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Read
            get_tunnels,
            get_tunnel,
            get_tunnel_logs,
            // Write
            add_tunnel,
            remove_tunnel,
            update_tunnel,
            // Lifecycle
            start_tunnel,
            stop_tunnel,
            restart_tunnel,
        ])
        .build(tauri::generate_context!())
        .expect("error while running SSH Tunnel Guardian")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Kill all SSH child processes before the app exits so they
                // don't keep holding local ports after Cmd+Q / window close.
                let state = app_handle.state::<AppState>();
                tauri::async_runtime::block_on(state.manager.stop_all_silent());
            }
        });
}

