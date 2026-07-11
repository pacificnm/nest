import { useCallback, useEffect, useMemo, useState, type MouseEvent as ReactMouseEvent } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faChevronDown,
  faChevronRight,
  faFile,
  faFolder,
  faFolderOpen,
  faArrowsRotate,
} from "@fortawesome/free-solid-svg-icons";
import {
  filesCopy,
  filesCreateDir,
  filesCreateFile,
  filesDelete,
  filesList,
  filesRename,
  filesReveal,
  formatIpcError,
  joinRel,
  parentRel,
  type FileEntry,
  type FilesRoot,
} from "../../../lib/files";
import { ContextMenu, type ContextMenuItem } from "./ContextMenu";
import { PromptDialog } from "./PromptDialog";
import { ConfirmDialog } from "./ConfirmDialog";

const ROOT = ".";

function baseName(rel: string): string {
  const idx = rel.lastIndexOf("/");
  return idx === -1 ? rel : rel.slice(idx + 1);
}

/** Right-click target: a tree row, or the panel background (repo root). */
type MenuTarget = { relPath: string; isDir: boolean };
type MenuState = { x: number; y: number; target: MenuTarget };
type Clipboard = { op: "cut" | "copy"; relPath: string };
type PromptState = { mode: "newFile" | "newFolder"; parentDir: string } | { mode: "rename"; target: MenuTarget };
type VisibleRow = { entry: FileEntry; depth: number };

function buildVisibleRows(
  entriesByDir: Record<string, FileEntry[]>,
  expanded: ReadonlySet<string>,
  rel: string,
  depth: number,
  rows: VisibleRow[],
): void {
  const entries = entriesByDir[rel];
  if (!entries) {
    return;
  }
  for (const entry of entries) {
    rows.push({ entry, depth });
    if (entry.isDir && expanded.has(entry.relPath)) {
      buildVisibleRows(entriesByDir, expanded, entry.relPath, depth + 1, rows);
    }
  }
}

function pruneSubtree<T>(map: Record<string, T>, rel: string): Record<string, T> {
  const prefix = `${rel}/`;
  const next: Record<string, T> = {};
  for (const [key, value] of Object.entries(map)) {
    if (key === rel || key.startsWith(prefix)) {
      continue;
    }
    next[key] = value;
  }
  return next;
}

type FileTreeProps = {
  root: FilesRoot | null;
  activePath: string | null;
  onOpenFile: (relPath: string, name: string) => void;
  /** Notifies the editor tabs that a file moved (rename/cut-paste), so open tabs stay in sync. */
  onFileRenamed: (from: string, to: string, name: string) => void;
  /** Notifies the editor tabs that a file (or a folder's contents) was deleted. */
  onPathDeleted: (relPath: string) => void;
  onError: (message: string) => void;
};

