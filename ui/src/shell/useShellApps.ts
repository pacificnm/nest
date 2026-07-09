import { useEffect, useState } from "react";

import { appsList, registeredToShellApp } from "../lib/apps";
import { HELP_APP, type ShellApp } from "./types";

export function useShellApps() {
  const [apps, setApps] = useState<ShellApp[]>([HELP_APP]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const registered = await appsList();
        if (cancelled) {
          return;
        }
        setApps([HELP_APP, ...registered.map(registeredToShellApp)]);
        setError(null);
      } catch (loadError) {
        if (!cancelled) {
          setError(loadError instanceof Error ? loadError.message : String(loadError));
          setApps([HELP_APP]);
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

  return { apps, loading, error };
}
