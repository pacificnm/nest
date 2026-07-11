export type ShellAppKind = "builtin" | "registered";

export type ShellApp = {
  id: string;
  name: string;
  icon: string;
  category: string;
  windowTitle: string;
  description: string;
  kind: ShellAppKind;
  path?: string;
};

export type ShellWindow = {
  id: string;
  appId: string;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
  minimized: boolean;
  zIndex: number;
  embedUrl?: string;
  launchMessage?: string;
};

export const HELP_APP: ShellApp = {
  id: "help",
  name: "Help",
  icon: "fa-solid fa-circle-question",
  category: "System",
  windowTitle: "Nest Help",
  description: "Browse Nest framework documentation.",
  kind: "builtin",
};

export const COMPONENTS_APP: ShellApp = {
  id: "components",
  name: "Component Library",
  icon: "fa-solid fa-layer-group",
  category: "Development",
  windowTitle: "Nest Components",
  description: "Browse and test @nest/components UI library.",
  kind: "builtin",
};

export const TERMINAL_APP: ShellApp = {
  id: "terminal",
  name: "Nest Terminal",
  icon: "fa-solid fa-terminal",
  category: "Development",
  windowTitle: "Nest Terminal",
  description: "Integrated terminal emulator for Nest Desktop.",
  kind: "builtin",
};

export const AGENT_APP: ShellApp = {
  id: "agent",
  name: "Nest Agent",
  icon: "fa-solid fa-robot",
  category: "Development",
  windowTitle: "Nest Agent",
  description: "Embedded AI coding agent for Nest Desktop.",
  kind: "builtin",
};

export const FILES_APP: ShellApp = {
  id: "files",
  name: "Nest Files",
  icon: "fa-solid fa-folder-open",
  category: "System",
  windowTitle: "Nest Files",
  description: "Browse and manage files in the Nest repository.",
  kind: "builtin",
};

export const SETTINGS_APP: ShellApp = {
  id: "settings",
  name: "Settings",
  icon: "fa-solid fa-gear",
  category: "System",
  windowTitle: "Nest Settings",
  description: "Configure Nest Desktop preferences.",
  kind: "builtin",
};

export const THEME_APP: ShellApp = {
  id: "theme",
  name: "Nest Theme",
  icon: "fa-solid fa-palette",
  category: "Development",
  windowTitle: "Nest Theme",
  description: "Browse built-in Nest theme tokens: colors, fonts, spacing, radius.",
  kind: "builtin",
};

export const CLAUDE_CONFIG_APP: ShellApp = {
  id: "claude-config",
  name: "Claude Config",
  icon: "fa-solid fa-wand-magic-sparkles",
  category: "Development",
  windowTitle: "Claude Config",
  description: "Configure your Claude API key, and view Skills and Agents.",
  kind: "builtin",
};

export function getShellApp(apps: ShellApp[], appId: string): ShellApp | undefined {
  return apps.find((app) => app.id === appId);
}
