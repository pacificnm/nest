/**
 * Nest Agent - Embedded AI coding agent for Nest Desktop
 *
 * Launches an external coding agent (Claude Code, Codex, OpenCode, …) in a
 * PTY, either routed through Ollama or run directly with its own account.
 * Ported from Kiwi's Agent Panel. Runtime/model/connection are configured in
 * Settings > Agent (shared via AgentSettingsProvider) — this window is
 * launch-only, matching Kiwi's simplified AgentPanel.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { ClipboardAddon } from "@xterm/addon-clipboard";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPlay, faXmark } from "@fortawesome/free-solid-svg-icons";
import {
  formatIpcError,
  launchAgent,
  onAgentExit,
  onAgentOutput,
  resizeAgent,
  sendAgentInput,
  stopAgent,
  type AgentRuntime,
} from "../../lib/agent";
import { useAgentSettings } from "../../lib/agentSettingsStore";

export function NestAgent() {
  const { settings, ollamaHost } = useAgentSettings();
  const runtime = settings.runtime as AgentRuntime;
  const model = settings.model;
  const direct = settings.connection === "account";
  const [running, setRunning] = useState(false);
  const [statusMessage, setStatusMessage] = useState(
    "Configure the runtime + model in Settings > Agent, then press Launch.",
  );

  const hostRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const clipboardRef = useRef<ClipboardAddon | null>(null);
  const unlistenRef = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }
    const term = new Terminal({
      fontFamily: "JetBrains Mono, Consolas, monospace",
      fontSize: 13,
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
      void sendAgentInput(data).catch(() => {});
    });

    clipboardRef.current = new ClipboardAddon();
    term.loadAddon(clipboardRef.current);

    let rafId = 0;
    const applyFit = () => {
      fit.fit();
      void resizeAgent(term.rows, term.cols).catch(() => {});
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
      if (rafId) {
        window.cancelAnimationFrame(rafId);
      }
      observer.disconnect();
      unlistenRef.current.forEach((off) => off());
      unlistenRef.current = [];
      void stopAgent().catch(() => {});
      clipboardRef.current?.dispose();
      clipboardRef.current = null;
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, []);

  const handleLaunch = useCallback(async () => {
    const term = termRef.current;
    const fit = fitRef.current;
    if (!term || !fit) {
      return;
    }
    fit.fit();
    term.focus();

    unlistenRef.current.forEach((off) => off());
    unlistenRef.current = [];

    try {
      const offOutput = await onAgentOutput((bytes) => term.write(bytes));
      const offExit = await onAgentExit((message) => {
        setStatusMessage(message);
        setRunning(false);
      });
      unlistenRef.current = [offOutput, offExit];

      setStatusMessage(direct ? `${runtime} · account` : `${runtime} @ ${ollamaHost} · ${model}`);
      await launchAgent({
        runtime,
        model,
        ollamaHost,
        direct,
        rows: term.rows,
        cols: term.cols,
      });
      setRunning(true);
    } catch (error) {
      const message = formatIpcError(error);
      term.writeln(`\r\n\x1b[31mLaunch failed: ${message}\x1b[0m`);
      setStatusMessage(`Launch failed: ${message}`);
      setRunning(false);
    }
  }, [runtime, model, ollamaHost, direct]);

  const handleStop = useCallback(async () => {
    await stopAgent().catch(() => {});
    setRunning(false);
    setStatusMessage("Agent stopped.");
  }, []);

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex shrink-0 items-center gap-2 border-b border-nest-border px-2 py-1.5">
        <div className="min-w-0 flex-1">
          <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">Agent</span>
          <p
            className="truncate font-mono text-[11px] text-nest-muted"
            title={direct ? `${runtime} · account` : `${runtime} · ${model} · ${ollamaHost}`}
          >
            {running ? statusMessage : direct ? `${runtime} · account` : `${runtime} · ${model} · ${ollamaHost}`}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-1.5">
          {running ? (
            <button
              type="button"
              onClick={() => void handleStop()}
              title="Stop agent"
              className="inline-flex h-6 items-center gap-1 rounded-nest-sm border border-nest-border px-2 text-xs text-nest-error hover:bg-nest-error/10"
            >
              <FontAwesomeIcon icon={faXmark} className="size-3" />
              Stop
            </button>
          ) : (
            <button
              type="button"
              onClick={() => void handleLaunch()}
              title="Launch agent"
              className="inline-flex h-6 items-center gap-1 rounded-nest-sm bg-nest-primary px-2 text-xs font-medium text-white hover:opacity-90"
            >
              <FontAwesomeIcon icon={faPlay} className="size-3" />
              Launch
            </button>
          )}
        </div>
      </header>
      <div className="min-h-0 flex-1 overflow-hidden p-1">
        <div ref={hostRef} className="h-full w-full" />
      </div>
    </div>
  );
}
