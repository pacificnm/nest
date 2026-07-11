import { useEffect, useState } from "react";

import { appsList, registeredToShellApp } from "../lib/apps";
import {
  AGENT_APP,
  CLAUDE_CONFIG_APP,
  COMPONENTS_APP,
  FILES_APP,
  HELP_APP,
  SETTINGS_APP,
  TERMINAL_APP,
  THEME_APP,
  type ShellApp,
} from "./types";

export function useShellApps() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [registeredApps, setRegisteredApps] = useState<ShellApp[]>([]);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const registered = await appsList();
        if (cancelled) {
          return;
        }
        setRegisteredApps(registered.map(registeredToShellApp));
        setError(null);
      } catch (loadError) {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : String(loadError));
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  const apps: ShellApp[] = [
    HELP_APP,
    COMPONENTS_APP,
    TERMINAL_APP,
    AGENT_APP,
    FILES_APP,
    SETTINGS_APP,
    THEME_APP,
    CLAUDE_CONFIG_APP,
    ...registeredApps,
  ];

  return { apps, loading, error };
}
