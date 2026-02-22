use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ─── Tunnel States ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TunnelState {
    /// Process is being spawned; not yet verified alive.
    Starting,
    /// Process is running and TCP health check is passing.
    Healthy,
    /// Process alive but health check failing (port not responding).
    Degraded,
    /// Process died; attempting reconnect with exponential backoff.
    Reconnecting,
    /// Exceeded max reconnect attempts; requires manual intervention.
    Failed,
    /// Manually stopped; will not auto-reconnect.
    Stopped,
}

impl std::fmt::Display for TunnelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelState::Starting => write!(f, "STARTING"),
            TunnelState::Healthy => write!(f, "HEALTHY"),
            TunnelState::Degraded => write!(f, "DEGRADED"),
            TunnelState::Reconnecting => write!(f, "RECONNECTING"),
            TunnelState::Failed => write!(f, "FAILED"),
            TunnelState::Stopped => write!(f, "STOPPED"),
        }
    }
}

// ─── Error Classification ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TunnelErrorKind {
    BrokenPipe,
    ConnectionTimeout,
    AuthFailure,
    PortInUse,
    HostUnreachable,
    PermissionDenied,
    UnknownHost,
    NetworkUnreachable,
    Unknown,
}

impl std::fmt::Display for TunnelErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TunnelErrorKind::BrokenPipe => "Broken pipe",
            TunnelErrorKind::ConnectionTimeout => "Connection timeout",
            TunnelErrorKind::AuthFailure => "Authentication failure",
            TunnelErrorKind::PortInUse => "Local port already in use",
            TunnelErrorKind::HostUnreachable => "Host unreachable",
            TunnelErrorKind::PermissionDenied => "Permission denied",
            TunnelErrorKind::UnknownHost => "Unknown host",
            TunnelErrorKind::NetworkUnreachable => "Network unreachable",
            TunnelErrorKind::Unknown => "Unknown error",
        };
        write!(f, "{s}")
    }
}

// ─── Tunnel Configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    /// Human-readable label for this tunnel.
    pub name: String,
    /// SSH user@host target.
    pub ssh_host: String,
    /// SSH port (default 22).
    pub ssh_port: u16,
    /// SSH username.
    pub ssh_user: String,
    /// Local bind port for the forwarded tunnel.
    pub local_port: u16,
    /// Remote host to forward to (from the perspective of the SSH server).
    pub remote_host: String,
    /// Remote port to forward to.
    pub remote_port: u16,
    /// Optional: path to identity file (-i flag).
    pub identity_file: Option<String>,
    /// Optional: SSH password used via sshpass (requires sshpass installed).
    /// NOTE: stored in plaintext — use key-based auth for production.
    pub ssh_password: Option<String>,
    /// When false, adds -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null.
    /// Useful for ephemeral/dev servers. Default: true.
    #[serde(default = "default_true")]
    pub strict_host_checking: bool,
    /// Optional: extra SSH flags (e.g. ["-o", "StrictHostKeyChecking=no"]).
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Reconnect settings.
    #[serde(default)]
    pub reconnect: ReconnectConfig,
    /// Health check settings.
    #[serde(default)]
    pub health_check: HealthCheckConfig,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconnectConfig {
    /// Maximum number of reconnect attempts before entering FAILED.
    pub max_attempts: u32,
    /// Initial backoff delay in milliseconds.
    pub initial_delay_ms: u64,
    /// Maximum backoff delay in milliseconds.
    pub max_delay_ms: u64,
    /// Backoff multiplier.
    pub multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            initial_delay_ms: 1_000,
            max_delay_ms: 60_000,
            multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckConfig {
    /// Interval between TCP health checks in milliseconds.
    pub interval_ms: u64,
    /// TCP connect timeout in milliseconds.
    pub timeout_ms: u64,
    /// Consecutive failures before transitioning to DEGRADED.
    pub failure_threshold: u32,
    /// Consecutive successes after DEGRADED before back to HEALTHY.
    pub recovery_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_ms: 5_000,
            timeout_ms: 3_000,
            failure_threshold: 3,
            recovery_threshold: 2,
        }
    }
}

// ─── Runtime Tunnel Info ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInfo {
    pub id: String,
    pub config: TunnelConfig,
    pub state: TunnelState,
    pub pid: Option<u32>,
    /// Unix timestamp (ms) when the current state was entered.
    pub state_entered_at: u64,
    /// Unix timestamp (ms) of last successful health check.
    pub last_health_check_at: Option<u64>,
    /// Total number of reconnect attempts in the current lifecycle.
    pub reconnect_attempts: u32,
    /// Last classified error, if any.
    pub last_error: Option<TunnelError>,
    /// Uptime in milliseconds since last HEALTHY transition.
    pub uptime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelError {
    pub kind: TunnelErrorKind,
    pub message: String,
    pub occurred_at: u64,
}

// ─── Log Entry ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub tunnel_id: String,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// ─── Tauri Events ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChangedPayload {
    pub tunnel_id: String,
    pub state: TunnelState,
    pub message: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsPayload {
    pub tunnel_id: String,
    pub uptime_ms: u64,
    pub reconnect_attempts: u32,
    pub last_health_check_at: Option<u64>,
    pub pid: Option<u32>,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}
