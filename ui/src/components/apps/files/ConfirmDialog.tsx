import { useEffect, useId, useState } from "react";

type ConfirmDialogProps = {
  open: boolean;
  title: string;
  description: string;
  confirmLabel?: string;
  danger?: boolean;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
};

/** Confirmation modal for destructive actions (e.g. Delete). */
export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel = "Confirm",
  danger = true,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const titleId = useId();
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setError(null);
      setSubmitting(false);
    }
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onCancel();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onCancel]);

  if (!open) {
    return null;
  }

  const confirm = () => {
    if (submitting) {
      return;
    }
    setSubmitting(true);
    setError(null);
    onConfirm()
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => setSubmitting(false));
  };

  return (
    <div
      className="fixed inset-0 z-[70] flex items-center justify-center bg-black/40 p-4"
      onClick={onCancel}
      role="presentation"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="w-full max-w-sm overflow-hidden rounded-nest-lg border border-nest-border bg-nest-surface shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="border-b border-nest-border px-5 py-3">
          <h2 id={titleId} className="text-sm font-semibold">
            {title}
          </h2>
        </header>

        <div className="px-5 py-4">
          <p className="text-xs text-nest-muted">{description}</p>
          {error ? <p className="mt-2 text-xs text-nest-error">{error}</p> : null}
        </div>

        <footer className="flex justify-end gap-2 border-t border-nest-border px-5 py-3">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-nest-md border border-nest-border px-4 py-2 text-sm hover:bg-nest-muted/10"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={confirm}
            disabled={submitting}
            className={[
              "rounded-nest-md px-4 py-2 text-sm font-medium text-white disabled:cursor-default disabled:opacity-50",
              danger ? "bg-nest-error hover:opacity-90" : "bg-nest-primary hover:opacity-90",
            ].join(" ")}
          >
            {submitting ? "Working…" : confirmLabel}
          </button>
        </footer>
      </div>
    </div>
  );
}
