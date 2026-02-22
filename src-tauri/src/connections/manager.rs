//! Manager for stored SSH connections — CRUD + terminal launch.

use std::path::PathBuf;

use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::connections::{keychain, persistence, types::{SshConnection, SshConnectionConfig}};

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn new_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Simple unique ID: timestamp-nanos + thread-id-derived counter.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let ts = now_ms();
    format!("conn-{ts}-{nanos:08x}")
}

// ─── Manager ─────────────────────────────────────────────────────────────────

pub struct ConnectionsManager {
    connections: Mutex<HashMap<String, SshConnection>>,
    data_dir: PathBuf,
}

impl ConnectionsManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let saved = persistence::load(&data_dir);
        let mut map = HashMap::new();
        for conn in saved {
            map.insert(conn.id.clone(), conn);
        }
        Self {
            connections: Mutex::new(map),
            data_dir,
        }
    }

    // ── CRUD ─────────────────────────────────────────────────────────────────

    pub async fn list(&self) -> Vec<SshConnection> {
        let conns = self.connections.lock().await;
        let mut list: Vec<SshConnection> = conns.values().cloned().collect();
        for c in list.iter_mut() {
            c.has_password = keychain::exists(&c.id);
        }
        list.sort_by(|a, b| a.config.name.cmp(&b.config.name));
        list
    }

    pub async fn add(&self, config: SshConnectionConfig) -> SshConnection {
        let conn = SshConnection {
            id: new_id(),
            config,
            last_connected_at: None,
            has_password: false,
        };
        let mut conns = self.connections.lock().await;
        conns.insert(conn.id.clone(), conn.clone());
        let all: Vec<SshConnection> = conns.values().cloned().collect();
        persistence::save(&self.data_dir, &all);
        conn
    }

    pub async fn remove(&self, id: &str) -> Result<(), String> {
        let mut conns = self.connections.lock().await;
        if conns.remove(id).is_none() {
            return Err(format!("Connection '{id}' not found"));
        }
        // Remove any stored password from the keychain.
        keychain::delete(id);
        let all: Vec<SshConnection> = conns.values().cloned().collect();
        persistence::save(&self.data_dir, &all);
        Ok(())
    }

    pub async fn update(&self, id: &str, config: SshConnectionConfig) -> Result<SshConnection, String> {
        let mut conns = self.connections.lock().await;
        let conn = conns.get_mut(id).ok_or_else(|| format!("Connection '{id}' not found"))?;
        conn.config = config;
        let mut updated = conn.clone();
        let all: Vec<SshConnection> = conns.values().cloned().collect();
        persistence::save(&self.data_dir, &all);
        updated.has_password = keychain::exists(id);
        Ok(updated)
    }

    pub async fn get(&self, id: &str) -> Option<SshConnection> {
        let mut conn = self.connections.lock().await.get(id).cloned()?;
        conn.has_password = keychain::exists(id);
        Some(conn)
    }

    // ── Password management ──────────────────────────────────────────────────

    /// Saves a password for this connection in the OS keychain.
    pub fn save_password(&self, id: &str, password: &str) -> Result<(), String> {
        keychain::save(id, password)
    }

    /// Removes the stored password from the OS keychain.
    pub fn delete_password(&self, id: &str) {
        keychain::delete(id);
    }

    // ── Launch ───────────────────────────────────────────────────────────────

    /// Builds an SSH command string and opens it in the system terminal.
    pub async fn launch(&self, id: &str) -> Result<(), String> {
        let config = {
            let mut conns = self.connections.lock().await;
            let conn = conns.get_mut(id).ok_or_else(|| format!("Connection '{id}' not found"))?;
            conn.last_connected_at = Some(now_ms());
            let cfg = conn.config.clone();
            let all: Vec<SshConnection> = conns.values().cloned().collect();
            persistence::save(&self.data_dir, &all);
            cfg
        };

        // Retrieve password from keychain (None = key-based auth).
        let password = keychain::get(id);
        let ssh_cmd = build_ssh_command(&config);
        open_in_terminal(&ssh_cmd, password.as_deref())
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn build_ssh_command(config: &SshConnectionConfig) -> String {
    let mut parts: Vec<String> = vec!["ssh".to_string()];

    // Port (only if non-default)
    if config.port != 22 {
        parts.push("-p".to_string());
        parts.push(config.port.to_string());
    }

    // Identity file
    if let Some(ref key) = config.identity_file {
        if !key.trim().is_empty() {
            parts.push("-i".to_string());
            // Quote the path in case it contains spaces.
            parts.push(format!("\"{}\"", key.trim()));
        }
    }

    // Jump host
    if let Some(ref jump) = config.jump_host {
        if !jump.trim().is_empty() {
            parts.push("-J".to_string());
            parts.push(jump.trim().to_string());
        }
    }

    // Extra args (raw, user-supplied)
    if let Some(ref extra) = config.extra_args {
        let trimmed = extra.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }

    // user@host — always last positional arg
    parts.push(format!("{}@{}", config.username, config.host));

    parts.join(" ")
}

#[cfg(target_os = "macos")]
fn open_in_terminal(ssh_cmd: &str, password: Option<&str>) -> Result<(), String> {
    // If a password is saved, prefer sshpass (transparent) or fall back to
    // copying the password to the clipboard so the user can paste it.
    let final_cmd: String = if let Some(pwd) = password {
        let sshpass_available = std::process::Command::new("which")
            .arg("sshpass")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if sshpass_available {
            // Wrap the command: sshpass -p 'PASSWORD' ssh ...
            // Single-quote the password and escape any inner single-quotes.
            let safe_pwd = pwd.replace('\'', "'\\''" );
            format!("sshpass -p '{safe_pwd}' {ssh_cmd}")
        } else {
            // Copy to clipboard silently, then open terminal normally.
            let _ = std::process::Command::new("bash")
                .args(["-c", &format!("printf '%s' {} | pbcopy", shlex_quote(pwd))])
                .status();
            // Prepend a visible reminder inside the terminal window.
            format!(
                r#"bash -c 'echo "ℹ️ Password copied to clipboard — paste with ⌘V when prompted"; {ssh_cmd}; exec $SHELL'"#
            )
        }
    } else {
        ssh_cmd.to_string()
    };

    // Escape double-quotes inside the command so the AppleScript string is valid.
    let escaped = final_cmd.replace('\\', "\\\\").replace('"', "\\\"");

    // Check if iTerm2 is installed and prefer it, otherwise fall back to Terminal.
    let iterm_check = std::process::Command::new("osascript")
        .args(["-e", "id of application \"iTerm\""])
        .output();

    let script = if iterm_check.map(|o| o.status.success()).unwrap_or(false) {
        format!(
            r#"tell application "iTerm"
    activate
    tell current window
        create tab with default profile
        tell current session
            write text "{escaped}"
        end tell
    end tell
end tell"#
        )
    } else {
        format!(
            r#"tell application "Terminal"
    activate
    do script "{escaped}"
end tell"#
        )
    };

    std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .map_err(|e| format!("Failed to launch terminal: {e}"))?;

    Ok(())
}

/// Minimal shell-quoting: wraps the string in single-quotes and escapes embedded single-quotes.
fn shlex_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''" ))
}

