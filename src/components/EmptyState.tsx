import { Network } from "lucide-react";

export function EmptyState({ onAdd }: { onAdd: () => void }) {
  return (
    <div className="empty-state">
      <div className="empty-state__icon">
        <Network size={48} strokeWidth={1.25} />
      </div>
      <h2 className="empty-state__title">No tunnels yet</h2>
      <p className="empty-state__desc">
        Add your first SSH tunnel to start monitoring port forwarding.
      </p>
      <button className="btn btn--primary" onClick={onAdd}>
        Add Tunnel
      </button>
    </div>
  );
}
