mod commands;
mod connections;
mod tunnel;

use tauri::Manager;
use commands::tunnel_commands::*;
use commands::connection_commands::*;
use tunnel::manager::TunnelManager;
use connections::ConnectionsManager;

/// Shared application state injected into Tauri commands.
pub struct AppState {
    pub manager: TunnelManager,
    pub connections: ConnectionsManager,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState {
                manager: TunnelManager::new(data_dir.clone()),
                connections: ConnectionsManager::new(data_dir),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tunnel — read
            get_tunnels,
            get_tunnel,
            get_tunnel_logs,
            // Tunnel — write
            add_tunnel,
            remove_tunnel,
            update_tunnel,
            // Tunnel — lifecycle
            start_tunnel,
            stop_tunnel,
            restart_tunnel,
            // Connection — CRUD
            get_connections,
            get_connection,
            add_connection,
            remove_connection,
            update_connection,
            // Connection — launch
            launch_connection,
            // Connection — keychain
            save_connection_password,
            delete_connection_password,
        ])
        .build(tauri::generate_context!())
        .expect("error while running SSH Tunnel Guardian")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                // Block the default exit — we'll call app_handle.exit(0)
                // ourselves once every SSH process is confirmed dead.
                api.prevent_exit();

                let app = app_handle.clone();
                std::thread::spawn(move || {
                    let state = app.state::<AppState>();

                    // 1. Send shutdown signals + SIGKILL all tracked PIDs.
                    //    block_on is safe here because we're on a plain OS thread,
                    //    not inside the tokio runtime.
                    let killed_pids =
                        tauri::async_runtime::block_on(state.manager.stop_all_silent());

                    // 2. Wait until every killed PID has actually exited.
                    //    `kill -0 <pid>` returns an error once the process is gone.
                    //    We poll for up to 3 seconds to avoid hanging forever.
                    let deadline = std::time::Instant::now()
                        + std::time::Duration::from_secs(3);
                    for pid in killed_pids {
                        while std::time::Instant::now() < deadline {
                            let alive = std::process::Command::new("kill")
                                .args(["-0", &pid.to_string()])
                                .status()
                                .map(|s| s.success())
                                .unwrap_or(false);
                            if !alive {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                    }

                    // 3. All SSH processes are gone — safe to exit.
                    app.exit(0);
                });
            }
        });
}