#[cfg(target_os = "linux")]
fn open_in_terminal(ssh_cmd: &str, password: Option<&str>) -> Result<(), String> {
    let final_cmd = if let Some(pwd) = password {
        let sshpass_available = std::process::Command::new("which")
            .arg("sshpass")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if sshpass_available {
            let safe_pwd = pwd.replace('\'', "'\\''" );
            format!("sshpass -p '{safe_pwd}' {ssh_cmd}")
        } else {
            ssh_cmd.to_string()
        }
    } else {
        ssh_cmd.to_string()
    };

    // Try common terminal emulators in order of preference.
    let terminals = ["gnome-terminal", "konsole", "xfce4-terminal", "xterm"];
    for term in &terminals {
        let spawn_result = if *term == "gnome-terminal" {
            std::process::Command::new(term)
                .args(["--", "bash", "-c", &format!("{final_cmd}; exec bash")])
                .spawn()
        } else {
            std::process::Command::new(term)
                .args(["-e", &format!("bash -c '{final_cmd}; exec bash'")])
                .spawn()
        };
        if spawn_result.is_ok() {
            return Ok(());
        }
    }
    Err("No supported terminal emulator found (tried gnome-terminal, konsole, xfce4-terminal, xterm)".to_string())
}

#[cfg(target_os = "windows")]
fn open_in_terminal(ssh_cmd: &str, password: Option<&str>) -> Result<(), String> {
    let final_cmd = if let Some(pwd) = password {
        let sshpass_available = std::process::Command::new("where")
            .arg("sshpass")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if sshpass_available {
            format!("sshpass -p '{}' {ssh_cmd}", pwd.replace('\'', "'\\''" ))
        } else {
            ssh_cmd.to_string()
        }
    } else {
        ssh_cmd.to_string()
    };

    // Try Windows Terminal first, fall back to cmd.
    let wt = std::process::Command::new("wt")
        .args(["new-tab", "--", "powershell", "-NoExit", "-Command", &final_cmd])
        .spawn();
    if wt.is_err() {
        std::process::Command::new("cmd")
            .args(["/C", "start", "powershell", "-NoExit", "-Command", &final_cmd])
            .spawn()
            .map_err(|e| format!("Failed to launch terminal: {e}"))?;
    }
    Ok(())
}
