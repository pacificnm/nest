import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** External agent runtimes selectable in the Nest Agent window (`ollama launch`). */
export type AgentRuntime =
  | "claude"
  | "codex"
  | "codex-app"
  | "opencode"
  | "openclaw"
  | "hermes"
  | "copilot"
  | "droid"
  | "kimi"
  | "qwen"
  | "cline";

export const AGENT_RUNTIMES: { id: AgentRuntime; label: string }[] = [
  { id: "claude", label: "Claude Code" },
  { id: "codex", label: "Codex" },
  { id: "codex-app", label: "Codex App" },
  { id: "opencode", label: "OpenCode" },
  { id: "openclaw", label: "OpenClaw" },
  { id: "hermes", label: "Hermes Agent" },
  { id: "copilot", label: "Copilot CLI" },
  { id: "droid", label: "Droid" },
  { id: "kimi", label: "Kimi Code" },
  { id: "qwen", label: "Qwen Code" },
  { id: "cline", label: "Cline" },
];

export type AgentOutput = { base64: string };
export type AgentExit = { message: string };

/** Shape of an error rejected from a Tauri command (string or Error). */
export function formatIpcError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

const OUTPUT_EVENT = "nest://agent-output";
const EXIT_EVENT = "nest://agent-exit";

/** Default Ollama inference server. */
export const DEFAULT_OLLAMA_HOST = "192.168.88.10:11434";

export type LaunchArgs = {
  runtime: AgentRuntime;
  model: string;
  ollamaHost?: string | null;
  cwd?: string | null;
  /** When true, run the CLI directly with its own account (no Ollama). */
  direct?: boolean;
  rows: number;
  cols: number;
};

export async function launchAgent(args: LaunchArgs): Promise<void> {
  const cwd = args.cwd?.trim();
  return invoke("agent_launch", {
    runtime: args.runtime,
    model: args.model,
    ollamaHost: args.ollamaHost ?? null,
    cwd: cwd && cwd.length > 0 ? cwd : null,
    direct: args.direct ?? false,
    rows: args.rows,
    cols: args.cols,
  });
}

/** Keystrokes — high-frequency IPC. */
export async function sendAgentInput(data: string): Promise<void> {
  return invoke("agent_input", { data });
}

/** PTY resize — high-frequency IPC. */
export async function resizeAgent(rows: number, cols: number): Promise<void> {
  return invoke("agent_resize", { rows, cols });
}

export async function stopAgent(): Promise<void> {
  return invoke("agent_stop");
}

export async function agentStatus(): Promise<boolean> {
  return invoke<boolean>("agent_status");
}

/** Subscribes to agent PTY output; decodes base64 to raw bytes. */
export async function onAgentOutput(
  handler: (bytes: Uint8Array) => void,
): Promise<UnlistenFn> {
  return listen<AgentOutput>(OUTPUT_EVENT, (event) => {
    handler(decodeBase64(event.payload.base64));
  });
}

/** Subscribes to the agent exit event. */
export async function onAgentExit(
  handler: (message: string) => void,
): Promise<UnlistenFn> {
  return listen<AgentExit>(EXIT_EVENT, (event) => {
    handler(event.payload.message);
  });
}

function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