/** Left-sidebar file/folder tree, like Kiwi's Explorer panel. */
export function FileTree({ root, activePath, onOpenFile, onFileRenamed, onPathDeleted, onError }: FileTreeProps) {
  const [entriesByDir, setEntriesByDir] = useState<Record<string, FileEntry[]>>({});
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState<Set<string>>(new Set());
  const [dirErrors, setDirErrors] = useState<Record<string, string>>({});
  const [menu, setMenu] = useState<MenuState | null>(null);
  const [clipboard, setClipboard] = useState<Clipboard | null>(null);
  const [prompt, setPrompt] = useState<PromptState | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<MenuTarget | null>(null);

  const loadDir = useCallback(
    (rel: string) => {
      setLoading((current) => new Set(current).add(rel));
      void filesList(rel)
        .then((entries) => {
          setEntriesByDir((current) => ({ ...current, [rel]: entries }));
          setDirErrors((current) => {
            const next = { ...current };
            delete next[rel];
            return next;
          });
        })
        .catch((error) => {
          setDirErrors((current) => ({ ...current, [rel]: formatIpcError(error) }));
        })
        .finally(() => {
          setLoading((current) => {
            const next = new Set(current);
            next.delete(rel);
            return next;
          });
        });
    },
    [],
  );

  useEffect(() => {
    if (!root) {
      return;
    }
    loadDir(ROOT);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [root]);

  const toggleDir = useCallback(
    (rel: string) => {
      const willExpand = !expanded.has(rel);
      setExpanded((current) => {
        const next = new Set(current);
        if (next.has(rel)) {
          next.delete(rel);
        } else {
          next.add(rel);
        }
        return next;
      });
      if (willExpand && !entriesByDir[rel]) {
        loadDir(rel);
      }
    },
    [expanded, entriesByDir, loadDir],
  );

  const expandDir = useCallback((rel: string) => {
    if (rel === ROOT) {
      return;
    }
    setExpanded((current) => (current.has(rel) ? current : new Set(current).add(rel)));
  }, []);

  const collapseAll = useCallback(() => setExpanded(new Set()), []);

  const visibleRows = useMemo(() => {
    const rows: VisibleRow[] = [];
    buildVisibleRows(entriesByDir, expanded, ROOT, 0, rows);
    return rows;
  }, [entriesByDir, expanded]);

  const forgetSubtree = useCallback((rel: string) => {
    setEntriesByDir((current) => pruneSubtree(current, rel));
    setExpanded((current) => {
      const pruned = pruneSubtree(Object.fromEntries([...current].map((key) => [key, true])), rel);
      return new Set(Object.keys(pruned));
    });
  }, []);

  const submitPrompt = useCallback(
    async (value: string) => {
      if (!prompt) {
        return;
      }
      if (prompt.mode === "rename") {
        const { target } = prompt;
        const dest = joinRel(parentRel(target.relPath), value);
        if (dest === target.relPath) {
          setPrompt(null);
          return;
        }
        await filesRename(target.relPath, dest);
        forgetSubtree(target.relPath);
        if (!target.isDir) {
          onFileRenamed(target.relPath, dest, value);
        }
        loadDir(parentRel(target.relPath));
        setPrompt(null);
        return;
      }

      const { parentDir, mode } = prompt;
      const rel = joinRel(parentDir, value);
      if (mode === "newFolder") {
        await filesCreateDir(rel);
      } else {
        await filesCreateFile(rel);
      }
      expandDir(parentDir);
      loadDir(parentDir);
      if (mode === "newFile") {
        onOpenFile(rel, value);
      }
      setPrompt(null);
    },
    [prompt, forgetSubtree, onFileRenamed, loadDir, expandDir, onOpenFile],
  );

  const runDelete = useCallback(
    (target: MenuTarget) => {
      void filesDelete(target.relPath)
        .then(() => {
          forgetSubtree(target.relPath);
          onPathDeleted(target.relPath);
          loadDir(parentRel(target.relPath));
        })
        .catch((error) => onError(formatIpcError(error)))
        .finally(() => setConfirmDelete(null));
    },
    [forgetSubtree, onPathDeleted, loadDir, onError],
  );

  const runPaste = useCallback(
    (target: MenuTarget) => {
      if (!clipboard) {
        return;
      }
      const destDir = target.isDir ? target.relPath : parentRel(target.relPath);
      const dest = joinRel(destDir, baseName(clipboard.relPath));
      const op = clipboard.op;
      const source = clipboard.relPath;
      const action = op === "cut" ? filesRename(source, dest) : filesCopy(source, dest);
      void action
        .then(() => {
          if (op === "cut") {
            forgetSubtree(source);
            onFileRenamed(source, dest, baseName(dest));
            loadDir(parentRel(source));
            setClipboard(null);
          }
          expandDir(destDir);
          loadDir(destDir);
        })
        .catch((error) => onError(formatIpcError(error)));
    },
    [clipboard, forgetSubtree, onFileRenamed, loadDir, expandDir, onError],
  );

  const runReveal = useCallback(
    (target: MenuTarget) => {
      void filesReveal(target.relPath).catch((error) => onError(formatIpcError(error)));
    },
    [onError],
  );

  const buildMenuItems = useCallback(
    (target: MenuTarget): ContextMenuItem[] => {
      const isRoot = target.relPath === ROOT;
      const canPaste = clipboard !== null;
      const dirForCreate = target.isDir ? target.relPath : parentRel(target.relPath);

      return [
        { id: "new-file", label: "New File", onSelect: () => setPrompt({ mode: "newFile", parentDir: dirForCreate }) },
        { id: "new-folder", label: "New Folder", onSelect: () => setPrompt({ mode: "newFolder", parentDir: dirForCreate }) },
        { id: "reveal", label: "Open Containing Folder", onSelect: () => runReveal(target) },
        { kind: "separator", id: "sep-1" },
        { id: "cut", label: "Cut", disabled: isRoot, onSelect: () => setClipboard({ op: "cut", relPath: target.relPath }) },
        { id: "copy", label: "Copy", disabled: isRoot, onSelect: () => setClipboard({ op: "copy", relPath: target.relPath }) },
        { id: "paste", label: "Paste", disabled: !canPaste, onSelect: () => runPaste(target) },
        { kind: "separator", id: "sep-2" },
        { id: "rename", label: "Rename", disabled: isRoot, onSelect: () => setPrompt({ mode: "rename", target }) },
        { id: "delete", label: "Delete", danger: true, disabled: isRoot, onSelect: () => setConfirmDelete(target) },
      ];
    },
    [clipboard, runReveal, runPaste],
  );

  const openMenu = useCallback((event: ReactMouseEvent, target: MenuTarget) => {
    event.preventDefault();
    event.stopPropagation();
    setMenu({ x: event.clientX, y: event.clientY, target });
  }, []);

  const rootError = dirErrors[ROOT];
  const rootLoading = loading.has(ROOT) && !entriesByDir[ROOT];

  const promptProps = (() => {
    if (!prompt) {
      return null;
    }
    if (prompt.mode === "rename") {
      const name = baseName(prompt.target.relPath);
      return { title: "Rename", label: "New name", description: `Rename ${name}.`, initialValue: name, confirmLabel: "Rename" };
    }
    const where = prompt.parentDir === ROOT ? (root?.name ?? "the repository root") : prompt.parentDir;
    if (prompt.mode === "newFolder") {
      return { title: "New Folder", label: "Folder name", description: `Create a new folder in ${where}.`, placeholder: "components", confirmLabel: "Create" };
    }
    return { title: "New File", label: "File name", description: `Create a new file in ${where}.`, placeholder: "example.ts", confirmLabel: "Create" };
  })();

  return (
    <div className="flex h-full min-h-0 w-64 shrink-0 flex-col border-r border-nest-border bg-nest-surface">
      <header className="flex h-8 shrink-0 items-center gap-1 border-b border-nest-border px-2">
        <span className="truncate text-xs font-semibold uppercase tracking-wide text-nest-muted">
          {root?.name ?? "Explorer"}
        </span>
        <div className="ml-auto flex items-center gap-0.5">
          <button
            type="button"
            onClick={() => loadDir(ROOT)}
            title="Refresh"
            aria-label="Refresh"
            className="flex size-5 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <FontAwesomeIcon icon={faArrowsRotate} className="size-2.5" />
          </button>
          <button
            type="button"
            onClick={collapseAll}
            title="Collapse all"
            aria-label="Collapse all"
            className="flex size-5 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <FontAwesomeIcon icon={faChevronDown} className="size-2.5" />
          </button>
        </div>
      </header>

      <div
        className="min-h-0 flex-1 overflow-auto"
        onContextMenu={(event) => openMenu(event, { relPath: ROOT, isDir: true })}
      >
        {rootError ? (
          <p className="px-3 py-2 text-xs text-nest-error">{rootError}</p>
        ) : rootLoading ? (
          <p className="px-3 py-2 text-xs text-nest-muted">Loading…</p>
        ) : (
          <ul className="py-1 text-[13px]" role="tree">
            {visibleRows.map(({ entry, depth }) => {
              const isExpanded = entry.isDir && expanded.has(entry.relPath);
              const isSelected = !entry.isDir && activePath === entry.relPath;
              const isCut = clipboard?.op === "cut" && clipboard.relPath === entry.relPath;
              const indent = 8 + depth * 12;
              const dirLoading = entry.isDir && isExpanded && loading.has(entry.relPath);

              return (
                <li key={entry.relPath} role="treeitem" aria-expanded={entry.isDir ? isExpanded : undefined}>
                  <button
                    type="button"
                    onClick={() => (entry.isDir ? toggleDir(entry.relPath) : onOpenFile(entry.relPath, entry.name))}
                    onContextMenu={(event) => openMenu(event, { relPath: entry.relPath, isDir: entry.isDir })}
                    title={entry.relPath}
                    className={[
                      "flex w-full items-center gap-1.5 py-0.5 pr-2 text-left transition-colors",
                      isCut ? "opacity-50" : "",
                      isSelected ? "bg-nest-accent/15 text-nest-foreground" : "text-nest-foreground/90 hover:bg-nest-muted/10",
                    ].join(" ")}
                    style={{ paddingLeft: `${indent}px` }}
                  >
                    {entry.isDir ? (
                      <FontAwesomeIcon icon={isExpanded ? faChevronDown : faChevronRight} className="size-2.5 shrink-0 text-nest-muted" />
                    ) : (
                      <span className="w-2.5 shrink-0" />
                    )}
                    <FontAwesomeIcon
                      icon={entry.isDir ? (isExpanded ? faFolderOpen : faFolder) : faFile}
                      className={["size-3.5 shrink-0", entry.isDir ? "text-nest-accent" : "text-nest-muted"].join(" ")}
                    />
                    <span className="truncate">{entry.name}</span>
                  </button>
                  {dirLoading ? (
                    <p className="py-0.5 pr-2 text-xs text-nest-muted" style={{ paddingLeft: `${indent + 20}px` }}>
                      Loading…
                    </p>
                  ) : null}
                  {dirErrors[entry.relPath] ? (
                    <p className="py-0.5 pr-2 text-xs text-nest-error" style={{ paddingLeft: `${indent + 20}px` }}>
                      {dirErrors[entry.relPath]}
                    </p>
                  ) : null}
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {menu ? <ContextMenu x={menu.x} y={menu.y} items={buildMenuItems(menu.target)} onClose={() => setMenu(null)} /> : null}

      {promptProps ? (
        <PromptDialog
          open
          title={promptProps.title}
          label={promptProps.label}
          description={promptProps.description}
          initialValue={promptProps.initialValue}
          placeholder={promptProps.placeholder}
          confirmLabel={promptProps.confirmLabel}
          onSubmit={submitPrompt}
          onCancel={() => setPrompt(null)}
        />
      ) : null}

      <ConfirmDialog
        open={confirmDelete !== null}
        title="Delete"
        confirmLabel="Delete"
        description={
          confirmDelete
            ? `Delete ${confirmDelete.isDir ? "folder" : "file"} "${baseName(confirmDelete.relPath)}"? ${
                confirmDelete.isDir ? "This removes all of its contents." : "This cannot be undone."
              }`
            : ""
        }
        onConfirm={() => (confirmDelete ? Promise.resolve(runDelete(confirmDelete)) : Promise.resolve())}
        onCancel={() => setConfirmDelete(null)}
      />
    </div>
  );
}
