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

export async function appsList(): Promise<RegisteredApp[]> {
  return invoke<RegisteredApp[]>("apps_list");
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
