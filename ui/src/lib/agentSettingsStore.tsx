import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { formatIpcError } from "./agent";
import {
  codexAccountStatus,
  codexLogin,
  codexLogout,
  listOllamaModels,
  ollamaAuthStatus,
  ollamaSignIn,
  ollamaSignOut,
  type AccountStatus,
  type OllamaAuthStatus,
  type OllamaModel,
} from "./ollama";

/** Connection mode for launching agents. */
export type AgentConnection = "ollama" | "account";

/** Agent settings persisted in Nest Desktop's `config.toml`, shared by the Settings and Agent apps. */
export type AgentSettings = {
  host: string;
  port: number;
  model: string;
  models: string[];
  runtime: string;
  connection: AgentConnection;
};

type AgentSettingsContextValue = {
  settings: AgentSettings;
  /** Models returned by the last `ollama list` refresh. */
  remoteModels: OllamaModel[];
  auth: OllamaAuthStatus | null;
  /** Codex native-account status (direct mode). */
  codexAccount: AccountStatus | null;
  loadingModels: boolean;
  saving: boolean;
  /** Most recent status/error message, for the Settings UI to surface. */
  message: { text: string; kind: "info" | "success" | "error" } | null;
  /** `host:port` for `OLLAMA_HOST` / `ollama list`. */
  ollamaHost: string;
  updateSettings: (patch: Partial<AgentSettings>) => void;
  setModel: (model: string) => void;
  addModel: (name: string) => void;
  removeModel: (name: string) => void;
  refreshModels: () => Promise<void>;
  refreshAuth: () => Promise<void>;
  refreshCodexAccount: () => Promise<void>;
  saveSettings: () => Promise<void>;
  signIn: () => Promise<void>;
  signOut: () => Promise<void>;
  codexSignIn: () => Promise<void>;
  codexSignOut: () => Promise<void>;
};

const AgentSettingsContext = createContext<AgentSettingsContextValue | null>(null);

const DEFAULT_SETTINGS: AgentSettings = {
  host: "192.168.88.10",
  port: 11434,
  model: "qwen3.5:2b",
  models: ["qwen3.5:2b"],
  runtime: "claude",
  connection: "ollama",
};

