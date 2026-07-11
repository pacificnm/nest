/**
 * Nest Files - IDE-style file explorer + editor for Nest Desktop
 *
 * A left-sidebar file/folder tree (like Kiwi's Explorer panel) plus a
 * tab-driven Monaco editor in the main area. Ported from Kiwi's workbench
 * Explorer + EditorArea, trimmed to just files (no git/issues/tasks tabs).
 */

import { useCallback, useEffect, useState } from "react";
import { filesInfo, filesReadText, filesWriteText, formatIpcError, type FilesRoot } from "../../lib/files";
import { FileTree } from "./files/FileTree";
import { EditorTabs } from "./files/EditorTabs";
import type { EditorTab } from "./files/types";

export function NestFiles() {
  const [root, setRoot] = useState<FilesRoot | null>(null);
  const [tabs, setTabs] = useState<EditorTab[]>([]);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    filesInfo()
      .then((info) => {
        if (!cancelled) {
          setRoot(info);
        }
      })
      .catch((infoError) => {
        if (!cancelled) {
          setError(formatIpcError(infoError));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const openFile = useCallback((relPath: string, name: string) => {
    setActivePath(relPath);
    setTabs((current) => {
      if (current.some((tab) => tab.relPath === relPath)) {
        return current;
      }
      return [...current, { relPath, name, content: "", loading: true, error: null, dirty: false }];
    });
    void filesReadText(relPath)
      .then((file) => {
        setTabs((current) =>
          current.map((tab) =>
            tab.relPath === relPath ? { ...tab, content: file.content, loading: false, error: null } : tab,
          ),
        );
      })
      .catch((readError) => {
        setTabs((current) =>
          current.map((tab) =>
            tab.relPath === relPath ? { ...tab, loading: false, error: formatIpcError(readError) } : tab,
          ),
        );
      });
  }, []);

  const focusTab = useCallback((relPath: string) => setActivePath(relPath), []);

  const closeTab = useCallback((relPath: string) => {
    setTabs((current) => {
      const next = current.filter((tab) => tab.relPath !== relPath);
      setActivePath((current_) => {
        if (current_ !== relPath) {
          return current_;
        }
        return next.length > 0 ? next[next.length - 1].relPath : null;
      });
      return next;
    });
  }, []);

  const changeContent = useCallback((relPath: string, content: string) => {
    setTabs((current) =>
      current.map((tab) => (tab.relPath === relPath ? { ...tab, content, dirty: true } : tab)),
    );
  }, []);

  const saveTab = useCallback((relPath: string) => {
    setTabs((current) => {
      const tab = current.find((entry) => entry.relPath === relPath);
      if (!tab || !tab.dirty) {
        return current;
      }
      void filesWriteText(relPath, tab.content)
        .then(() => {
          setTabs((latest) =>
            latest.map((entry) => (entry.relPath === relPath ? { ...entry, dirty: false } : entry)),
          );
        })
        .catch((saveError) => setError(formatIpcError(saveError)));
      return current;
    });
  }, []);

  const handleFileRenamed = useCallback((from: string, to: string, name: string) => {
    setTabs((current) =>
      current.map((tab) => (tab.relPath === from ? { ...tab, relPath: to, name } : tab)),
    );
    setActivePath((current) => (current === from ? to : current));
  }, []);

  const handlePathDeleted = useCallback((relPath: string) => {
    setTabs((current) => {
      const next = current.filter((tab) => tab.relPath !== relPath && !tab.relPath.startsWith(`${relPath}/`));
      setActivePath((current_) => {
        if (current_ && !next.some((tab) => tab.relPath === current_)) {
          return next.length > 0 ? next[next.length - 1].relPath : null;
        }
        return current_;
      });
      return next;
    });
  }, []);

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background text-nest-foreground">
      {error ? (
        <div className="shrink-0 border-b border-nest-error/30 bg-nest-error/10 px-3 py-1.5 text-xs text-nest-error">
          {error}
        </div>
      ) : null}
      <div className="flex min-h-0 flex-1">
        <FileTree
          root={root}
          activePath={activePath}
          onOpenFile={openFile}
          onFileRenamed={handleFileRenamed}
          onPathDeleted={handlePathDeleted}
          onError={setError}
        />
        <div className="min-w-0 flex-1">
          <EditorTabs
            tabs={tabs}
            activePath={activePath}
            onFocusTab={focusTab}
            onCloseTab={closeTab}
            onChangeContent={changeContent}
            onSaveTab={saveTab}
          />
        </div>
      </div>
    </div>
  );
}
