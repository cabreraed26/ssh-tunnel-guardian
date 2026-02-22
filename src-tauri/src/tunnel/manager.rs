use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

use crate::tunnel::{
    error_classifier,
    health::tcp_check,
    persistence,
    process::{collect_stderr, spawn_ssh},
    state_machine::{backoff_delay_ms, transition, StateEvent},
    types::{
        LogEntry, LogLevel, MetricsPayload, StateChangedPayload, TunnelConfig,
        TunnelError, TunnelErrorKind, TunnelInfo, TunnelState, now_ms,
    },
};

// ─── Internal Tunnel Actor ────────────────────────────────────────────────────

/// A running tunnel's mutable state owned by the manager.
#[allow(dead_code)]
struct TunnelActor {
    pub info: TunnelInfo,
    /// Channel to signal the actor task to shut down.
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl TunnelActor {
    fn new(config: TunnelConfig) -> Self {
        Self::with_id(Uuid::new_v4().to_string(), config)
    }

    /// Restores a previously saved tunnel using its persisted ID.
    fn with_id(id: String, config: TunnelConfig) -> Self {
        Self {
            info: TunnelInfo {
                id,
                config,
                state: TunnelState::Stopped,
                pid: None,
                state_entered_at: now_ms(),
                last_health_check_at: None,
                reconnect_attempts: 0,
                last_error: None,
                uptime_ms: 0,
            },
            shutdown_tx: None,
        }
    }
}

// ─── Tunnel Manager ──────────────────────────────────────────────────────────

/// Maximum log entries kept per tunnel in memory.
const MAX_LOG_ENTRIES: usize = 500;

pub struct TunnelManager {
    tunnels: Arc<Mutex<HashMap<String, TunnelActor>>>,
    logs: Arc<Mutex<HashMap<String, Vec<LogEntry>>>>,
    /// Directory where `tunnels.json` is stored.
    data_dir: PathBuf,
}

impl TunnelManager {
    /// Creates a new manager, loading any previously persisted tunnels.
    pub fn new(data_dir: PathBuf) -> Self {
        let saved = persistence::load(&data_dir);
        let mut tunnels = HashMap::new();
        let mut logs = HashMap::new();
        for (id, config) in saved {
            logs.insert(id.clone(), Vec::new());
            tunnels.insert(id.clone(), TunnelActor::with_id(id, config));
        }
        Self {
            tunnels: Arc::new(Mutex::new(tunnels)),
            logs: Arc::new(Mutex::new(logs)),
            data_dir,
        }
    }

    /// Snapshots all tunnel configs to disk.  Called after every mutation.
    async fn persist(&self) {
        let tunnels = self.tunnels.lock().await;
        let entries: Vec<(String, crate::tunnel::types::TunnelConfig)> = tunnels
            .iter()
            .map(|(id, actor)| (id.clone(), actor.info.config.clone()))
            .collect();
        drop(tunnels);
        persistence::save(&self.data_dir, &entries);
    }

    // ─── CRUD ────────────────────────────────────────────────────────────────

    pub async fn add_tunnel(&self, config: TunnelConfig) -> TunnelInfo {
        let actor = TunnelActor::new(config);
        let info = actor.info.clone();
        let id = info.id.clone();
        self.tunnels.lock().await.insert(id.clone(), actor);
        self.logs.lock().await.insert(id, Vec::new());
        self.persist().await;
        info
    }

    pub async fn remove_tunnel(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        // Stop first if running.
        let _ = self.stop_tunnel(app, id).await;
        {
            let mut tunnels = self.tunnels.lock().await;
            tunnels
                .remove(id)
                .ok_or_else(|| format!("Tunnel {id} not found"))?;
            self.logs.lock().await.remove(id);
        }
        self.persist().await;
        Ok(())
    }

