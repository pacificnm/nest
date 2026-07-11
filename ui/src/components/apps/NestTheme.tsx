/**
 * Nest Theme - Theme token viewer for Nest Desktop
 *
 * Same layout as the Component Library: a left panel listing every built-in
 * Nest theme, and a main panel showing the selected theme's tokens (colors,
 * status colors, typography, spacing, and radius scale).
 */

import { useEffect, useMemo, useState } from "react";
import { Palette } from "lucide-react";
import { themeLabel, themesList, type ThemeDefinition, type TypographyStyle } from "../../lib/themes";

export function NestTheme() {
  const [themes, setThemes] = useState<ThemeDefinition[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    themesList()
      .then((list) => {
        if (cancelled) {
          return;
        }
        setThemes(list);
        setSelectedId((current) => current ?? list[0]?.id ?? null);
      })
      .catch((loadError) => {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : String(loadError));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const selected = useMemo(() => themes.find((theme) => theme.id === selectedId) ?? null, [themes, selectedId]);

  return (
    <div className="help-app">
      <aside className="help-app-sidebar">
        <div className="help-app-sidebar-header">
          <h2 className="text-sm font-semibold text-nest-foreground">Nest Theme</h2>
          <p className="text-xs text-nest-muted">Built-in theme tokens</p>
        </div>
        <nav className="help-app-toc">
          {themes.map((theme) => (
            <button
              key={theme.id}
              type="button"
              className={["help-app-toc-item", "flex items-center gap-2", selectedId === theme.id ? "help-app-toc-item-active" : ""].join(" ")}
              onClick={() => setSelectedId(theme.id)}
              title={theme.id}
            >
              <span
                className="size-3 shrink-0 rounded-full border border-nest-border"
                style={{ backgroundColor: theme.colors.primary }}
                aria-hidden
              />
              <span className="truncate">{themeLabel(theme.id)}</span>
            </button>
          ))}
        </nav>
      </aside>
      <main className="help-app-content">
        {error ? (
          <div className="help-app-error">{error}</div>
        ) : !selected ? (
          <div className="help-app-loading">Loading…</div>
        ) : (
          <ThemeDetail theme={selected} />
        )}
      </main>
    </div>
  );
}

function ThemeDetail({ theme }: { theme: ThemeDefinition }) {
  const colorEntries: { label: string; value?: string }[] = [
    { label: "Background", value: theme.colors.background },
    { label: "Foreground", value: theme.colors.foreground },
    { label: "Primary", value: theme.colors.primary },
    { label: "Secondary", value: theme.colors.secondary },
    { label: "Surface", value: theme.colors.surface },
    { label: "Border", value: theme.colors.border },
    { label: "Accent", value: theme.colors.accent },
    { label: "Muted", value: theme.colors.muted },
  ];

  const statusEntries: { label: string; value: string }[] = [
    { label: "Success", value: theme.status.success },
    { label: "Warning", value: theme.status.warning },
    { label: "Error", value: theme.status.error },
    { label: "Info", value: theme.status.info },
  ];

  const typographyEntries: { label: string; style?: TypographyStyle }[] = [
    { label: "Body", style: theme.typography.body },
    { label: "Heading", style: theme.typography.heading },
    { label: "Caption", style: theme.typography.caption },
    { label: "Mono", style: theme.typography.mono },
  ];

  const spacingEntries = Object.entries(theme.spacing).filter(
    (entry): entry is [string, number] => entry[1] !== undefined,
  );
  const maxSpacing = Math.max(...spacingEntries.map(([, value]) => value));

  const radiusEntries = Object.entries(theme.radius).filter(
    (entry): entry is [string, number] => entry[1] !== undefined,
  );

  return (
    <div className="max-w-3xl space-y-8">
      <header className="flex items-center gap-3">
        <Palette className="size-6 text-nest-primary" />
        <div>
          <h1 className="text-lg font-semibold text-nest-foreground">{themeLabel(theme.id)}</h1>
          <p className="text-xs text-nest-muted">
            <code className="text-nest-foreground">{theme.id}</code> ·{" "}
            <span className="capitalize">{theme.mode}</span> mode
          </p>
        </div>
      </header>

      <section>
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Colors</h2>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          {colorEntries
            .filter((entry): entry is { label: string; value: string } => Boolean(entry.value))
            .map((entry) => (
              <ColorSwatch key={entry.label} label={entry.label} value={entry.value} />
            ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Status Colors</h2>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          {statusEntries.map((entry) => (
            <ColorSwatch key={entry.label} label={entry.label} value={entry.value} />
          ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Typography</h2>
        <div className="space-y-3">
          {typographyEntries
            .filter((entry): entry is { label: string; style: TypographyStyle } => Boolean(entry.style))
            .map((entry) => (
              <div key={entry.label} className="rounded-nest-md border border-nest-border p-3">
                <div className="mb-1 flex items-baseline justify-between gap-2">
                  <span className="text-xs font-semibold text-nest-foreground">{entry.label}</span>
                  <span className="truncate font-mono text-[11px] text-nest-muted">
                    {entry.style.font_family} · {entry.style.size}px / {entry.style.line_height}px · {entry.style.weight}
                  </span>
                </div>
                <p
                  className="truncate text-nest-foreground"
                  style={{
                    fontFamily: entry.style.font_family,
                    fontSize: `${Math.min(entry.style.size, 28)}px`,
                    lineHeight: `${entry.style.line_height}px`,
                    fontWeight: entry.style.weight,
                  }}
                >
                  The quick brown fox jumps over the lazy dog
                </p>
              </div>
            ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Spacing</h2>
        <div className="space-y-1.5">
          {spacingEntries.map(([key, value]) => (
            <div key={key} className="flex items-center gap-3">
              <span className="w-10 shrink-0 font-mono text-xs text-nest-muted">{key}</span>
              <div
                className="h-3 rounded-nest-sm bg-nest-primary/60"
                style={{ width: `${Math.max((value / maxSpacing) * 200, 4)}px` }}
              />
              <span className="font-mono text-xs text-nest-muted">{value}px</span>
            </div>
          ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-nest-muted">Radius</h2>
        <div className="flex flex-wrap items-end gap-4">
          {radiusEntries.map(([key, value]) => (
            <div key={key} className="flex flex-col items-center gap-1.5">
              <div
                className="size-12 border border-nest-border bg-nest-surface"
                style={{ borderRadius: `${Math.min(value, 24)}px` }}
              />
              <span className="font-mono text-[11px] text-nest-muted">
                {key}: {value}px
              </span>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

function ColorSwatch({ label, value }: { label: string; value: string }) {
  return (
    <div className="overflow-hidden rounded-nest-md border border-nest-border">
      <div className="h-12 w-full" style={{ backgroundColor: value }} />
      <div className="px-2 py-1.5">
        <p className="text-xs font-medium text-nest-foreground">{label}</p>
        <p className="font-mono text-[11px] text-nest-muted">{value}</p>
      </div>
    </div>
  );
}
