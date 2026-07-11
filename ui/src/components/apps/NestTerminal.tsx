/**
 * Nest Terminal - Integrated terminal emulator for Nest Desktop
 *
 * Reusable terminal component that can be used as:
 * - A standalone app in Nest Desktop
 * - Embedded in other applications
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ClipboardAddon } from "@xterm/addon-clipboard";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faPlus,
  faTerminal,
  faTrash,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";

const OUTPUT_EVENT = "nest://terminal-output";
const EXIT_EVENT = "nest://terminal-exit";

type TerminalSession = { id: string; title: string };

/** Decode base64 to Uint8Array */
function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/** Open a terminal session */
async function openTerminal(args: {
  id: string;
  cwd?: string | null;
  shell?: string | null;
  rows: number;
  cols: number;
}): Promise<void> {
  const cwd = args.cwd?.trim();
  await invoke("terminal_open", {
    id: args.id,
    cwd: cwd && cwd.length > 0 ? cwd : null,
    shell: args.shell ?? null,
    rows: args.rows,
    cols: args.cols,
  });
}

/** Send input to terminal */
async function sendTerminalInput(id: string, data: string): Promise<void> {
  await invoke("terminal_input", { id, data });
}

/** Resize terminal */
async function resizeTerminal(id: string, rows: number, cols: number): Promise<void> {
  await invoke("terminal_resize", { id, rows, cols });
}

/** Close terminal */
async function closeTerminal(id: string): Promise<void> {
  await invoke("terminal_close", { id });
}

/** Listen to terminal output */
async function onTerminalOutput(
  handler: (id: string, bytes: Uint8Array) => void,
): Promise<UnlistenFn> {
  return listen<{ id: string; base64: string }>(OUTPUT_EVENT, (event: { payload: { id: string; base64: string } }) => {
    handler(event.payload.id, decodeBase64(event.payload.base64));
  });
}

/** Listen to terminal exit */
async function onTerminalExit(
  handler: (id: string, message: string) => void,
): Promise<UnlistenFn> {
  return listen<{ id: string; message: string }>(EXIT_EVENT, (event: { payload: { id: string; message: string } }) => {
    handler(event.payload.id, event.payload.message);
  });
}

