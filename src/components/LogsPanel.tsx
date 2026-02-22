import { useEffect, useRef } from "react";
import { X, Trash2 } from "lucide-react";
import clsx from "clsx";
import type { LogEntry } from "../types";
import { useTunnelStore } from "../store/tunnelStore";

interface LogsPanelProps {
  tunnelId: string;
  tunnelName: string;
  onClose: () => void;
}

const LEVEL_CLASS: Record<LogEntry["level"], string> = {
  DEBUG: "log-line--debug",
  INFO: "log-line--info",
  WARN: "log-line--warn",
  ERROR: "log-line--error",
};

function formatTime(ts: number): string {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  const ms = String(d.getMilliseconds()).padStart(3, "0");
  return `${hh}:${mm}:${ss}.${ms}`;
}

export function LogsPanel({ tunnelId, tunnelName, onClose }: LogsPanelProps) {
  const logs = useTunnelStore((s) => s.logs[tunnelId] ?? []);
  const fetchLogs = useTunnelStore((s) => s.fetchLogs);
  const clearLogs = useTunnelStore((s) => s.clearLogs);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Initial fetch
  useEffect(() => {
    fetchLogs(tunnelId, 200);
  }, [tunnelId, fetchLogs]);

  // Auto-scroll on new entries
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs.length]);

  return (
    <div className="logs-panel">
      <div className="logs-panel__header">
        <span className="logs-panel__title">
          Logs — <strong>{tunnelName}</strong>
        </span>
        <div className="logs-panel__actions">
          <button
            className="btn btn--ghost btn--icon btn--sm"
            onClick={() => clearLogs(tunnelId)}
            title="Clear logs"
          >
            <Trash2 size={14} />
          </button>
          <button
            className="btn btn--ghost btn--icon btn--sm"
            onClick={onClose}
            title="Close"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      <div className="logs-panel__body">
        {logs.length === 0 ? (
          <p className="logs-panel__empty">No log entries yet.</p>
        ) : (
          logs.map((entry, idx) => (
            <div
              key={idx}
              className={clsx("log-line", LEVEL_CLASS[entry.level])}
            >
              <span className="log-line__time">{formatTime(entry.timestamp)}</span>
              <span className="log-line__level">{entry.level}</span>
              <span className="log-line__msg">{entry.message}</span>
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
