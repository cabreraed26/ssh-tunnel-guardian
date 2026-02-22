import { Plus, RefreshCw, Shield } from "lucide-react";
import { useTunnelStore } from "../store/tunnelStore";

interface HeaderProps {
  onAdd: () => void;
}

export function Header({ onAdd }: HeaderProps) {
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

      <div className="header__stats">
        {tunnels.length > 0 && (
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
        <button className="btn btn--primary" onClick={onAdd}>
          <Plus size={16} />
          Add Tunnel
        </button>
      </div>
    </header>
  );
}
