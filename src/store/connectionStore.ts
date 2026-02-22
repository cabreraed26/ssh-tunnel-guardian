import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { SshConnection, SshConnectionConfig } from "../types";
import { api } from "../lib/tauriApi";

// ─── State Shape ──────────────────────────────────────────────────────────────

interface ConnectionStore {
  connections: SshConnection[];
  loadingIds: string[];
  error: string | null;
  initialized: boolean;

  // ─── Actions ───────────────────────────────────────────────────────────────
  init: () => Promise<void>;

  // CRUD
  addConnection:    (config: SshConnectionConfig) => Promise<SshConnection>;
  removeConnection: (id: string) => Promise<void>;
  updateConnection: (id: string, config: SshConnectionConfig) => Promise<SshConnection>;
  // Keychain
  savePassword:   (id: string, password: string) => Promise<void>;
  deletePassword: (id: string) => Promise<void>;
  // Launch
  launchConnection: (id: string) => Promise<void>;

  // UI helpers
  setError: (msg: string | null) => void;
}

// ─── Store ────────────────────────────────────────────────────────────────────

export const useConnectionStore = create<ConnectionStore>()(
  immer((set, get) => ({
    connections: [],
    loadingIds: [],
    error: null,
    initialized: false,

    init: async () => {
      if (get().initialized) return;
      try {
        const connections = await api.getConnections();
        set((s) => {
          s.connections = connections;
          s.initialized = true;
        });
      } catch (err) {
        set((s) => {
          s.error = String(err);
        });
      }
    },

    addConnection: async (config) => {
      const conn = await api.addConnection(config);
      set((s) => {
        s.connections.push(conn);
        s.connections.sort((a, b) => a.config.name.localeCompare(b.config.name));
      });
      return conn;
    },

    removeConnection: async (id) => {
      await api.removeConnection(id);
      set((s) => {
        s.connections = s.connections.filter((c) => c.id !== id);
      });
    },

    updateConnection: async (id, config) => {
      const updated = await api.updateConnection(id, config);
      set((s) => {
        const idx = s.connections.findIndex((c) => c.id === id);
        if (idx !== -1) s.connections[idx] = updated;
        s.connections.sort((a, b) => a.config.name.localeCompare(b.config.name));
      });
      return updated;
    },

    launchConnection: async (id) => {
      set((s) => { s.loadingIds.push(id); });
      try {
        await api.launchConnection(id);
        // Update last_connected_at locally (the backend persists it).
        set((s) => {
          const conn = s.connections.find((c) => c.id === id);
          if (conn) conn.lastConnectedAt = Date.now();
        });
      } finally {
        set((s) => { s.loadingIds = s.loadingIds.filter((i) => i !== id); });
      }
    },

    savePassword: async (id, password) => {
      await api.saveConnectionPassword(id, password);
      set((s) => {
        const conn = s.connections.find((c) => c.id === id);
        if (conn) conn.hasPassword = password.length > 0;
      });
    },

    deletePassword: async (id) => {
      await api.deleteConnectionPassword(id);
      set((s) => {
        const conn = s.connections.find((c) => c.id === id);
        if (conn) conn.hasPassword = false;
      });
    },

    setError: (msg) => set((s) => { s.error = msg; }),
  }))
);
