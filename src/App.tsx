import { useEffect, useState } from "react";
import { X, Server } from "lucide-react";
import { useTunnelStore } from "./store/tunnelStore";
import { useConnectionStore } from "./store/connectionStore";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { Header } from "./components/Header";
import { TunnelCard } from "./components/TunnelCard";
import { AddTunnelModal } from "./components/AddTunnelModal";
import { ConnectionCard } from "./components/ConnectionCard";
import { AddConnectionModal } from "./components/AddConnectionModal";
import { LogsPanel } from "./components/LogsPanel";
import { EmptyState } from "./components/EmptyState";
import type { SshConnection, TunnelInfo } from "./types";
import "./App.css";

type Tab = "tunnels" | "connections";

export default function App() {
  useTauriEvents();

  const initTunnels = useTunnelStore((s) => s.init);
  const tunnels = useTunnelStore((s) => s.tunnels);
  const tunnelError = useTunnelStore((s) => s.error);
  const setTunnelError = useTunnelStore((s) => s.setError);

  const initConnections = useConnectionStore((s) => s.init);
  const connections = useConnectionStore((s) => s.connections);
  const connError = useConnectionStore((s) => s.error);
  const setConnError = useConnectionStore((s) => s.setError);

  const [tab, setTab] = useState<Tab>("tunnels");

  // Tunnel modal state
  const [tunnelModalOpen, setTunnelModalOpen] = useState(false);
  const [editTunnel, setEditTunnel] = useState<TunnelInfo | null>(null);
  const [logsTarget, setLogsTarget] = useState<TunnelInfo | null>(null);

  // Connection modal state
  const [connModalOpen, setConnModalOpen] = useState(false);
  const [editConn, setEditConn] = useState<SshConnection | null>(null);

  useEffect(() => {
    initTunnels();
    initConnections();
  }, [initTunnels, initConnections]);

  // ── Tunnel handlers ──────────────────────────────────────────────────────
  function openAddTunnel() { setEditTunnel(null); setTunnelModalOpen(true); }
  function openEditTunnel(t: TunnelInfo) { setEditTunnel(t); setTunnelModalOpen(true); }
  function closeTunnelModal() { setTunnelModalOpen(false); setEditTunnel(null); }

  // ── Connection handlers ──────────────────────────────────────────────────
  function openAddConn() { setEditConn(null); setConnModalOpen(true); }
  function openEditConn(c: SshConnection) { setEditConn(c); setConnModalOpen(true); }
  function closeConnModal() { setConnModalOpen(false); setEditConn(null); }

  const error = tunnelError ?? connError;
  const clearError = () => { setTunnelError(null); setConnError(null); };

  return (
    <div className="app">
      <Header
        tab={tab}
        onTabChange={setTab}
        onAddTunnel={openAddTunnel}
        onAddConnection={openAddConn}
      />

      {/* ── Global error toast ───────────────────────────────────────────── */}
      {error && (
        <div className="toast toast--error">
          <span>{error}</span>
          <button
            className="btn btn--ghost btn--icon btn--sm"
            onClick={clearError}
          >
            <X size={14} />
          </button>
        </div>
      )}

      <main className="app__main">
        {/* ── Tunnels tab ─────────────────────────────────────────────── */}
        {tab === "tunnels" && (
          <>
            {tunnels.length === 0 ? (
              <EmptyState onAdd={openAddTunnel} />
            ) : (
              <div className="tunnel-grid">
                {tunnels.map((tunnel) => (
                  <TunnelCard
                    key={tunnel.id}
                    tunnel={tunnel}
                    onEdit={openEditTunnel}
                    onShowLogs={setLogsTarget}
                  />
                ))}
              </div>
            )}
          </>
        )}

        {/* ── Connections tab ─────────────────────────────────────────── */}
        {tab === "connections" && (
          <>
            {connections.length === 0 ? (
              <div className="empty-state">
                <Server size={36} className="empty-state__icon" />
                <h2 className="empty-state__title">No connections yet</h2>
                <p className="empty-state__body">
                  Store your SSH hosts here and connect with one click — they
                  open directly in Terminal.app (or iTerm2 if installed).
                </p>
                <button className="btn btn--primary" onClick={openAddConn}>
                  Add Connection
                </button>
              </div>
            ) : (
              <div className="conn-grid">
                {connections.map((c) => (
                  <ConnectionCard key={c.id} conn={c} onEdit={openEditConn} />
                ))}
              </div>
            )}
          </>
        )}
      </main>

      {/* ── Logs panel ──────────────────────────────────────────────────── */}
      {logsTarget && (
        <LogsPanel
          tunnelId={logsTarget.id}
          tunnelName={logsTarget.config.name}
          onClose={() => setLogsTarget(null)}
        />
      )}

      {/* ── Tunnel modal ────────────────────────────────────────────────── */}
      {tunnelModalOpen && (
        <AddTunnelModal editTarget={editTunnel} onClose={closeTunnelModal} />
      )}

      {/* ── Connection modal ────────────────────────────────────────────── */}
      {connModalOpen && (
        <AddConnectionModal editTarget={editConn} onClose={closeConnModal} />
      )}
    </div>
  );
}

