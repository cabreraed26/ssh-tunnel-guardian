import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type {
  LogEntry,
  MetricsPayload,
  StateChangedPayload,
  TunnelConfig,
  TunnelInfo,
} from "../types";
import { api } from "../lib/tauriApi";

// ─── State Shape ──────────────────────────────────────────────────────────────

interface TunnelStore {
  // Data
  tunnels: TunnelInfo[];
  logs: Record<string, LogEntry[]>;
  loadingIds: string[];
  error: string | null;

  // Loaded flag
  initialized: boolean;

  // ─── Actions ───────────────────────────────────────────────────────────────
  init: () => Promise<void>;

  // CRUD
  addTunnel: (config: TunnelConfig) => Promise<TunnelInfo>;
  removeTunnel: (id: string) => Promise<void>;
  updateTunnel: (id: string, config: TunnelConfig) => Promise<TunnelInfo>;

  // Lifecycle
  startTunnel: (id: string) => Promise<void>;
  stopTunnel: (id: string) => Promise<void>;
  restartTunnel: (id: string) => Promise<void>;

  // Logs
  fetchLogs: (id: string, limit?: number) => Promise<void>;
  clearLogs: (id: string) => void;

  // Event handlers (called by the event subscription hook)
  applyStateChange: (payload: StateChangedPayload) => void;
  applyMetrics: (payload: MetricsPayload) => void;
  appendLog: (entry: LogEntry) => void;

  // UI helpers
  setError: (msg: string | null) => void;
}

// ─── Max in-memory logs per tunnel ────────────────────────────────────────────
const MAX_LOGS = 300;

// ─── Store ─────────────────────────────────────────────────────────────────────

export const useTunnelStore = create<TunnelStore>()(
  immer((set, get) => ({
    tunnels: [],
    logs: {},
    loadingIds: [],
    error: null,
    initialized: false,

    // ── Initialisation ───────────────────────────────────────────────────────
    init: async () => {
      if (get().initialized) return;
      try {
        const tunnels = await api.getTunnels();
        set((s) => {
          s.tunnels = tunnels;
          s.initialized = true;
        });
      } catch (err) {
        set((s) => {
          s.error = String(err);
        });
      }
    },

    // ── CRUD ─────────────────────────────────────────────────────────────────
    addTunnel: async (config) => {
      const info = await api.addTunnel(config);
      set((s) => {
        s.tunnels.push(info);
        s.logs[info.id] = [];
      });
      return info;
    },

    removeTunnel: async (id) => {
      await api.removeTunnel(id);
      set((s) => {
        s.tunnels = s.tunnels.filter((t: TunnelInfo) => t.id !== id);
        delete s.logs[id];
      });
    },

    updateTunnel: async (id, config) => {
      const updated = await api.updateTunnel(id, config);
      set((s) => {
        const idx = s.tunnels.findIndex((t: TunnelInfo) => t.id === id);
        if (idx !== -1) s.tunnels[idx] = updated;
      });
      return updated;
    },

    // ── Lifecycle ────────────────────────────────────────────────────────────
    startTunnel: async (id) => {
      set((s) => {
        if (!s.loadingIds.includes(id)) s.loadingIds.push(id);
      });
      try {
        await api.startTunnel(id);
      } finally {
        set((s) => {
          s.loadingIds = s.loadingIds.filter((i) => i !== id);
        });
      }
    },

    stopTunnel: async (id) => {
      set((s) => {
        if (!s.loadingIds.includes(id)) s.loadingIds.push(id);
      });
      try {
        await api.stopTunnel(id);
      } finally {
        set((s) => {
          s.loadingIds = s.loadingIds.filter((i) => i !== id);
        });
      }
    },

    restartTunnel: async (id) => {
      set((s) => {
        if (!s.loadingIds.includes(id)) s.loadingIds.push(id);
      });
      try {
        await api.restartTunnel(id);
      } finally {
        set((s) => {
          s.loadingIds = s.loadingIds.filter((i) => i !== id);
        });
      }
    },

    // ── Logs ─────────────────────────────────────────────────────────────────
    fetchLogs: async (id, limit = 100) => {
      const entries = await api.getTunnelLogs(id, limit);
      set((s) => {
        s.logs[id] = entries;
      });
    },

    clearLogs: (id) => {
      set((s) => {
        s.logs[id] = [];
      });
    },

    // ── Event handlers ───────────────────────────────────────────────────────
    applyStateChange: (payload) => {
      set((s) => {
        const tunnel = s.tunnels.find((t: TunnelInfo) => t.id === payload.tunnelId);
        if (tunnel) {
          tunnel.state = payload.state;
          tunnel.stateEnteredAt = payload.timestamp;
        }
        // Also push as a log entry.
        const logs = s.logs[payload.tunnelId] ?? [];
        const entry: LogEntry = {
          tunnelId: payload.tunnelId,
          level: "INFO",
          message: payload.message
            ? `[${payload.state}] ${payload.message}`
            : `State → ${payload.state}`,
          timestamp: payload.timestamp,
        };
        logs.push(entry);
        if (logs.length > MAX_LOGS) logs.splice(0, logs.length - MAX_LOGS);
        s.logs[payload.tunnelId] = logs;
      });
    },

    applyMetrics: (payload) => {
      set((s) => {
        const tunnel = s.tunnels.find((t: TunnelInfo) => t.id === payload.tunnelId);
        if (tunnel) {
          tunnel.uptimeMs = payload.uptimeMs;
          tunnel.reconnectAttempts = payload.reconnectAttempts;
          tunnel.lastHealthCheckAt = payload.lastHealthCheckAt;
          tunnel.pid = payload.pid;
        }
      });
    },

    appendLog: (entry) => {
      set((s) => {
        const logs = s.logs[entry.tunnelId] ?? [];
        logs.push(entry);
        if (logs.length > MAX_LOGS) logs.splice(0, logs.length - MAX_LOGS);
        s.logs[entry.tunnelId] = logs;
      });
    },

    // ── UI ───────────────────────────────────────────────────────────────────
    setError: (msg) => {
      set((s) => {
        s.error = msg;
      });
    },
  }))
);
