import { useState } from "react";
import {
  Play,
  Square,
  RefreshCw,
  Trash2,
  Pencil,
  Terminal,
  ChevronDown,
  ChevronUp,
  AlertTriangle,
} from "lucide-react";
import clsx from "clsx";
import type { TunnelInfo } from "../types";
import { STATE_META } from "../types";
import { StatusBadge } from "./StatusBadge";
import { useTunnelStore } from "../store/tunnelStore";

interface TunnelCardProps {
  tunnel: TunnelInfo;
  onEdit: (tunnel: TunnelInfo) => void;
  onShowLogs: (tunnel: TunnelInfo) => void;
}

function formatUptime(ms: number): string {
  if (ms === 0) return "—";
  const s = Math.floor(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ${s % 60}s`;
  const h = Math.floor(m / 60);
  return `${h}h ${m % 60}m`;
}

export function TunnelCard({ tunnel, onEdit, onShowLogs }: TunnelCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const startTunnel = useTunnelStore((s) => s.startTunnel);
  const stopTunnel = useTunnelStore((s) => s.stopTunnel);
  const restartTunnel = useTunnelStore((s) => s.restartTunnel);
  const removeTunnel = useTunnelStore((s) => s.removeTunnel);
  const loadingIds = useTunnelStore((s) => s.loadingIds);
  const setError = useTunnelStore((s) => s.setError);

  const isLoading = loadingIds.includes(tunnel.id);
  const isStopped = tunnel.state === "STOPPED" || tunnel.state === "FAILED";
  const meta = STATE_META[tunnel.state];

  const handleAction = async (action: () => Promise<void>) => {
    try {
      await action();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div
      className={clsx("tunnel-card", `tunnel-card--${tunnel.state.toLowerCase()}`)}
      style={{ "--state-color": meta.color } as React.CSSProperties}
    >
      {/* ── Card Header ──────────────────────────────────────────────────── */}
      <div className="tunnel-card__header">
        <div className="tunnel-card__identity">
          <span className="tunnel-card__name">{tunnel.config.name}</span>
          <StatusBadge state={tunnel.state} />
        </div>
        <div className="tunnel-card__controls">
          {isStopped ? (
            <button
              className="btn btn--success btn--sm"
              onClick={() => handleAction(() => startTunnel(tunnel.id))}
              disabled={isLoading}
              title="Start"
            >
              <Play size={14} />
            </button>
          ) : (
            <button
              className="btn btn--danger btn--sm"
              onClick={() => handleAction(() => stopTunnel(tunnel.id))}
              disabled={isLoading || tunnel.state === "STARTING"}
              title="Stop"
            >
              <Square size={14} />
            </button>
          )}
          <button
            className="btn btn--ghost btn--sm"
            onClick={() => handleAction(() => restartTunnel(tunnel.id))}
            disabled={isLoading || tunnel.state === "STOPPED"}
            title="Restart"
          >
            <RefreshCw size={14} className={isLoading ? "spin" : ""} />
          </button>
          <button
            className="btn btn--ghost btn--sm"
            onClick={() => onShowLogs(tunnel)}
            title="View logs"
          >
            <Terminal size={14} />
          </button>
          <button
            className="btn btn--ghost btn--sm"
            onClick={() => onEdit(tunnel)}
            title="Edit"
          >
            <Pencil size={14} />
          </button>
          {confirmDelete ? (
            <>
              <button
                className="btn btn--danger btn--sm"
                onClick={() => handleAction(() => removeTunnel(tunnel.id))}
              >
                Confirm
              </button>
              <button
                className="btn btn--ghost btn--sm"
                onClick={() => setConfirmDelete(false)}
              >
                Cancel
              </button>
            </>
          ) : (
            <button
              className="btn btn--ghost btn--sm btn--ghost-danger"
              onClick={() => setConfirmDelete(true)}
              title="Delete"
            >
              <Trash2 size={14} />
            </button>
          )}
          <button
            className="btn btn--ghost btn--sm"
            onClick={() => setExpanded((v) => !v)}
            title={expanded ? "Collapse" : "Expand"}
          >
            {expanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
          </button>
        </div>
      </div>

      {/* ── Route summary ────────────────────────────────────────────────── */}
      <div className="tunnel-card__route">
        <span className="tunnel-card__port">:{tunnel.config.localPort}</span>
        <span className="tunnel-card__arrow">→</span>
        <span className="tunnel-card__remote">
          {tunnel.config.remoteHost}:{tunnel.config.remotePort}
        </span>
        <span className="tunnel-card__via">via</span>
        <span className="tunnel-card__ssh">
          {tunnel.config.sshUser}@{tunnel.config.sshHost}:{tunnel.config.sshPort}
        </span>
      </div>

      {/* ── Metrics strip ────────────────────────────────────────────────── */}
      <div className="tunnel-card__metrics">
        <div className="metric">
          <span className="metric__label">Uptime</span>
          <span className="metric__value">{formatUptime(tunnel.uptimeMs)}</span>
        </div>
        <div className="metric">
          <span className="metric__label">Reconnects</span>
          <span className="metric__value">{tunnel.reconnectAttempts}</span>
        </div>
        {tunnel.pid && (
          <div className="metric">
            <span className="metric__label">PID</span>
            <span className="metric__value">{tunnel.pid}</span>
          </div>
        )}
        {tunnel.lastHealthCheckAt && (
          <div className="metric">
            <span className="metric__label">Last check</span>
            <span className="metric__value">
              {new Date(tunnel.lastHealthCheckAt).toLocaleTimeString()}
            </span>
          </div>
        )}
      </div>

      {/* ── Error banner ─────────────────────────────────────────────────── */}
      {tunnel.lastError && (
        <div className="tunnel-card__error">
          <AlertTriangle size={13} />
          <span>
            {tunnel.lastError.kind}: {tunnel.lastError.message}
          </span>
        </div>
      )}

      {/* ── Expanded details ─────────────────────────────────────────────── */}
      {expanded && (
        <div className="tunnel-card__details">
          <h4>Configuration</h4>
          <dl className="details-grid">
            <dt>Identity file</dt>
            <dd>{tunnel.config.identityFile ?? "default"}</dd>
            <dt>Max reconnects</dt>
            <dd>{tunnel.config.reconnect.maxAttempts}</dd>
            <dt>Backoff</dt>
            <dd>
              {tunnel.config.reconnect.initialDelayMs}ms →{" "}
              {tunnel.config.reconnect.maxDelayMs}ms ×
              {tunnel.config.reconnect.multiplier}
            </dd>
            <dt>Health interval</dt>
            <dd>{tunnel.config.healthCheck.intervalMs}ms</dd>
            <dt>Fail threshold</dt>
            <dd>{tunnel.config.healthCheck.failureThreshold} checks</dd>
            {tunnel.config.extraArgs.length > 0 && (
              <>
                <dt>Extra args</dt>
                <dd>
                  <code>{tunnel.config.extraArgs.join(" ")}</code>
                </dd>
              </>
            )}
          </dl>
        </div>
      )}
    </div>
  );
}
