import { Plus, RefreshCw, Shield, Network, Server } from "lucide-react";
import clsx from "clsx";
import { useTunnelStore } from "../store/tunnelStore";

type Tab = "tunnels" | "connections";

interface HeaderProps {
  tab: Tab;
  onTabChange: (t: Tab) => void;
  onAddTunnel: () => void;
  onAddConnection: () => void;
}

export function Header({ tab, onTabChange, onAddTunnel, onAddConnection }: HeaderProps) {
  const tunnels = useTunnelStore((s) => s.tunnels);
  const init = useTunnelStore((s) => s.init);

  const healthy = tunnels.filter((t) => t.state === "HEALTHY").length;
  const degraded = tunnels.filter(
    (t) => t.state === "DEGRADED" || t.state === "RECONNECTING"
  ).length;
  const failed = tunnels.filter((t) => t.state === "FAILED").length;

  return (
    <header className="header">
      <div className="header__brand">
        <Shield size={22} className="header__logo" />
        <span className="header__title">SSH Tunnel Guardian</span>
        <span className="header__version">STG</span>
      </div>

      {/* ── Tab switcher ─────────────────────────────────────────────── */}
      <nav className="header__tabs">
        <button
          className={clsx("header__tab", { "header__tab--active": tab === "tunnels" })}
          onClick={() => onTabChange("tunnels")}
        >
          <Network size={13} />
          Tunnels
          {tunnels.length > 0 && (
            <span className="header__tab-count">{tunnels.length}</span>
          )}
        </button>
        <button
          className={clsx("header__tab", { "header__tab--active": tab === "connections" })}
          onClick={() => onTabChange("connections")}
        >
          <Server size={13} />
          Connections
        </button>
      </nav>

      <div className="header__stats">
        {tab === "tunnels" && tunnels.length > 0 && (
          <>
            <span className="header__stat header__stat--healthy">
              {healthy} healthy
            </span>
            {degraded > 0 && (
              <span className="header__stat header__stat--warn">
                {degraded} degraded
              </span>
            )}
            {failed > 0 && (
              <span className="header__stat header__stat--error">
                {failed} failed
              </span>
            )}
          </>
        )}
      </div>

      <div className="header__actions">
        <button
          className="btn btn--ghost btn--icon"
          onClick={() => init()}
          title="Refresh"
        >
          <RefreshCw size={16} />
        </button>
        {tab === "tunnels" ? (
          <button className="btn btn--primary" onClick={onAddTunnel}>
            <Plus size={16} />
            Add Tunnel
          </button>
        ) : (
          <button className="btn btn--primary" onClick={onAddConnection}>
            <Plus size={16} />
            Add Connection
          </button>
        )}
      </div>
    </header>
  );
}

