//! Types for stored SSH connections (direct shell sessions, not tunnels).

use serde::{Deserialize, Serialize};

/// User-editable configuration for a direct SSH connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConnectionConfig {
    /// Human-readable label shown in the UI.
    pub name: String,
    /// Remote hostname or IP address.
    pub host: String,
    /// SSH port (default 22).
    #[serde(default = "default_port")]
    pub port: u16,
    /// SSH username.
    pub username: String,
    /// Path to private key file (optional).
    pub identity_file: Option<String>,
    /// Jump/bastion host in `user@host[:port]` format (optional).
    pub jump_host: Option<String>,
    /// Raw extra SSH arguments appended to the command (optional).
    pub extra_args: Option<String>,
    /// Optional free-text description.
    pub description: Option<String>,
    /// Optional tags for grouping / filtering.
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_port() -> u16 {
    22
}

/// A stored SSH connection — config + runtime metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConnection {
    /// Unique stable identifier (UUID v4).
    pub id: String,
    pub config: SshConnectionConfig,
    /// Unix-ms timestamp of the last call to `launch_connection`, if any.
    pub last_connected_at: Option<u64>,
    /// Whether a password is stored in the OS keychain for this connection.
    /// Computed at query time; never written to disk.
    #[serde(skip_serializing)]
    #[serde(default)]
    pub has_password: bool,
}
