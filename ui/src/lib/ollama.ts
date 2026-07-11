import { invoke } from "@tauri-apps/api/core";

export type OllamaModel = { name: string; size: string | null; modified: string | null };
export type OllamaAuthStatus = { signedIn: boolean; detail: string };
export type AccountStatus = { signedIn: boolean; detail: string };

export async function listOllamaModels(host: string): Promise<OllamaModel[]> {
  return invoke<OllamaModel[]>("ollama_list_models", { host });
}

export async function ollamaAuthStatus(host: string): Promise<OllamaAuthStatus> {
  return invoke<OllamaAuthStatus>("ollama_auth_status", { host });
}

export async function ollamaSignIn(host: string): Promise<void> {
  return invoke("ollama_signin", { host });
}

export async function ollamaSignOut(host: string): Promise<void> {
  return invoke("ollama_signout", { host });
}

export async function codexAccountStatus(): Promise<AccountStatus> {
  return invoke<AccountStatus>("codex_account_status");
}

export async function codexLogin(): Promise<void> {
  return invoke("codex_login");
}

export async function codexLogout(): Promise<void> {
  return invoke("codex_logout");
}
