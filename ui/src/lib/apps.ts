import { invoke } from "@tauri-apps/api/core";

import type { ShellApp } from "../shell/types";

export type RegisteredApp = {
  id: string;
  name: string;
  category: string;
  icon: string;
  description: string;
  path: string;
};

export type LaunchMode = "module" | "embed" | "spawn";

export type LaunchTarget = {
  appId: string;
  mode: LaunchMode;
  url?: string;
  devPort?: number;
  program?: string;
  args?: string[];
  message?: string;
};

export async function appsList(): Promise<RegisteredApp[]> {
  return invoke<RegisteredApp[]>("apps_list");
}

export async function appsResolveLaunch(appId: string): Promise<LaunchTarget> {
  return invoke<LaunchTarget>("apps_resolve_launch", { appId });
}

export async function appsSpawn(
  program: string,
  args: string[] = [],
  cwd?: string,
): Promise<number> {
  return invoke<number>("apps_spawn", { program, args, cwd: cwd ?? null });
}

export async function appsLaunchKiwi(): Promise<number> {
  return invoke<number>("apps_launch_kiwi");
}

export function registeredToShellApp(app: RegisteredApp): ShellApp {
  return {
    id: app.id,
    name: app.name,
    icon: app.icon,
    category: app.category,
    windowTitle: app.name,
    description: app.description,
    kind: "registered",
    path: app.path,
  };
}
