/**
 * Bootstraps Tauri event subscriptions that feed the Zustand store.
 * Call this once at the root of the application.
 */
import { useEffect } from "react";
import { events } from "../lib/tauriApi";
import { useTunnelStore } from "../store/tunnelStore";

export function useTauriEvents(): void {
  const applyStateChange = useTunnelStore((s) => s.applyStateChange);
  const applyMetrics = useTunnelStore((s) => s.applyMetrics);
  const appendLog = useTunnelStore((s) => s.appendLog);

  useEffect(() => {
    const unlistenPromises = [
      events.onStateChanged(applyStateChange),
      events.onMetrics(applyMetrics),
      events.onLog(appendLog),
    ];

    return () => {
      // Cleanup all listeners on unmount.
      unlistenPromises.forEach((p) => p.then((fn) => fn()));
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}
