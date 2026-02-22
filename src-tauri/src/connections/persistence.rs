//! Persists the list of SSH connections to `connections.json` in the app data dir.
//! Only configs + IDs are saved; `last_connected_at` is also persisted.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::connections::types::SshConnection;

// ─── On-disk schema ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct Store {
    #[serde(default = "store_version")]
    version: u32,
    connections: Vec<SshConnection>,
}

fn store_version() -> u32 {
    1
}

// ─── Public API ──────────────────────────────────────────────────────────────

pub fn data_file(data_dir: &Path) -> PathBuf {
    data_dir.join("connections.json")
}

/// Saves the complete list of connections atomically (tmp + rename).
pub fn save(data_dir: &Path, connections: &[SshConnection]) {
    let store = Store {
        version: 1,
        connections: connections.to_vec(),
    };

    let Ok(json) = serde_json::to_string_pretty(&store) else {
        eprintln!("[STG connections] serialization failed");
        return;
    };

    if let Err(e) = std::fs::create_dir_all(data_dir) {
        eprintln!("[STG connections] could not create data dir: {e}");
        return;
    }

    let tmp = data_file(data_dir).with_extension("json.tmp");
    if let Err(e) = std::fs::write(&tmp, &json) {
        eprintln!("[STG connections] write to tmp failed: {e}");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, data_file(data_dir)) {
        eprintln!("[STG connections] rename failed: {e}");
    }
}

/// Loads all connections from disk. Returns an empty vec on any error.
pub fn load(data_dir: &Path) -> Vec<SshConnection> {
    let path = data_file(data_dir);
    let Ok(bytes) = std::fs::read(&path) else {
        return Vec::new();
    };
    match serde_json::from_slice::<Store>(&bytes) {
        Ok(store) => store.connections,
        Err(e) => {
            eprintln!("[STG connections] failed to parse {}: {e}", path.display());
            Vec::new()
        }
    }
}
