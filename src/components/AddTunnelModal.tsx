import { useEffect, useState } from "react";
import { X } from "lucide-react";
import type { TunnelConfig, TunnelFormData, TunnelInfo } from "../types";
import {
  DEFAULT_HEALTH_CHECK,
  DEFAULT_RECONNECT,
} from "../types";
import { useTunnelStore } from "../store/tunnelStore";

interface AddTunnelModalProps {
  /** Pass an existing tunnel to edit it; undefined = create new. */
  editTarget?: TunnelInfo | null;
  onClose: () => void;
}

const EMPTY_FORM: TunnelFormData = {
  name: "",
  sshHost: "",
  sshPort: 22,
  sshUser: "",
  localPort: 0,
  remoteHost: "localhost",
  remotePort: 0,
  identityFile: "",
  sshPassword: "",
  strictHostChecking: true,
  extraArgsRaw: "",
};

function formToConfig(form: TunnelFormData): TunnelConfig {
  return {
    name: form.name.trim(),
    sshHost: form.sshHost.trim(),
    sshPort: Number(form.sshPort),
    sshUser: form.sshUser.trim(),
    localPort: Number(form.localPort),
    remoteHost: form.remoteHost.trim(),
    remotePort: Number(form.remotePort),
    identityFile: form.identityFile?.trim() || null,
    sshPassword: form.sshPassword.trim() || null,
    strictHostChecking: form.strictHostChecking,
    extraArgs: form.extraArgsRaw
      .split(/\s+/)
      .map((s) => s.trim())
      .filter(Boolean),
    reconnect: DEFAULT_RECONNECT,
    healthCheck: DEFAULT_HEALTH_CHECK,
  };
}

