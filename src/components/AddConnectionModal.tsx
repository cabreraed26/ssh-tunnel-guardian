import { useEffect, useState } from "react";
import { X, KeyRound } from "lucide-react";
import type { SshConnection, SshConnectionConfig, SshConnectionFormData } from "../types";
import { useConnectionStore } from "../store/connectionStore";

interface AddConnectionModalProps {
  editTarget?: SshConnection | null;
  onClose: () => void;
}

const EMPTY_FORM: SshConnectionFormData = {
  name: "",
  host: "",
  port: 22,
  username: "",
  identityFile: "",
  jumpHost: "",
  extraArgs: "",
  description: "",
  tags: "",
  password: "",
};

function formToConfig(form: SshConnectionFormData): SshConnectionConfig {
  return {
    name: form.name.trim(),
    host: form.host.trim(),
    port: Number(form.port),
    username: form.username.trim(),
    identityFile: form.identityFile.trim() || null,
    jumpHost: form.jumpHost.trim() || null,
    extraArgs: form.extraArgs.trim() || null,
    description: form.description.trim() || null,
    tags: form.tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean),
  };
}

export function AddConnectionModal({ editTarget, onClose }: AddConnectionModalProps) {
  const addConnection = useConnectionStore((s) => s.addConnection);
  const updateConnection = useConnectionStore((s) => s.updateConnection);
  const savePassword = useConnectionStore((s) => s.savePassword);
  const deletePassword = useConnectionStore((s) => s.deletePassword);
  const setError = useConnectionStore((s) => s.setError);

  const [form, setForm] = useState<SshConnectionFormData>(EMPTY_FORM);
  const [submitting, setSubmitting] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [removePassword, setRemovePassword] = useState(false);

  const isEditing = !!editTarget;
  const existingPassword = !!editTarget?.hasPassword;

  // Pre-fill when editing.
  useEffect(() => {
    if (editTarget) {
      const c = editTarget.config;
      setForm({
        name: c.name,
        host: c.host,
        port: c.port,
        username: c.username,
        identityFile: c.identityFile ?? "",
        jumpHost: c.jumpHost ?? "",
        extraArgs: c.extraArgs ?? "",
        description: c.description ?? "",
        tags: c.tags.join(", "),
        password: "", // never pre-fill; shown as placeholder if saved
      });
      setRemovePassword(false);
    }
  }, [editTarget]);

  const set =
    (field: keyof SshConnectionFormData) =>
    (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
      setForm((prev) => ({ ...prev, [field]: e.target.value }));

  function validate(): string | null {
    if (!form.name.trim()) return "Name is required.";
    if (!form.host.trim()) return "Host is required.";
    if (!form.username.trim()) return "Username is required.";
    if (!form.port || form.port < 1 || form.port > 65535)
      return "Port must be 1–65535.";
    return null;
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const err = validate();
    if (err) { setValidationError(err); return; }
    setValidationError(null);
    setSubmitting(true);
    try {
      const config = formToConfig(form);
      if (isEditing && editTarget) {
        await updateConnection(editTarget.id, config);
        // Update keychain: replace, remove, or leave unchanged.
        if (removePassword) {
          await deletePassword(editTarget.id);
        } else if (form.password.trim()) {
          await savePassword(editTarget.id, form.password.trim());
        }
        // If both are empty and removePassword is false: keep existing.
      } else {
        const conn = await addConnection(config);
        if (form.password.trim()) {
          await savePassword(conn.id, form.password.trim());
        }
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
    const handler = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  return (
    <div
      className="modal-overlay"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="modal" role="dialog" aria-modal="true">
        <div className="modal__header">
          <h2 className="modal__title">
            {isEditing ? `Edit — ${editTarget!.config.name}` : "Add Connection"}
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
          {/* ── Identity ─────────────────────────────────────────────────── */}
          <fieldset className="form-section">
            <legend>Connection identity</legend>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-name">Name *</label>
                <input
                  id="c-name"
                  type="text"
                  placeholder="production-server"
                  value={form.name}
                  onChange={set("name")}
                  required
                  autoFocus
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-description">Description</label>
                <input
                  id="c-description"
                  type="text"
                  placeholder="Main production bastion (optional)"
                  value={form.description}
                  onChange={set("description")}
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-tags">
                  Tags <span className="form-hint">(comma-separated)</span>
                </label>
                <input
                  id="c-tags"
                  type="text"
                  placeholder="production, aws, us-east (optional)"
                  value={form.tags}
                  onChange={set("tags")}
                />
              </div>
            </div>
          </fieldset>

          {/* ── SSH Target ───────────────────────────────────────────────── */}
          <fieldset className="form-section">
            <legend>SSH server</legend>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-username">User *</label>
                <input
                  id="c-username"
                  type="text"
                  placeholder="ubuntu"
                  value={form.username}
                  onChange={set("username")}
                  required
                />
              </div>
              <div className="form-field form-field--grow-2">
                <label htmlFor="c-host">Host *</label>
                <input
                  id="c-host"
                  type="text"
                  placeholder="bastion.example.com"
                  value={form.host}
                  onChange={set("host")}
                  required
                />
              </div>
              <div className="form-field form-field--narrow">
                <label htmlFor="c-port">Port</label>
                <input
                  id="c-port"
                  type="number"
                  min={1}
                  max={65535}
                  value={form.port}
                  onChange={set("port")}
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-identity">Identity file</label>
                <input
                  id="c-identity"
                  type="text"
                  placeholder="~/.ssh/id_rsa (optional)"
                  value={form.identityFile}
                  onChange={set("identityFile")}
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-password">
                  Password{" "}
                  <span className="form-hint">
                    (stored in OS keychain — leave blank to use key auth)
                  </span>
                </label>
                {isEditing && existingPassword && !removePassword ? (
                  <div className="conn-password-row">
                    <div className="conn-password-saved">
                      <KeyRound size={12} />
                      Password saved in keychain
                    </div>
                    <button
                      type="button"
                      className="btn btn--ghost btn--sm btn--ghost-danger"
                      onClick={() => setRemovePassword(true)}
                    >
                      Remove
                    </button>
                    <span className="form-hint">or type a new one to replace:</span>
                    <input
                      id="c-password"
                      type="password"
                      autoComplete="new-password"
                      placeholder="New password…"
                      value={form.password}
                      onChange={set("password")}
                      className="conn-password-replace"
                    />
                  </div>
                ) : removePassword ? (
                  <div className="conn-password-row">
                    <span className="conn-password-removing">
                      Password will be removed on save.
                    </span>
                    <button
                      type="button"
                      className="btn btn--ghost btn--sm"
                      onClick={() => setRemovePassword(false)}
                    >
                      Cancel
                    </button>
                  </div>
                ) : (
                  <input
                    id="c-password"
                    type="password"
                    autoComplete="new-password"
                    placeholder={isEditing ? "Leave blank to keep unchanged" : "Leave blank for key auth"}
                    value={form.password}
                    onChange={set("password")}
                  />
                )}
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-jump">
                  Jump host <span className="form-hint">(optional — user@host[:port])</span>
                </label>
                <input
                  id="c-jump"
                  type="text"
                  placeholder="admin@jump.host:22"
                  value={form.jumpHost}
                  onChange={set("jumpHost")}
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-field form-field--grow">
                <label htmlFor="c-extra">
                  Extra SSH args <span className="form-hint">(optional)</span>
                </label>
                <input
                  id="c-extra"
                  type="text"
                  placeholder="-o ServerAliveInterval=60"
                  value={form.extraArgs}
                  onChange={set("extraArgs")}
                />
              </div>
            </div>
          </fieldset>

          {/* ── Validation + Submit ──────────────────────────────────────── */}
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
                ? isEditing ? "Saving…" : "Adding…"
                : isEditing ? "Save" : "Add Connection"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
