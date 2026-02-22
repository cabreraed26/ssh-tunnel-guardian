import { useState } from "react";
import { Terminal, Pencil, Trash2, Key, Server, GitBranch, Tag, KeyRound } from "lucide-react";
import clsx from "clsx";
import type { SshConnection } from "../types";
import { useConnectionStore } from "../store/connectionStore";

interface ConnectionCardProps {
  conn: SshConnection;
  onEdit: (conn: SshConnection) => void;
}

function timeAgo(ms: number): string {
  const diff = Date.now() - ms;
  const s = Math.floor(diff / 1000);
  if (s < 60) return "just now";
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

export function ConnectionCard({ conn, onEdit }: ConnectionCardProps) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  const launchConnection = useConnectionStore((s) => s.launchConnection);
  const removeConnection = useConnectionStore((s) => s.removeConnection);
  const loadingIds = useConnectionStore((s) => s.loadingIds);
  const setError = useConnectionStore((s) => s.setError);

  const isLaunching = loadingIds.includes(conn.id);
  const { config } = conn;
  const sshTarget = `${config.username}@${config.host}${config.port !== 22 ? `:${config.port}` : ""}`;

  const handleLaunch = async () => {
    try {
      await launchConnection(conn.id);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async () => {
    try {
      await removeConnection(conn.id);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="conn-card">
      {/* ── Header ─────────────────────────────────────────────────────────── */}
      <div className="conn-card__header">
        <div className="conn-card__title-row">
          <Server size={14} className="conn-card__server-icon" />
          <span className="conn-card__name">{config.name}</span>
          {conn.hasPassword && (
            <span className="conn-card__pwd-badge" title="Password saved in keychain">
              <KeyRound size={10} />
            </span>
          )}
        </div>
        <div className="conn-card__actions">
          {!confirmDelete ? (
            <>
              <button
                className="btn btn--ghost btn--icon btn--sm"
                title="Edit"
                onClick={() => onEdit(conn)}
              >
                <Pencil size={13} />
              </button>
              <button
                className="btn btn--ghost btn--icon btn--sm btn--ghost-danger"
                title="Delete"
                onClick={() => setConfirmDelete(true)}
              >
                <Trash2 size={13} />
              </button>
            </>
          ) : (
            <div className="conn-card__confirm-delete">
              <span className="conn-card__confirm-text">Delete?</span>
              <button
                className="btn btn--danger btn--sm"
                onClick={handleDelete}
              >
                Yes
              </button>
              <button
                className="btn btn--ghost btn--sm"
                onClick={() => setConfirmDelete(false)}
              >
                No
              </button>
            </div>
          )}
        </div>
      </div>

      {/* ── Body ───────────────────────────────────────────────────────────── */}
      <div className="conn-card__body">
        <div className="conn-card__target">
          <code className="conn-card__ssh-target">{sshTarget}</code>
        </div>

        {config.identityFile && (
          <div className="conn-card__meta-row">
            <Key size={11} className="conn-card__meta-icon" />
            <span className="conn-card__meta-text">{config.identityFile}</span>
          </div>
        )}

        {config.jumpHost && (
          <div className="conn-card__meta-row">
            <GitBranch size={11} className="conn-card__meta-icon" />
            <span className="conn-card__meta-text">via {config.jumpHost}</span>
          </div>
        )}

        {config.description && (
          <p className="conn-card__description">{config.description}</p>
        )}

        {config.tags.length > 0 && (
          <div className="conn-card__tags">
            <Tag size={11} className="conn-card__meta-icon" />
            {config.tags.map((tag) => (
              <span key={tag} className="conn-card__tag">{tag}</span>
            ))}
          </div>
        )}
      </div>

      {/* ── Footer ─────────────────────────────────────────────────────────── */}
      <div className="conn-card__footer">
        {conn.lastConnectedAt && (
          <span className="conn-card__last-connected">
            Last connected {timeAgo(conn.lastConnectedAt)}
          </span>
        )}
        <button
          className={clsx("btn btn--primary btn--sm conn-card__launch-btn", { "btn--loading": isLaunching })}
          disabled={isLaunching}
          onClick={handleLaunch}
          title="Open SSH session in terminal"
        >
          <Terminal size={13} />
          {isLaunching ? "Opening…" : "Connect"}
        </button>
      </div>
    </div>
  );
}
