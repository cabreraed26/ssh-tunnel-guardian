/**
 * Type-safe wrapper around all Tauri `invoke` calls and event listeners.
 * All commands mirror the Tauri command handlers defined in src-tauri/src/commands/.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  LogEntry,
  MetricsPayload,
  StateChangedPayload,
  SshConnection,
  SshConnectionConfig,
  TunnelConfig,
  TunnelInfo,
} from "../types";

// ─── Commands ─────────────────────────────────────────────────────────────────

export const api = {
  // Read
  getTunnels: (): Promise<TunnelInfo[]> => invoke("get_tunnels"),

  getTunnel: (id: string): Promise<TunnelInfo> =>
    invoke("get_tunnel", { id }),

  getTunnelLogs: (id: string, limit = 100): Promise<LogEntry[]> =>
    invoke("get_tunnel_logs", { id, limit }),

  // Write
  addTunnel: (config: TunnelConfig): Promise<TunnelInfo> =>
    invoke("add_tunnel", { config }),

  removeTunnel: (id: string): Promise<void> =>
    invoke("remove_tunnel", { id }),

  updateTunnel: (id: string, config: TunnelConfig): Promise<TunnelInfo> =>
    invoke("update_tunnel", { id, config }),

  // Lifecycle
  startTunnel: (id: string): Promise<void> => invoke("start_tunnel", { id }),
  stopTunnel: (id: string): Promise<void> => invoke("stop_tunnel", { id }),
  restartTunnel: (id: string): Promise<void> =>
    invoke("restart_tunnel", { id }),

  // ── Connections ─────────────────────────────────────────────────────────────
  getConnections: (): Promise<SshConnection[]> => invoke("get_connections"),

  getConnection: (id: string): Promise<SshConnection> =>
    invoke("get_connection", { id }),

  addConnection: (config: SshConnectionConfig): Promise<SshConnection> =>
    invoke("add_connection", { config }),

  removeConnection: (id: string): Promise<void> =>
    invoke("remove_connection", { id }),

  updateConnection: (id: string, config: SshConnectionConfig): Promise<SshConnection> =>
    invoke("update_connection", { id, config }),

  launchConnection: (id: string): Promise<void> =>
    invoke("launch_connection", { id }),

  saveConnectionPassword: (id: string, password: string): Promise<void> =>
    invoke("save_connection_password", { id, password }),

  deleteConnectionPassword: (id: string): Promise<void> =>
    invoke("delete_connection_password", { id }),
} as const;

// ─── Events ───────────────────────────────────────────────────────────────────

export const events = {
  onStateChanged: (
    handler: (payload: StateChangedPayload) => void
  ): Promise<UnlistenFn> =>
    listen<StateChangedPayload>("stg://tunnel-state-changed", (e) =>
      handler(e.payload)
    ),

  onMetrics: (
    handler: (payload: MetricsPayload) => void
  ): Promise<UnlistenFn> =>
    listen<MetricsPayload>("stg://tunnel-metrics", (e) => handler(e.payload)),

  onLog: (handler: (entry: LogEntry) => void): Promise<UnlistenFn> =>
    listen<LogEntry>("stg://tunnel-log", (e) => handler(e.payload)),
} as const;
