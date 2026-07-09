import { useEffect, useState } from "react";

import { MarkdownViewer } from "../MarkdownViewer";
import { docsList, docsRead, type DocEntry } from "../../lib/docs";

const DEFAULT_DOC = "README.md";

export function HelpApp() {
  const [entries, setEntries] = useState<DocEntry[]>([]);
  const [selectedPath, setSelectedPath] = useState(DEFAULT_DOC);
  const [markdown, setMarkdown] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function loadIndex() {
      try {
        const list = await docsList();
        if (cancelled) {
          return;
        }
        setEntries(list);
        const hasDefault = list.some((entry) => entry.path === DEFAULT_DOC);
        setSelectedPath(hasDefault ? DEFAULT_DOC : list[0]?.path ?? DEFAULT_DOC);
      } catch (loadError) {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : String(loadError));
        }
      }
    }

    void loadIndex();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!selectedPath) {
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    async function loadDoc() {
      try {
        const content = await docsRead(selectedPath);
        if (!cancelled) {
          setMarkdown(content);
        }
      } catch (loadError) {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : String(loadError));
          setMarkdown("");
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    void loadDoc();
    return () => {
      cancelled = true;
    };
  }, [selectedPath]);

  return (
    <div className="help-app">
      <aside className="help-app-sidebar">
        <div className="help-app-sidebar-header">
          <h2 className="text-sm font-semibold text-nest-foreground">Documentation</h2>
          <p className="text-xs text-nest-muted">Nest framework docs</p>
        </div>
        <nav className="help-app-toc">
          {entries.map((entry) => (
            <button
              key={entry.path}
              type="button"
              className={[
                "help-app-toc-item",
                selectedPath === entry.path ? "help-app-toc-item-active" : "",
              ].join(" ")}
              style={{ paddingLeft: `${12 + entry.depth * 14}px` }}
              onClick={() => setSelectedPath(entry.path)}
              title={entry.path}
            >
              {entry.name}
            </button>
          ))}
        </nav>
      </aside>
      <main className="help-app-content">
        {error ? (
          <div className="help-app-error">{error}</div>
        ) : loading ? (
          <div className="help-app-loading">Loading…</div>
        ) : (
          <MarkdownViewer markdown={markdown} />
        )}
      </main>
    </div>
  );
}