export function AgentSettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AgentSettings>(DEFAULT_SETTINGS);
  const [remoteModels, setRemoteModels] = useState<OllamaModel[]>([]);
  const [auth, setAuth] = useState<OllamaAuthStatus | null>(null);
  const [codexAccount, setCodexAccount] = useState<AccountStatus | null>(null);
  const [loadingModels, setLoadingModels] = useState(false);
  const [saving, setSaving] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [message, setMessage] = useState<AgentSettingsContextValue["message"]>(null);

  const ollamaHost = useMemo(() => `${settings.host.trim()}:${settings.port}`, [settings.host, settings.port]);

  useEffect(() => {
    void invoke<AgentSettings>("agent_settings_get")
      .then((value) => {
        setSettings(value);
        setLoaded(true);
      })
      .catch((error) => setMessage({ text: formatIpcError(error), kind: "error" }));
  }, []);

  const refreshAuth = useCallback(async () => {
    setAuth(await ollamaAuthStatus(ollamaHost));
  }, [ollamaHost]);

  useEffect(() => {
    void refreshAuth();
  }, [refreshAuth]);

  const refreshCodexAccount = useCallback(async () => {
    try {
      setCodexAccount(await codexAccountStatus());
    } catch {
      // Codex CLI not installed — leave status unset, UI shows "Checking…".
    }
  }, []);

  useEffect(() => {
    void refreshCodexAccount();
  }, [refreshCodexAccount]);

  const refreshModels = useCallback(async () => {
    setLoadingModels(true);
    try {
      const models = await listOllamaModels(ollamaHost);
      setRemoteModels(models);
      // Merge remote names into the saved list without dropping custom entries.
      setSettings((current) => {
        const merged = [...current.models];
        for (const remote of models) {
          if (!merged.includes(remote.name)) {
            merged.push(remote.name);
          }
        }
        return { ...current, models: merged };
      });
    } catch (error) {
      setMessage({ text: formatIpcError(error), kind: "error" });
    } finally {
      setLoadingModels(false);
    }
  }, [ollamaHost]);

  // Auto-refresh models once settings load from disk.
  useEffect(() => {
    if (loaded) {
      void refreshModels();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loaded]);

  const updateSettings = useCallback((patch: Partial<AgentSettings>) => {
    setSettings((current) => ({ ...current, ...patch }));
  }, []);

  const setModel = useCallback((model: string) => {
    setSettings((current) => {
      const models = current.models.includes(model) ? current.models : [model, ...current.models];
      return { ...current, model, models };
    });
  }, []);

  const addModel = useCallback((name: string) => {
    const trimmed = name.trim();
    if (!trimmed) {
      return;
    }
    setSettings((current) => {
      if (current.models.includes(trimmed)) {
        return current;
      }
      return { ...current, models: [...current.models, trimmed] };
    });
  }, []);

  const removeModel = useCallback((name: string) => {
    setSettings((current) => {
      const models = current.models.filter((item) => item !== name);
      const model = current.model === name ? (models[0] ?? current.model) : current.model;
      return { ...current, models, model };
    });
  }, []);

  const saveSettings = useCallback(async () => {
    setSaving(true);
    try {
      const saved = await invoke<AgentSettings>("agent_settings_save", { settings });
      setSettings(saved);
      setMessage({ text: `Saved agent settings (${saved.model})`, kind: "success" });
    } catch (error) {
      setMessage({ text: formatIpcError(error), kind: "error" });
    } finally {
      setSaving(false);
    }
  }, [settings]);

  const signIn = useCallback(async () => {
    try {
      await ollamaSignIn(ollamaHost);
      const immediate = await ollamaAuthStatus(ollamaHost);
      setAuth(immediate);
      if (immediate.signedIn) {
        setMessage({ text: immediate.detail || "Signed in to ollama.com", kind: "success" });
        return;
      }
      setMessage({ text: "Complete sign-in in your browser, then click Refresh", kind: "info" });
      for (let attempt = 0; attempt < 10; attempt += 1) {
        await new Promise((resolve) => window.setTimeout(resolve, 3000));
        const status = await ollamaAuthStatus(ollamaHost);
        setAuth(status);
        if (status.signedIn) {
          setMessage({ text: status.detail || "Signed in to ollama.com", kind: "success" });
          return;
        }
      }
    } catch (error) {
      setMessage({ text: formatIpcError(error), kind: "error" });
    }
  }, [ollamaHost]);

  const signOut = useCallback(async () => {
    try {
      await ollamaSignOut(ollamaHost);
      await refreshAuth();
      setMessage({ text: "Signed out of ollama.com", kind: "success" });
    } catch (error) {
      setMessage({ text: formatIpcError(error), kind: "error" });
    }
  }, [ollamaHost, refreshAuth]);

  const codexSignIn = useCallback(async () => {
    try {
      await codexLogin();
      setMessage({ text: "Complete Codex sign-in in your browser, then click Refresh", kind: "info" });
      window.setTimeout(() => void refreshCodexAccount(), 3000);
    } catch (error) {
      setMessage({ text: formatIpcError(error), kind: "error" });
    }
  }, [refreshCodexAccount]);

  const codexSignOut = useCallback(async () => {
    try {
      await codexLogout();
      await refreshCodexAccount();
      setMessage({ text: "Signed out of Codex", kind: "success" });
    } catch (error) {
      setMessage({ text: formatIpcError(error), kind: "error" });
    }
  }, [refreshCodexAccount]);

  const value = useMemo<AgentSettingsContextValue>(
    () => ({
      settings,
      remoteModels,
      auth,
      codexAccount,
      loadingModels,
      saving,
      message,
      ollamaHost,
      updateSettings,
      setModel,
      addModel,
      removeModel,
      refreshModels,
      refreshAuth,
      refreshCodexAccount,
      saveSettings,
      signIn,
      signOut,
      codexSignIn,
      codexSignOut,
    }),
    [
      settings,
      remoteModels,
      auth,
      codexAccount,
      loadingModels,
      saving,
      message,
      ollamaHost,
      updateSettings,
      setModel,
      addModel,
      removeModel,
      refreshModels,
      refreshAuth,
      refreshCodexAccount,
      saveSettings,
      signIn,
      signOut,
      codexSignIn,
      codexSignOut,
    ],
  );

  return <AgentSettingsContext.Provider value={value}>{children}</AgentSettingsContext.Provider>;
}

export function useAgentSettings(): AgentSettingsContextValue {
  const value = useContext(AgentSettingsContext);
  if (!value) {
    throw new Error("useAgentSettings must be used within AgentSettingsProvider");
  }
  return value;
}
