import clsx from "clsx";
import type { TunnelState } from "../types";
import { STATE_META } from "../types";

interface StatusBadgeProps {
  state: TunnelState;
  className?: string;
}

export function StatusBadge({ state, className }: StatusBadgeProps) {
  const meta = STATE_META[state];
  const isAnimated = state === "STARTING" || state === "RECONNECTING";

  return (
    <span
      className={clsx("status-badge", className)}
      style={
        {
          "--badge-color": meta.color,
          "--badge-bg": meta.bg,
        } as React.CSSProperties
      }
    >
      <span className={clsx("status-dot", isAnimated && "status-dot--pulse")} />
      {meta.label}
    </span>
  );
}
