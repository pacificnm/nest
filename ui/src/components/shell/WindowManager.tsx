import { HelpApp } from "../apps/HelpApp";
import { ComponentsApp } from "../apps/ComponentsApp";
import { NestTerminal } from "../apps/NestTerminal";
import { NestAgent } from "../apps/NestAgent";
import { NestFiles } from "../apps/NestFiles";
import { SettingsApp } from "../apps/SettingsApp";
import { NestTheme } from "../apps/NestTheme";
import { ClaudeConfigApp } from "../apps/ClaudeConfigApp";
import { EmbeddedApp } from "../apps/EmbeddedApp";
import { PlaceholderApp } from "../apps/PlaceholderApp";
import { getShellApp, type ShellApp, type ShellWindow } from "../../shell/types";
import { WindowFrame } from "./WindowFrame";

// ComponentsApp now exports NestUIBrowser - the component viewer for Nest UI Components

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

  if (app.id === "components") {
    return <ComponentsApp />;
  }

  if (app.id === "terminal") {
    return <NestTerminal />;
  }

  if (app.id === "agent") {
    return <NestAgent />;
  }

  if (app.id === "files") {
    return <NestFiles />;
  }

  if (app.id === "settings") {
    return <SettingsApp />;
  }

  if (app.id === "theme") {
    return <NestTheme />;
  }

  if (app.id === "claude-config") {
    return <ClaudeConfigApp />;
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