    pub async fn update_tunnel(
        &self,
        app: &AppHandle,
        id: &str,
        config: TunnelConfig,
    ) -> Result<TunnelInfo, String> {
        let running = {
            let tunnels = self.tunnels.lock().await;
            let actor = tunnels.get(id).ok_or_else(|| format!("Tunnel {id} not found"))?;
            actor.info.state != TunnelState::Stopped
        };
        if running {
            self.stop_tunnel(app, id).await?;
        }
        {
            let mut tunnels = self.tunnels.lock().await;
            let actor = tunnels.get_mut(id).ok_or_else(|| format!("Tunnel {id} not found"))?;
            actor.info.config = config;
            actor.info.state = TunnelState::Stopped;
            actor.info.state_entered_at = now_ms();
            actor.info.last_error = None;
            actor.info.reconnect_attempts = 0;
        }
        self.persist().await;
        Ok(self.get_tunnel(id).await.unwrap())
    }

    pub async fn get_tunnels(&self) -> Vec<TunnelInfo> {
        let tunnels = self.tunnels.lock().await;
        tunnels.values().map(|a| a.info.clone()).collect()
    }

    pub async fn get_tunnel(&self, id: &str) -> Option<TunnelInfo> {
        let tunnels = self.tunnels.lock().await;
        tunnels.get(id).map(|a| a.info.clone())
    }

    pub async fn get_logs(&self, id: &str, limit: usize) -> Vec<LogEntry> {
        let logs = self.logs.lock().await;
        if let Some(entries) = logs.get(id) {
            let skip = entries.len().saturating_sub(limit);
            entries[skip..].to_vec()
        } else {
            vec![]
        }
    }

    // ─── Lifecycle ───────────────────────────────────────────────────────────

    pub async fn start_tunnel(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        {
            let tunnels = self.tunnels.lock().await;
            let actor = tunnels.get(id).ok_or_else(|| format!("Tunnel {id} not found"))?;
            // Only Stopped/Failed can be started.
            if !matches!(
                actor.info.state,
                TunnelState::Stopped | TunnelState::Failed
            ) {
                return Err(format!(
                    "Tunnel {} is already {} — stop it first",
                    id, actor.info.state
                ));
            }
        }
        self.spawn_supervisor(app.clone(), id.to_string(), 0).await;
        Ok(())
    }

    pub async fn stop_tunnel(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        let mut tunnels = self.tunnels.lock().await;
        let actor = tunnels.get_mut(id).ok_or_else(|| format!("Tunnel {id} not found"))?;

        // Signal the supervisor to shut down.
        if let Some(tx) = actor.shutdown_tx.take() {
            let _ = tx.send(());
        }
        actor.info.state = TunnelState::Stopped;
        actor.info.pid = None;
        actor.info.state_entered_at = now_ms();
        actor.info.uptime_ms = 0;

        self.emit_state_change(app, id, &TunnelState::Stopped, Some("Stopped by user".into()));
        Ok(())
    }

    /// Signals all running tunnels to shut down without emitting Tauri events.
    /// Called on app exit so SSH child processes don't outlive the application.
    pub async fn stop_all_silent(&self) {
        let mut tunnels = self.tunnels.lock().await;
        for actor in tunnels.values_mut() {
            if let Some(tx) = actor.shutdown_tx.take() {
                let _ = tx.send(());
            }
            actor.info.state = TunnelState::Stopped;
            actor.info.pid = None;
        }
    }

    pub async fn restart_tunnel(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        self.stop_tunnel(app, id).await?;
        // Small delay so the OS can reclaim the port.
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start_tunnel(app, id).await
    }

    // ─── Supervisor Task ─────────────────────────────────────────────────────

