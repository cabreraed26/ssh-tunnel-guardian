import { useEffect, useState } from "react";
import { X } from "lucide-react";
import { useTunnelStore } from "./store/tunnelStore";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { Header } from "./components/Header";
import { TunnelCard } from "./components/TunnelCard";
import { AddTunnelModal } from "./components/AddTunnelModal";
import { LogsPanel } from "./components/LogsPanel";
import { EmptyState } from "./components/EmptyState";
import type { TunnelInfo } from "./types";
import "./App.css";

export default function App() {
  // Bootstrap event subscriptions and initial data load.
  useTauriEvents();

  const init = useTunnelStore((s) => s.init);
  const tunnels = useTunnelStore((s) => s.tunnels);
  const error = useTunnelStore((s) => s.error);
  const setError = useTunnelStore((s) => s.setError);

  const [modalOpen, setModalOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<TunnelInfo | null>(null);
  const [logsTarget, setLogsTarget] = useState<TunnelInfo | null>(null);

  useEffect(() => {
    init();
  }, [init]);

  function openAddModal() {
    setEditTarget(null);
    setModalOpen(true);
  }

  function openEditModal(tunnel: TunnelInfo) {
    setEditTarget(tunnel);
    setModalOpen(true);
  }

  function closeModal() {
    setModalOpen(false);
    setEditTarget(null);
  }

  return (
    <div className="app">
      <Header onAdd={openAddModal} />

      {/* ── Global error toast ─────────────────────────────────────────── */}
      {error && (
        <div className="toast toast--error">
          <span>{error}</span>
          <button
            className="btn btn--ghost btn--icon btn--sm"
            onClick={() => setError(null)}
          >
            <X size={14} />
          </button>
        </div>
      )}

      <main className="app__main">
        {tunnels.length === 0 ? (
          <EmptyState onAdd={openAddModal} />
        ) : (
          <div className="tunnel-grid">
            {tunnels.map((tunnel) => (
              <TunnelCard
                key={tunnel.id}
                tunnel={tunnel}
                onEdit={openEditModal}
                onShowLogs={setLogsTarget}
              />
            ))}
          </div>
        )}
      </main>

      {/* ── Logs Panel ─────────────────────────────────────────────────── */}
      {logsTarget && (
        <LogsPanel
          tunnelId={logsTarget.id}
          tunnelName={logsTarget.config.name}
          onClose={() => setLogsTarget(null)}
        />
      )}

      {/* ── Add / Edit Modal ───────────────────────────────────────────── */}
      {modalOpen && (
        <AddTunnelModal editTarget={editTarget} onClose={closeModal} />
      )}
    </div>
  );
}
