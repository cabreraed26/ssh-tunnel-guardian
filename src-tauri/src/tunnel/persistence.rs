//! Persists the list of configured tunnels to a JSON file in the app data directory.
//!
//! Only the tunnel **configs** (and their IDs) are saved.  Runtime state (process,
//! health counters, logs, …) is always rebuilt fresh when the app restarts.
//!
//! File location: `<AppDataDir>/tunnels.json`

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::tunnel::types::TunnelConfig;

// ─── On-disk schema ──────────────────────────────────────────────────────────

/// Versioned wrapper so we can migrate the schema in future releases.
#[derive(Serialize, Deserialize, Default)]
struct Store {
    /// Schema version — currently 1.
    #[serde(default = "store_version")]
    version: u32,
    tunnels: Vec<SavedTunnel>,
}

fn store_version() -> u32 {
    1
}

#[derive(Serialize, Deserialize)]
struct SavedTunnel {
    id: String,
    config: TunnelConfig,
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Returns the path of the persistence file for the given app data directory.
pub fn data_file(data_dir: &Path) -> PathBuf {
    data_dir.join("tunnels.json")
}

/// Saves the complete list of tunnels atomically (write-to-temp + rename).
///
/// On failure the existing file is left untouched and the error is logged to
/// stderr (non-fatal — the app continues running).
pub fn save(data_dir: &Path, entries: &[(String, TunnelConfig)]) {
    let store = Store {
        version: 1,
        tunnels: entries
            .iter()
            .map(|(id, config)| SavedTunnel {
                id: id.clone(),
                config: config.clone(),
            })
            .collect(),
    };

    let Ok(json) = serde_json::to_string_pretty(&store) else {
        eprintln!("[STG persist] serialization failed");
        return;
    };

    if let Err(e) = std::fs::create_dir_all(data_dir) {
        eprintln!("[STG persist] could not create data dir: {e}");
        return;
    }

    // Write to a temp file and rename for atomicity.
    let tmp = data_file(data_dir).with_extension("json.tmp");
    if let Err(e) = std::fs::write(&tmp, &json) {
        eprintln!("[STG persist] write failed: {e}");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, data_file(data_dir)) {
        eprintln!("[STG persist] rename failed: {e}");
        // Clean up orphan tmp file — ignore error.
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Loads the persisted tunnel list.  Returns an empty Vec on any error
/// (missing file, corrupt JSON, etc.) so the app always starts cleanly.
pub fn load(data_dir: &Path) -> Vec<(String, TunnelConfig)> {
    let path = data_file(data_dir);
    let Ok(json) = std::fs::read_to_string(&path) else {
        return vec![];
    };
    match serde_json::from_str::<Store>(&json) {
        Ok(store) => store
            .tunnels
            .into_iter()
            .map(|t| (t.id, t.config))
            .collect(),
        Err(e) => {
            eprintln!("[STG persist] could not parse {}: {e}", path.display());
            vec![]
        }
    }
}
