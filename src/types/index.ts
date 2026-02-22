// ─── Tunnel States ─────────────────────────────────────────────────────────────
export type TunnelState =
  | "STARTING"
  | "HEALTHY"
  | "DEGRADED"
  | "RECONNECTING"
  | "FAILED"
  | "STOPPED";

// ─── Error Classification ──────────────────────────────────────────────────────
export type TunnelErrorKind =
  | "BROKEN_PIPE"
  | "CONNECTION_TIMEOUT"
  | "AUTH_FAILURE"
  | "PORT_IN_USE"
  | "HOST_UNREACHABLE"
  | "PERMISSION_DENIED"
  | "UNKNOWN_HOST"
  | "NETWORK_UNREACHABLE"
  | "UNKNOWN";

export interface TunnelError {
  kind: TunnelErrorKind;
  message: string;
  occurredAt: number; // unix ms
}

// ─── Configuration ─────────────────────────────────────────────────────────────
export interface ReconnectConfig {
  maxAttempts: number;
  initialDelayMs: number;
  maxDelayMs: number;
  multiplier: number;
}

export interface HealthCheckConfig {
  intervalMs: number;
  timeoutMs: number;
  failureThreshold: number;
  recoveryThreshold: number;
}

export interface TunnelConfig {
  name: string;
  sshHost: string;
  sshPort: number;
  sshUser: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  identityFile?: string | null;
  sshPassword?: string | null;
  strictHostChecking: boolean;
  extraArgs: string[];
  reconnect: ReconnectConfig;
  healthCheck: HealthCheckConfig;
}

// ─── Runtime Info ──────────────────────────────────────────────────────────────
export interface TunnelInfo {
  id: string;
  config: TunnelConfig;
  state: TunnelState;
  pid?: number | null;
  stateEnteredAt: number; // unix ms
  lastHealthCheckAt?: number | null;
  reconnectAttempts: number;
  lastError?: TunnelError | null;
  uptimeMs: number;
}

// ─── Log Entry ─────────────────────────────────────────────────────────────────
export type LogLevel = "DEBUG" | "INFO" | "WARN" | "ERROR";

export interface LogEntry {
  tunnelId: string;
  level: LogLevel;
  message: string;
  timestamp: number; // unix ms
}

// ─── Tauri Event Payloads ──────────────────────────────────────────────────────
export interface StateChangedPayload {
  tunnelId: string;
  state: TunnelState;
  message?: string | null;
  timestamp: number;
}

export interface MetricsPayload {
  tunnelId: string;
  uptimeMs: number;
  reconnectAttempts: number;
  lastHealthCheckAt?: number | null;
  pid?: number | null;
}

// ─── Form State ────────────────────────────────────────────────────────────────
export type TunnelFormData = Pick<
  TunnelConfig,
  | "name"
  | "sshHost"
  | "sshPort"
  | "sshUser"
  | "localPort"
  | "remoteHost"
  | "remotePort"
  | "identityFile"
> & {
  sshPassword: string;
  strictHostChecking: boolean;
  extraArgsRaw: string; // space-separated, parsed before submission
};

// ─── UI Helpers ───────────────────────────────────────────────────────────────
export const STATE_META: Record<
  TunnelState,
  { label: string; color: string; bg: string; dot: string }
> = {
  STARTING:     { label: "Starting",     color: "#3b82f6", bg: "rgba(59,130,246,0.12)",  dot: "#3b82f6" },
  HEALTHY:      { label: "Healthy",      color: "#22c55e", bg: "rgba(34,197,94,0.12)",   dot: "#22c55e" },
  DEGRADED:     { label: "Degraded",     color: "#f59e0b", bg: "rgba(245,158,11,0.12)",  dot: "#f59e0b" },
  RECONNECTING: { label: "Reconnecting", color: "#f97316", bg: "rgba(249,115,22,0.12)",  dot: "#f97316" },
  FAILED:       { label: "Failed",       color: "#ef4444", bg: "rgba(239,68,68,0.12)",   dot: "#ef4444" },
  STOPPED:      { label: "Stopped",      color: "#6b7280", bg: "rgba(107,114,128,0.12)", dot: "#6b7280" },
};

export const DEFAULT_RECONNECT: ReconnectConfig = {
  maxAttempts: 10,
  initialDelayMs: 1000,
  maxDelayMs: 60000,
  multiplier: 2.0,
};

export const DEFAULT_HEALTH_CHECK: HealthCheckConfig = {
  intervalMs: 5000,
  timeoutMs: 3000,
  failureThreshold: 3,
  recoveryThreshold: 2,
};

// ─── SSH Connections ───────────────────────────────────────────────────────────

export interface SshConnectionConfig {
  name: string;
  host: string;
  port: number;
  username: string;
  identityFile?: string | null;
  jumpHost?: string | null;
  extraArgs?: string | null;
  description?: string | null;
  tags: string[];
}

export interface SshConnection {
  id: string;
  config: SshConnectionConfig;
  lastConnectedAt?: number | null; // unix ms
  hasPassword: boolean;
}

export type SshConnectionFormData = {
  name: string;
  host: string;
  port: number;
  username: string;
  identityFile: string;
  jumpHost: string;
  extraArgs: string;
  description: string;
  tags: string; // comma-separated, parsed before submission
  password: string;
};