export function AddTunnelModal({ editTarget, onClose }: AddTunnelModalProps) {
  const addTunnel = useTunnelStore((s) => s.addTunnel);
  const updateTunnel = useTunnelStore((s) => s.updateTunnel);
  const setError = useTunnelStore((s) => s.setError);

  const [form, setForm] = useState<TunnelFormData>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const isEditing = !!editTarget;

  // Pre-fill form when editing.
  useEffect(() => {
    if (editTarget) {
      const c = editTarget.config;
      setForm({
        name: c.name,
        sshHost: c.sshHost,
        sshPort: c.sshPort,
        sshUser: c.sshUser,
        localPort: c.localPort,
        remoteHost: c.remoteHost,
        remotePort: c.remotePort,
        identityFile: c.identityFile ?? "",
        sshPassword: c.sshPassword ?? "",
        strictHostChecking: c.strictHostChecking,
        extraArgsRaw: c.extraArgs.join(" "),
      });
    }
  }, [editTarget]);

  const set = (field: keyof TunnelFormData) =>
    (e: React.ChangeEvent<HTMLInputElement>) =>
      setForm((prev) => ({ ...prev, [field]: e.target.value }));

  function validate(): string | null {
    if (!form.name.trim()) return "Name is required.";
    if (!form.sshHost.trim()) return "SSH host is required.";
    if (!form.sshUser.trim()) return "SSH user is required.";
    if (!form.localPort || form.localPort < 1 || form.localPort > 65535)
      return "Local port must be 1–65535.";
    if (!form.remoteHost.trim()) return "Remote host is required.";
    if (!form.remotePort || form.remotePort < 1 || form.remotePort > 65535)
      return "Remote port must be 1–65535.";
    return null;
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const err = validate();
    if (err) {
      setValidationError(err);
      return;
    }
    setValidationError(null);
    setSubmitting(true);
    try {
      const config = formToConfig(form);
      if (isEditing && editTarget) {
        await updateTunnel(editTarget.id, config);
      } else {
        await addTunnel(config);
      }
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSubmitting(false);
    }
  }

  // Close on Escape.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  return (
    <div className="modal-overlay" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal" role="dialog" aria-modal="true">
        <div className="modal__header">
          <h2 className="modal__title">
            {isEditing ? `Edit — ${editTarget!.config.name}` : "Add Tunnel"}
          </h2>
          <button
            className="btn btn--ghost btn--icon btn--sm"
            onClick={onClose}
            aria-label="Close"
          >
            <X size={16} />
          </button>
        </div>

        <form className="modal__form" onSubmit={handleSubmit}>
          {/* ── Identity ───────────────────────────────────────────────── */}
          <fieldset className="form-section">
            <legend>Tunnel identity</legend>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="f-name">Name *</label>
                <input
                  id="f-name"
                  type="text"
                  placeholder="production-db"
                  value={form.name}
                  onChange={set("name")}
                  required
                  autoFocus
                />
              </div>
            </div>
          </fieldset>

          {/* ── SSH Target ─────────────────────────────────────────────── */}
          <fieldset className="form-section">
            <legend>SSH server</legend>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="f-sshUser">User *</label>
                <input
                  id="f-sshUser"
                  type="text"
                  placeholder="ubuntu"
                  value={form.sshUser}
                  onChange={set("sshUser")}
                  required
                />
              </div>
              <div className="form-field form-field--grow-2">
                <label htmlFor="f-sshHost">Host *</label>
                <input
                  id="f-sshHost"
                  type="text"
                  placeholder="bastion.example.com"
                  value={form.sshHost}
                  onChange={set("sshHost")}
                  required
                />
              </div>
              <div className="form-field form-field--narrow">
                <label htmlFor="f-sshPort">Port</label>
                <input
                  id="f-sshPort"
                  type="number"
                  min={1}
                  max={65535}
                  value={form.sshPort}
                  onChange={set("sshPort")}
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="f-identity">Identity file</label>
                <input
                  id="f-identity"
                  type="text"
                  placeholder="~/.ssh/id_rsa (optional)"
                  value={form.identityFile ?? ""}
                  onChange={set("identityFile")}
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="f-password">
                  Password{" "}
                  <span className="form-hint">(optional — uses SSH_ASKPASS, no extra tools required)</span>
                </label>
                <input
                  id="f-password"
                  type="password"
                  autoComplete="new-password"
                  placeholder="Leave empty for key-based auth"
                  value={form.sshPassword}
                  onChange={set("sshPassword")}
                />
              </div>
            </div>
            <div className="form-row">
              <label className="form-checkbox">
                <input
                  type="checkbox"
                  checked={form.strictHostChecking}
                  onChange={(e) =>
                    setForm((prev) => ({ ...prev, strictHostChecking: e.target.checked }))
                  }
                />
                <span>Verify host key (recommended — uncheck for self-signed / new hosts)</span>
              </label>
            </div>
          </fieldset>

          {/* ── Port forwarding ────────────────────────────────────────── */}
          <fieldset className="form-section">
            <legend>Port forwarding — -L localPort:remoteHost:remotePort</legend>
            <div className="form-row">
              <div className="form-field form-field--narrow">
                <label htmlFor="f-localPort">Local port *</label>
                <input
                  id="f-localPort"
                  type="number"
                  min={1}
                  max={65535}
                  placeholder="5432"
                  value={form.localPort || ""}
                  onChange={set("localPort")}
                  required
                />
              </div>
              <div className="form-field form-field--grow">
                <label htmlFor="f-remoteHost">Remote host *</label>
                <input
                  id="f-remoteHost"
                  type="text"
                  placeholder="db.internal"
                  value={form.remoteHost}
                  onChange={set("remoteHost")}
                  required
                />
              </div>
              <div className="form-field form-field--narrow">
                <label htmlFor="f-remotePort">Remote port *</label>
                <input
                  id="f-remotePort"
                  type="number"
                  min={1}
                  max={65535}
                  placeholder="5432"
                  value={form.remotePort || ""}
                  onChange={set("remotePort")}
                  required
                />
              </div>
            </div>
          </fieldset>

          {/* ── Advanced ───────────────────────────────────────────────── */}
          <fieldset className="form-section">
            <legend>Extra SSH args (optional)</legend>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="f-extra">
                  Space-separated flags (e.g. -o StrictHostKeyChecking=no)
                </label>
                <input
                  id="f-extra"
                  type="text"
                  placeholder="-o StrictHostKeyChecking=no"
                  value={form.extraArgsRaw}
                  onChange={set("extraArgsRaw")}
                />
              </div>
            </div>
          </fieldset>

          {validationError && (
            <p className="form-error">{validationError}</p>
          )}

          <div className="modal__footer">
            <button
              type="button"
              className="btn btn--ghost"
              onClick={onClose}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="btn btn--primary"
              disabled={submitting}
            >
              {submitting
                ? "Saving…"
                : isEditing
                ? "Save changes"
                : "Add Tunnel"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
