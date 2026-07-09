import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import { Desktop } from "./components/Desktop";
import { StartMenu } from "./components/StartMenu";
import { Taskbar } from "./components/Taskbar";
import { WindowManager } from "./components/shell/WindowManager";
import { useShellApps } from "./shell/useShellApps";
import { useWindowManager } from "./shell/useWindowManager";

export function App() {
  const [startMenuOpen, setStartMenuOpen] = useState(false);
  const { apps, loading, error } = useShellApps();
  const {
    windows,
    focusedWindowId,
    runningApps,
    openApp,
    focusWindow,
    closeWindow,
    minimizeWindow,
    moveWindow,
    resizeWindow,
  } = useWindowManager(apps);

  const toggleStartMenu = () => {
    setStartMenuOpen((open) => !open);
  };

  const closeStartMenu = () => {
    setStartMenuOpen(false);
  };

  const launchApp = (appId: string) => {
    openApp(appId);
    closeStartMenu();
  };

  const handleTaskbarApp = (appId: string) => {
    const existing = windows.find((window) => window.appId === appId);
    if (existing) {
      focusWindow(existing.id);
      return;
    }
    openApp(appId);
  };

  const handleExit = async () => {
    try {
      await invoke("exit_app");
    } catch {
      window.close();
    }
  };

  return (
    <div className="relative h-screen w-screen overflow-hidden bg-nest-background">
      {error ? (
        <div className="absolute right-4 top-4 z-[60] rounded-nest-md border border-nest-error bg-nest-surface px-3 py-2 text-xs text-nest-error">
          App registry: {error}
        </div>
      ) : null}
      {loading ? (
        <div className="pointer-events-none absolute inset-0 bottom-12 flex items-center justify-center text-sm text-nest-muted">
          Loading apps…
        </div>
      ) : null}
      <Desktop apps={apps} onLaunchApp={launchApp} />
      <WindowManager
        apps={apps}
        windows={windows}
        focusedWindowId={focusedWindowId}
        onFocus={focusWindow}
        onClose={closeWindow}
        onMinimize={minimizeWindow}
        onMove={moveWindow}
        onResize={resizeWindow}
      />
      <StartMenu
        apps={apps}
        isOpen={startMenuOpen}
        onClose={closeStartMenu}
        onLaunchApp={launchApp}
        onExit={handleExit}
      />
      <Taskbar
        apps={apps}
        runningApps={runningApps}
        focusedApp={
          windows.find((window) => window.id === focusedWindowId)?.appId ?? null
        }
        onStartToggle={toggleStartMenu}
        onLaunchApp={handleTaskbarApp}
      />
    </div>
  );
}
