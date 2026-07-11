import { invoke } from "@tauri-apps/api/core";

/**
 * Types mirror `nest-claude`'s own response shapes exactly (snake_case field
 * names, same as `nest-design`'s `ThemeDefinition` in `lib/themes.ts`) —
 * `claude.rs` returns `nest_claude::Skill`/`Agent` directly rather than a
 * re-mapped camelCase view type, so this file must NOT rename fields.
 */

export type SkillSource = "custom" | "anthropic";

export type Skill = {
  id: string;
  created_at: string;
  updated_at: string;
  display_title: string;
  latest_version: string;
  source: SkillSource;
};

export type Speed = "standard" | "fast";

export type AgentModelInfo = {
  id: string;
  speed?: Speed | null;
};

export type AgentSkillRef = {
  type: "anthropic" | "custom";
  skill_id: string;
  version?: string | null;
};

/** Loosely typed — the viewer only needs to label each tool, not configure it. */
export type AgentTool = {
  type: string;
  mcp_server_name?: string;
  name?: string;
  [key: string]: unknown;
};

export type Agent = {
  id: string;
  version: number;
  name: string;
  description?: string | null;
  system?: string | null;
  created_at: string;
  updated_at: string;
  archived_at?: string | null;
  model: AgentModelInfo;
  tools: AgentTool[];
  skills: AgentSkillRef[];
  mcp_servers: unknown[];
  metadata: Record<string, string>;
  multiagent?: unknown;
};

/** Claude API key, persisted in Nest Desktop's `config.toml` (`[claude]` section). */
export type ClaudeSettings = {
  apiKey: string;
};

/** Lists every Skill (both `custom` and Anthropic-provided) on the account. */
export async function claudeListSkills(): Promise<Skill[]> {
  return invoke<Skill[]>("claude_list_skills");
}

/** Lists every non-archived Agent on the account, including assigned skills. */
export async function claudeListAgents(): Promise<Agent[]> {
  return invoke<Agent[]>("claude_list_agents");
}

export async function claudeSettingsGet(): Promise<ClaudeSettings> {
  return invoke<ClaudeSettings>("claude_settings_get");
}

export async function claudeSettingsSave(settings: ClaudeSettings): Promise<ClaudeSettings> {
  return invoke<ClaudeSettings>("claude_settings_save", { settings });
}

/** Labels an `AgentTool` for display (e.g. "Full toolset", "MCP: github", "Custom: run_tests"). */
export function agentToolLabel(tool: AgentTool): string {
  if (tool.type === "agent_toolset_20260401") {
    return "Full toolset (bash, read, write, edit, glob, grep, web)";
  }
  if (tool.type === "mcp_toolset") {
    return `MCP: ${tool.mcp_server_name ?? "unknown server"}`;
  }
  if (tool.type === "custom") {
    return `Custom: ${tool.name ?? "unnamed"}`;
  }
  return tool.type;
}