/** Terminal instance component */
function TerminalInstance({
  id,
  cwd,
  active,
  onExit,
}: {
  id: string;
  cwd: string;
  active: boolean;
  onExit: (id: string) => void;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const clipboardRef = useRef<ClipboardAddon | null>(null);
  const exitedRef = useRef(false);

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }
    const term = new Terminal({
      fontFamily: "JetBrains Mono, Consolas, monospace",
      fontSize: 14,
      cursorBlink: true,
      theme: {
        background: "#1b1f23",
        foreground: "#cccccc",
        cursor: "#cccccc",
        black: "#1b1f23",
        red: "#f85149",
        green: "#3fb950",
        yellow: "#d29922",
        blue: "#58a6ff",
        magenta: "#bc8cff",
        cyan: "#76e3ea",
        white: "#b1bac4",
        brightBlack: "#6e7681",
        brightRed: "#ff7b72",
        brightGreen: "#56d364",
        brightYellow: "#e3b341",
        brightBlue: "#79c0ff",
        brightMagenta: "#d2a8ff",
        brightCyan: "#b3f0ff",
        brightWhite: "#f0f6fc",
      },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    const host = hostRef.current;
    term.open(host);
    termRef.current = term;
    fitRef.current = fit;

    term.onData((data) => {
      void sendTerminalInput(id, data).catch(() => {});
    });

    // Simple clipboard handling
    clipboardRef.current = new ClipboardAddon();
    term.loadAddon(clipboardRef.current);

    const unlisten: UnlistenFn[] = [];
    let disposed = false;

    void onTerminalOutput((outId, bytes) => {
      if (outId === id) {
        term.write(bytes);
      }
    }).then((off) => (disposed ? off() : unlisten.push(off)));

    void onTerminalExit((exitId, message) => {
      if (exitId === id) {
        exitedRef.current = true;
        term.writeln(`\r\n\x1b[90m${message}\x1b[0m`);
        onExit(id);
      }
    }).then((off) => (disposed ? off() : unlisten.push(off)));

    let rafId = 0;
    let opened = false;
    const openPty = () => {
      if (opened) {
        return;
      }
      opened = true;
      void openTerminal({ id, cwd, rows: term.rows, cols: term.cols }).catch((error) => {
        term.writeln(`\r\n\x1b[31mFailed to start shell: ${String(error)}\x1b[0m`);
      });
    };
    const applyFit = () => {
      fit.fit();
      if (!opened) {
        openPty();
      } else {
        void resizeTerminal(id, term.rows, term.cols).catch(() => {});
      }
    };
    const observer = new ResizeObserver(() => {
      if (rafId) {
        return;
      }
      rafId = window.requestAnimationFrame(() => {
        rafId = 0;
        applyFit();
      });
    });
    observer.observe(host);

    rafId = window.requestAnimationFrame(() => {
      rafId = 0;
      applyFit();
    });

    return () => {
      disposed = true;
      if (rafId) {
        window.cancelAnimationFrame(rafId);
      }
      observer.disconnect();
      unlisten.forEach((off) => off());
      if (!exitedRef.current) {
        void closeTerminal(id).catch(() => {});
      }
      clipboardRef.current?.dispose();
      clipboardRef.current = null;
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, [id, cwd, onExit]);

  useEffect(() => {
    if (!active) {
      return;
    }
    const raf = window.requestAnimationFrame(() => {
      const term = termRef.current;
      const fit = fitRef.current;
      if (term && fit) {
        fit.fit();
        void resizeTerminal(id, term.rows, term.cols).catch(() => {});
      }
      term?.focus();
    });
    return () => window.cancelAnimationFrame(raf);
  }, [active, id]);

  return (
    <div
      className={active ? "absolute inset-0 p-1" : "absolute inset-0 hidden"}
      onClick={() => termRef.current?.focus()}
      role="presentation"
    >
      <div ref={hostRef} className="h-full w-full" />
    </div>
  );
}

/** Main Nest Terminal app component */
export function NestTerminal() {
  const [sessions, setSessions] = useState<TerminalSession[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const counterRef = useRef(0);
  const autoSpawnedRef = useRef(false);

  const newTerminal = useCallback(() => {
    counterRef.current += 1;
    const id =
      typeof crypto !== "undefined" && "randomUUID" in crypto
        ? crypto.randomUUID()
        : `term-${Date.now()}-${counterRef.current}`;
    const title = `Terminal ${counterRef.current}`;
    setSessions((current) => [...current, { id, title }]);
    setActiveId(id);
  }, []);

  const removeTerminal = useCallback((id: string) => {
    setSessions((current) => {
      const next = current.filter((session) => session.id !== id);
      setActiveId((active) => {
        if (active !== id) {
          return active;
        }
        return next.length > 0 ? next[next.length - 1].id : null;
      });
      return next;
    });
  }, []);

  const killActive = useCallback(() => {
    if (activeId) {
      removeTerminal(activeId);
    }
  }, [activeId, removeTerminal]);

  useEffect(() => {
    if (autoSpawnedRef.current) {
      return;
    }
    autoSpawnedRef.current = true;
    newTerminal();
  }, [newTerminal]);

  const showList = sessions.length > 1;

  const content = useMemo(() => {
    if (sessions.length === 0) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-2 text-xs text-nest-muted">
          <FontAwesomeIcon icon={faTerminal} className="size-5" />
          <p>No open terminals.</p>
          <button
            type="button"
            onClick={newTerminal}
            className="inline-flex items-center gap-1 rounded-nest-sm bg-nest-primary px-2 py-1 text-xs font-medium text-white hover:opacity-90"
          >
            <FontAwesomeIcon icon={faPlus} className="size-3" />
            New Terminal
          </button>
        </div>
      );
    }
    return sessions.map((session) => (
      <TerminalInstance
        key={session.id}
        id={session.id}
        cwd=""
        active={session.id === activeId}
        onExit={removeTerminal}
      />
    ));
  }, [sessions, activeId, newTerminal, removeTerminal]);

  return (
    <div className="flex h-full min-h-0 bg-nest-background">
      <div className="relative min-h-0 flex-1">{content}</div>

      <div className="flex w-8 shrink-0 flex-col items-center gap-0.5 border-l border-nest-border bg-nest-surface py-1">
        <button
          type="button"
          onClick={newTerminal}
          title="New Terminal"
          aria-label="New Terminal"
          className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
        >
          <FontAwesomeIcon icon={faPlus} className="size-3" />
        </button>
        <button
          type="button"
          onClick={killActive}
          disabled={!activeId}
          title="Kill Terminal"
          aria-label="Kill Terminal"
          className="flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-error/10 hover:text-nest-error disabled:opacity-40"
        >
          <FontAwesomeIcon icon={faTrash} className="size-3" />
        </button>
      </div>

      {showList ? (
        <ul className="flex w-40 shrink-0 flex-col border-l border-nest-border bg-nest-surface py-1 text-xs">
          {sessions.map((session) => (
            <li key={session.id}>
              <div
                className={[
                  "group flex items-center gap-1.5 px-2 py-1",
                  session.id === activeId
                    ? "bg-nest-accent/15 text-nest-foreground"
                    : "text-nest-muted hover:bg-nest-muted/10",
                ].join(" ")}
              >
                <button
                  type="button"
                  onClick={() => setActiveId(session.id)}
                  className="flex min-w-0 flex-1 items-center gap-1.5 text-left"
                >
                  <FontAwesomeIcon icon={faTerminal} className="size-3 shrink-0" />
                  <span className="truncate">{session.title}</span>
                </button>
                <button
                  type="button"
                  onClick={() => removeTerminal(session.id)}
                  title="Close"
                  aria-label={`Close ${session.title}`}
                  className="shrink-0 rounded-nest-sm p-0.5 text-nest-muted opacity-0 hover:bg-nest-muted/20 hover:text-nest-foreground group-hover:opacity-100"
                >
                  <FontAwesomeIcon icon={faXmark} className="size-2.5" />
                </button>
              </div>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
