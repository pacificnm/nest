import { useCallback, useMemo, useState } from "react";

import { clampWindowPosition, clampWindowSize } from "./windowBounds";
import type { LaunchTarget } from "../lib/apps";
import type { ShellApp, ShellWindow } from "./types";

const DEFAULT_SIZE = { width: 960, height: 640 };

let nextWindowId = 1;
let nextZIndex = 10;

function createWindow(app: ShellApp, launch?: LaunchTarget): ShellWindow {
  const offset = (nextWindowId - 1) * 28;
  return {
    id: `win-${nextWindowId++}`,
    appId: app.id,
    title: app.windowTitle,
    x: 120 + offset,
    y: 72 + offset,
    width: DEFAULT_SIZE.width,
    height: DEFAULT_SIZE.height,
    minimized: false,
    zIndex: nextZIndex++,
    embedUrl: launch?.mode === "embed" ? launch.url : undefined,
    launchMessage: launch?.message,
  };
}

export function useWindowManager(apps: ShellApp[]) {
  const appById = useMemo(() => new Map(apps.map((app) => [app.id, app])), [apps]);
  const [windows, setWindows] = useState<ShellWindow[]>([]);
  const [focusedWindowId, setFocusedWindowId] = useState<string | null>(null);

  const focusWindow = useCallback((windowId: string) => {
    setWindows((current) =>
      current.map((window) =>
        window.id === windowId
          ? { ...window, zIndex: nextZIndex++, minimized: false }
          : window,
      ),
    );
    setFocusedWindowId(windowId);
  }, []);

  const openApp = useCallback(
    (appId: string, launch?: LaunchTarget) => {
      const app = appById.get(appId);
      if (!app) {
        return;
      }

      setWindows((current) => {
        const existing = current.find((window) => window.appId === appId);
        if (existing) {
          setFocusedWindowId(existing.id);
          return current.map((window) =>
            window.id === existing.id
              ? {
                  ...window,
                  zIndex: nextZIndex++,
                  minimized: false,
                  embedUrl:
                    launch?.mode === "embed" ? launch.url ?? window.embedUrl : window.embedUrl,
                  launchMessage: launch?.message ?? window.launchMessage,
                }
              : window,
          );
        }

        const created = createWindow(app, launch);
        setFocusedWindowId(created.id);
        return [...current, created];
      });
    },
    [appById],
  );

  const closeWindow = useCallback((windowId: string) => {
    setWindows((current) => current.filter((window) => window.id !== windowId));
    setFocusedWindowId((current) => (current === windowId ? null : current));
  }, []);

  const minimizeWindow = useCallback((windowId: string) => {
    setWindows((current) =>
      current.map((window) =>
        window.id === windowId ? { ...window, minimized: true } : window,
      ),
    );
    setFocusedWindowId((current) => (current === windowId ? null : current));
  }, []);

  const moveWindow = useCallback((windowId: string, x: number, y: number) => {
    setWindows((current) =>
      current.map((window) => {
        if (window.id !== windowId) {
          return window;
        }
        const position = clampWindowPosition(x, y, window.width, window.height);
        return { ...window, ...position };
      }),
    );
  }, []);

  const resizeWindow = useCallback(
    (windowId: string, width: number, height: number) => {
      setWindows((current) =>
        current.map((window) => {
          if (window.id !== windowId) {
            return window;
          }
          const bounds = clampWindowSize(window.x, window.y, width, height);
          return { ...window, ...bounds };
        }),
      );
    },
    [],
  );

  const runningApps = Array.from(new Set(windows.map((window) => window.appId)));

  return {
    windows,
    focusedWindowId,
    runningApps,
    openApp,
    focusWindow,
    closeWindow,
    minimizeWindow,
    moveWindow,
    resizeWindow,
  };
}
