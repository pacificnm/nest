import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCircle, faFile, faXmark } from "@fortawesome/free-solid-svg-icons";
import { CodeEditor } from "./CodeEditor";
import type { EditorTab } from "./types";

type EditorTabsProps = {
  tabs: EditorTab[];
  activePath: string | null;
  onFocusTab: (relPath: string) => void;
  onCloseTab: (relPath: string) => void;
  onChangeContent: (relPath: string, content: string) => void;
  onSaveTab: (relPath: string) => void;
};

/** Tab bar + Monaco editor for the active tab, like Kiwi's EditorArea. */
export function EditorTabs({ tabs, activePath, onFocusTab, onCloseTab, onChangeContent, onSaveTab }: EditorTabsProps) {
  const active = tabs.find((tab) => tab.relPath === activePath) ?? null;

  if (tabs.length === 0) {
    return (
      <div className="flex h-full min-h-0 flex-col items-center justify-center bg-nest-background text-nest-muted">
        <FontAwesomeIcon icon={faFile} className="size-8 opacity-30" />
        <p className="mt-3 text-sm">Select a file in the sidebar to open it.</p>
        <p className="mt-1 text-xs opacity-70">Edits save with Ctrl/Cmd+S.</p>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <div className="flex h-8 shrink-0 items-stretch overflow-x-auto border-b border-nest-border">
        {tabs.map((tab) => {
          const selected = tab.relPath === activePath;
          return (
            <div
              key={tab.relPath}
              className={[
                "group flex items-center gap-2 border-r border-nest-border px-3 text-[13px]",
                selected ? "bg-nest-background text-nest-foreground" : "bg-nest-surface text-nest-muted hover:text-nest-foreground",
              ].join(" ")}
            >
              <button type="button" onClick={() => onFocusTab(tab.relPath)} title={tab.relPath} className="max-w-[16rem] truncate">
                {tab.name}
              </button>
              <span className="relative flex size-4 items-center justify-center">
                {tab.dirty ? (
                  <FontAwesomeIcon icon={faCircle} className="size-2 text-nest-foreground group-hover:hidden" title="Unsaved changes" />
                ) : null}
                <button
                  type="button"
                  onClick={() => onCloseTab(tab.relPath)}
                  title="Close"
                  aria-label={`Close ${tab.name}`}
                  className={[
                    "flex size-4 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/20 hover:text-nest-foreground",
                    tab.dirty ? "hidden group-hover:flex" : "opacity-0 group-hover:opacity-100",
                  ].join(" ")}
                >
                  <FontAwesomeIcon icon={faXmark} className="size-2.5" />
                </button>
              </span>
            </div>
          );
        })}
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">
        {active ? (
          active.loading ? (
            <p className="p-3 text-xs text-nest-muted">Loading {active.name}…</p>
          ) : active.error ? (
            <p className="p-3 text-xs text-nest-error">{active.error}</p>
          ) : (
            <CodeEditor tab={active} onChange={onChangeContent} onSave={onSaveTab} />
          )
        ) : null}
      </div>
    </div>
  );
}
