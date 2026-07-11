/**
 * Claude Config — API Key category. Standalone form (unlike Agent settings,
 * there's no shared store hook here — this is the only consumer of
 * `claudeSettingsGet`/`Save`, so local state is simplest).
 */

import { useEffect, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCircleCheck, faKey, faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
import { claudeSettingsGet, claudeSettingsSave } from "../../../lib/claude";

export function ApiKeyPanel() {
  const [apiKey, setApiKey] = useState("");
  const [savedKeyPresent, setSavedKeyPresent] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ kind: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    let cancelled = false;
    claudeSettingsGet()
      .then((settings) => {
        if (cancelled) {
          return;
        }
        setApiKey(settings.apiKey);
        setSavedKeyPresent(settings.apiKey.trim().length > 0);
      })
      .catch((error) => {
        if (!cancelled) {
          setMessage({ kind: "error", text: error instanceof Error ? error.message : String(error) });
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSave() {
    setSaving(true);
    setMessage(null);
    try {
      const saved = await claudeSettingsSave({ apiKey: apiKey.trim() });
      setSavedKeyPresent(saved.apiKey.trim().length > 0);
      setMessage({ kind: "success", text: "Saved." });
    } catch (error) {
      setMessage({ kind: "error", text: error instanceof Error ? error.message : String(error) });
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="max-w-xl space-y-6">
      <div>
        <h1 className="text-lg font-semibold text-nest-foreground">API Key</h1>
        <p className="text-xs text-nest-muted">
          Your Anthropic API key, used by the Skills and Agents tabs to call{" "}
          <code className="text-nest-foreground">api.anthropic.com</code> directly. Stored in Nest
          Desktop's <code className="text-nest-foreground">config.toml</code> (<code className="text-nest-foreground">[claude]</code> section) —
          same plaintext-on-this-machine trust model as other integration tokens.
        </p>
      </div>

      {message ? (
        <p
          className={[
            "rounded-nest-sm border px-3 py-2 text-xs",
            message.kind === "error"
              ? "border-nest-error/30 bg-nest-error/10 text-nest-error"
              : "border-nest-success/30 bg-nest-success/10 text-nest-success",
          ].join(" ")}
        >
          {message.text}
        </p>
      ) : null}

      <div className="flex items-center gap-2 rounded-nest-sm border border-nest-border bg-nest-surface px-2 py-1.5">
        <FontAwesomeIcon
          icon={savedKeyPresent ? faCircleCheck : faTriangleExclamation}
          className={savedKeyPresent ? "size-3.5 text-nest-success" : "size-3.5 text-nest-muted"}
        />
        <span className="text-xs text-nest-muted">
          {savedKeyPresent ? "API key configured" : "No API key configured yet"}
        </span>
      </div>

      <section className="space-y-2">
        <label className="block">
          <span className="mb-0.5 flex items-center gap-1.5 text-[11px] text-nest-muted">
            <FontAwesomeIcon icon={faKey} className="size-3" />
            Anthropic API key
          </span>
          <input
            value={apiKey}
            onChange={(event) => setApiKey(event.target.value)}
            type="password"
            autoComplete="off"
            spellCheck={false}
            placeholder="sk-ant-…"
            disabled={loading}
            className="h-8 w-full rounded-nest-sm border border-nest-border bg-nest-surface px-2 font-mono text-sm disabled:opacity-50"
          />
        </label>
      </section>

      <button
        type="button"
        onClick={() => void handleSave()}
        disabled={saving || loading}
        className="h-9 w-full rounded-nest-sm bg-nest-primary text-sm font-medium text-white hover:opacity-90 disabled:opacity-50"
      >
        {saving ? "Saving…" : "Save"}
      </button>
    </div>
  );
}