    /// Spawns the async supervisor loop for a single tunnel.
    /// The supervisor manages the SSH process lifetime and health checks,
    /// transitioning between states via the state machine.
    async fn spawn_supervisor(&self, app: AppHandle, id: String, initial_attempts: u32) {
        let tunnels_arc = self.tunnels.clone();
        let logs_arc = self.logs.clone();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        // Register the shutdown sender.
        {
            let mut tunnels = tunnels_arc.lock().await;
            if let Some(actor) = tunnels.get_mut(&id) {
                actor.shutdown_tx = Some(shutdown_tx);
                actor.info.reconnect_attempts = initial_attempts;
                Self::_set_state_inner(actor, TunnelState::Starting, &app, &id, None);
            }
        }

        let config = {
            let tunnels = tunnels_arc.lock().await;
            tunnels.get(&id).map(|a| a.info.config.clone())
        };
        let config = match config {
            Some(c) => c,
            None => return,
        };

        tokio::spawn(async move {
            let reconnect_cfg = config.reconnect.clone();
            let health_cfg = config.health_check.clone();

            loop {
                // ── Spawn SSH Process ─────────────────────────────────────────
                Self::_push_log(
                    &logs_arc,
                    &id,
                    LogLevel::Info,
                    format!(
                        "Spawning SSH tunnel: 127.0.0.1:{} → {}:{} via {}@{}",
                        config.local_port,
                        config.remote_host,
                        config.remote_port,
                        config.ssh_user,
                        config.ssh_host
                    ),
                )
                .await;

                let mut child = match spawn_ssh(&config) {
                    Ok(c) => c,
                    Err(err) => {
                        let msg = format!("Failed to spawn SSH: {err}");
                        Self::_push_log(&logs_arc, &id, LogLevel::Error, msg.clone()).await;
                        Self::_update_actor(
                            &tunnels_arc,
                            &id,
                            TunnelState::Failed,
                            &app,
                            Some(TunnelError {
                                kind: TunnelErrorKind::Unknown,
                                message: msg.clone(),
                                occurred_at: now_ms(),
                            }),
                            Some(msg),
                        )
                        .await;
                        return;
                    }
                };

                // Record PID.
                let pid = child.id();
                {
                    let mut tunnels = tunnels_arc.lock().await;
                    if let Some(actor) = tunnels.get_mut(&id) {
                        actor.info.pid = pid;
                    }
                }

                // Wait briefly for the process to potentially die immediately
                // (e.g., port in use), then start health check polling.
                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

                let mut health_failure_streak: u32 = 0;
                let mut health_success_streak: u32 = 0;
                let healthy_since: Option<u64> = None;
                let _ = healthy_since; // will be updated below

                'health_loop: loop {
                    tokio::select! {
                        // ── Shutdown signal ───────────────────────────────────
                        _ = &mut shutdown_rx => {
                            let _ = child.kill().await;
                            return;
                        }

                        // ── Process exits unexpectedly ────────────────────────
                        status = child.wait() => {
                            let exit_status = status.ok();
                            let stderr = collect_stderr(&mut child).await;
                            let kind = error_classifier::classify(&stderr);
                            let msg = if stderr.is_empty() {
                                format!("SSH process exited: {exit_status:?}")
                            } else {
                                stderr.trim().to_string()
                            };

                            Self::_push_log(&logs_arc, &id, LogLevel::Warn, msg.clone()).await;

                            let is_fatal = matches!(
                                kind,
                                TunnelErrorKind::AuthFailure
                                    | TunnelErrorKind::PortInUse
                                    | TunnelErrorKind::PermissionDenied
                            );

                            let current_attempts = {
                                let tunnels = tunnels_arc.lock().await;
                                tunnels.get(&id).map(|a| a.info.reconnect_attempts).unwrap_or(0)
                            };

                            if is_fatal || current_attempts >= reconnect_cfg.max_attempts {
                                let next = if is_fatal {
                                    TunnelState::Failed
                                } else {
                                    TunnelState::Failed
                                };
                                let hint = if is_fatal {
                                    format!("Fatal error ({kind}): manual intervention required")
                                } else {
                                    format!("Max reconnect attempts ({}) exceeded", reconnect_cfg.max_attempts)
                                };
                                Self::_push_log(&logs_arc, &id, LogLevel::Error, hint.clone()).await;
                                Self::_update_actor(
                                    &tunnels_arc,
                                    &id,
                                    next,
                                    &app,
                                    Some(TunnelError { kind, message: msg, occurred_at: now_ms() }),
                                    Some(hint),
                                )
                                .await;
                                return;
                            }

                            // Schedule reconnect.
                            let delay = backoff_delay_ms(
                                current_attempts,
                                reconnect_cfg.initial_delay_ms,
                                reconnect_cfg.max_delay_ms,
                                reconnect_cfg.multiplier,
                            );
                            let reconnect_msg = format!(
                                "Reconnecting in {:.1}s (attempt {}/{})",
                                delay as f64 / 1000.0,
                                current_attempts + 1,
                                reconnect_cfg.max_attempts
                            );
                            Self::_push_log(&logs_arc, &id, LogLevel::Warn, reconnect_msg.clone()).await;
                            Self::_update_actor(
                                &tunnels_arc,
                                &id,
                                TunnelState::Reconnecting,
                                &app,
                                Some(TunnelError { kind, message: msg, occurred_at: now_ms() }),
                                Some(reconnect_msg),
                            )
                            .await;

                            {
                                let mut tunnels = tunnels_arc.lock().await;
                                if let Some(actor) = tunnels.get_mut(&id) {
                                    actor.info.reconnect_attempts += 1;
                                }
                            }

                            // Wait for the backoff delay, watching for shutdown.
                            tokio::select! {
                                _ = &mut shutdown_rx => return,
                                _ = tokio::time::sleep(tokio::time::Duration::from_millis(delay)) => {}
                            }

                            // Re-enter STARTING and re-spawn.
                            {
                                let mut tunnels = tunnels_arc.lock().await;
                                if let Some(actor) = tunnels.get_mut(&id) {
                                    Self::_set_state_inner(
                                        actor,
                                        TunnelState::Starting,
                                        &app,
                                        &id,
                                        None,
                                    );
                                }
                            }
                            break 'health_loop; // Re-enter outer loop to re-spawn.
                        }

                        // ── Periodic health check ─────────────────────────────
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(health_cfg.interval_ms)) => {
                            let ok = tcp_check("127.0.0.1", config.local_port, health_cfg.timeout_ms).await;
                            let now = now_ms();

                            {
                                let mut tunnels = tunnels_arc.lock().await;
                                if let Some(actor) = tunnels.get_mut(&id) {
                                    if ok {
                                        actor.info.last_health_check_at = Some(now);
                                    }
                                }
                            }

                            let current_state = {
                                let tunnels = tunnels_arc.lock().await;
                                tunnels.get(&id).map(|a| a.info.state.clone())
                            };

                            match current_state {
                                Some(TunnelState::Starting) | Some(TunnelState::Healthy) | Some(TunnelState::Degraded) => {
                                    if ok {
                                        health_failure_streak = 0;
                                        health_success_streak += 1;
                                        let current = current_state.unwrap();
                                        if current == TunnelState::Starting || current == TunnelState::Degraded {
                                            if health_success_streak >= health_cfg.recovery_threshold {
                                                health_success_streak = 0;
                                                Self::_push_log(
                                                    &logs_arc,
                                                    &id,
                                                    LogLevel::Info,
                                                    "Health check passing — tunnel is HEALTHY".into(),
                                                )
                                                .await;
                                                let event = if current == TunnelState::Starting {
                                                    StateEvent::HealthCheckPassed
                                                } else {
                                                    StateEvent::HealthCheckPassed
                                                };
                                                if let Some(next) = transition(&current, event) {
                                                    Self::_update_actor(
                                                        &tunnels_arc,
                                                        &id,
                                                        next,
                                                        &app,
                                                        None,
                                                        None,
                                                    )
                                                    .await;
                                                    // Reset reconnect counter on successful recovery.
                                                    let mut tunnels = tunnels_arc.lock().await;
                                                    if let Some(actor) = tunnels.get_mut(&id) {
                                                        actor.info.reconnect_attempts = 0;
                                                    }
                                                }
                                            }
                                        }
                                        // Emit metrics periodically.
                                        Self::_emit_metrics(&tunnels_arc, &id, &app).await;
                                    } else {
                                        health_success_streak = 0;
                                        health_failure_streak += 1;
                                        let current = current_state.unwrap();
                                        if current == TunnelState::Healthy
                                            && health_failure_streak >= health_cfg.failure_threshold
                                        {
                                            health_failure_streak = 0;
                                            let msg = format!(
                                                "Health check failed {} consecutive times — entering DEGRADED",
                                                health_cfg.failure_threshold
                                            );
                                            Self::_push_log(&logs_arc, &id, LogLevel::Warn, msg.clone()).await;
                                            if let Some(next) =
                                                transition(&current, StateEvent::HealthCheckFailed)
                                            {
                                                Self::_update_actor(
                                                    &tunnels_arc,
                                                    &id,
                                                    next,
                                                    &app,
                                                    None,
                                                    Some(msg),
                                                )
                                                .await;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // Outer loop: continue to re-spawn after reconnect.
            }
        });
    }

    // ─── Helpers ─────────────────────────────────────────────────────────────

    fn _set_state_inner(
        actor: &mut TunnelActor,
        state: TunnelState,
        app: &AppHandle,
        id: &str,
        msg: Option<String>,
    ) {
        actor.info.state = state.clone();
        actor.info.state_entered_at = now_ms();
        if state == TunnelState::Healthy {
            actor.info.reconnect_attempts = 0;
        }
        let payload = StateChangedPayload {
            tunnel_id: id.to_string(),
            state,
            message: msg,
            timestamp: now_ms(),
        };
        let _ = app.emit("stg://tunnel-state-changed", &payload);
    }

    async fn _update_actor(
        tunnels_arc: &Arc<Mutex<HashMap<String, TunnelActor>>>,
        id: &str,
        state: TunnelState,
        app: &AppHandle,
        error: Option<TunnelError>,
        msg: Option<String>,
    ) {
        let mut tunnels = tunnels_arc.lock().await;
        if let Some(actor) = tunnels.get_mut(id) {
            actor.info.state = state.clone();
            actor.info.state_entered_at = now_ms();
            if let Some(err) = error {
                actor.info.last_error = Some(err);
            }
            if state == TunnelState::Healthy {
                actor.info.reconnect_attempts = 0;
            }
        }
        let payload = StateChangedPayload {
            tunnel_id: id.to_string(),
            state,
            message: msg,
            timestamp: now_ms(),
        };
        let _ = app.emit("stg://tunnel-state-changed", &payload);
    }

    fn emit_state_change(
        &self,
        app: &AppHandle,
        id: &str,
        state: &TunnelState,
        msg: Option<String>,
    ) {
        let payload = StateChangedPayload {
            tunnel_id: id.to_string(),
            state: state.clone(),
            message: msg,
            timestamp: now_ms(),
        };
        let _ = app.emit("stg://tunnel-state-changed", &payload);
    }

    async fn _push_log(
        logs_arc: &Arc<Mutex<HashMap<String, Vec<LogEntry>>>>,
        id: &str,
        level: LogLevel,
        message: String,
    ) {
        let entry = LogEntry {
            tunnel_id: id.to_string(),
            level,
            message,
            timestamp: now_ms(),
        };
        let mut logs = logs_arc.lock().await;
        let bucket = logs.entry(id.to_string()).or_default();
        bucket.push(entry);
        if bucket.len() > MAX_LOG_ENTRIES {
            bucket.drain(0..bucket.len() - MAX_LOG_ENTRIES);
        }
    }

    async fn _emit_metrics(
        tunnels_arc: &Arc<Mutex<HashMap<String, TunnelActor>>>,
        id: &str,
        app: &AppHandle,
    ) {
        let payload = {
            let tunnels = tunnels_arc.lock().await;
            tunnels.get(id).map(|a| {
                let uptime = if a.info.state == TunnelState::Healthy {
                    now_ms().saturating_sub(a.info.state_entered_at)
                } else {
                    0
                };
                MetricsPayload {
                    tunnel_id: id.to_string(),
                    uptime_ms: uptime,
                    reconnect_attempts: a.info.reconnect_attempts,
                    last_health_check_at: a.info.last_health_check_at,
                    pid: a.info.pid,
                }
            })
        };
        if let Some(p) = payload {
            let _ = app.emit("stg://tunnel-metrics", &p);
        }
    }
}
