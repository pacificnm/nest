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

export function getShellApp(apps: ShellApp[], appId: string): ShellApp | undefined {
  return apps.find((app) => app.id === appId);
}
