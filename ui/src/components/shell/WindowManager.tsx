import { HelpApp } from "../apps/HelpApp";
import { EmbeddedApp } from "../apps/EmbeddedApp";
import { PlaceholderApp } from "../apps/PlaceholderApp";
import { getShellApp, type ShellApp, type ShellWindow } from "../../shell/types";
import { WindowFrame } from "./WindowFrame";

type WindowManagerProps = {
  apps: ShellApp[];
  windows: ShellWindow[];
  focusedWindowId: string | null;
  onFocus: (windowId: string) => void;
  onClose: (windowId: string) => void;
  onMinimize: (windowId: string) => void;
  onMove: (windowId: string, x: number, y: number) => void;
  onResize: (windowId: string, width: number, height: number) => void;
};

function renderWindowContent(window: ShellWindow, app: ShellApp | undefined) {
  if (!app) {
    return null;
  }

  if (app.id === "help") {
    return <HelpApp />;
  }

  if (window.embedUrl || window.launchMessage) {
    return (
      <EmbeddedApp
        title={app.name}
        url={window.embedUrl}
        message={window.launchMessage}
      />
    );
  }

  return (
    <PlaceholderApp
      title={app.name}
      description={app.description}
      path={app.path}
    />
  );
}

export function WindowManager({
  apps,
  windows,
  focusedWindowId,
  onFocus,
  onClose,
  onMinimize,
  onMove,
  onResize,
}: WindowManagerProps) {
  return (
    <>
      {windows.map((window) => (
        <WindowFrame
          key={window.id}
          window={window}
          focused={window.id === focusedWindowId}
          onFocus={() => onFocus(window.id)}
          onClose={() => onClose(window.id)}
          onMinimize={() => onMinimize(window.id)}
          onMove={(x, y) => onMove(window.id, x, y)}
          onResize={(width, height) => onResize(window.id, width, height)}
        >
          {renderWindowContent(window, getShellApp(apps, window.appId))}
        </WindowFrame>
      ))}
    </>
  );
}
