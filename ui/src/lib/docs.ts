import { invoke } from "@tauri-apps/api/core";

export type DocEntry = {
  path: string;
  name: string;
  depth: number;
};

export async function docsList(): Promise<DocEntry[]> {
  return invoke<DocEntry[]>("docs_list");
}

export async function docsRead(path: string): Promise<string> {
  return invoke<string>("docs_read", { relPath: path });
}
