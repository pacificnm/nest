import { invoke } from "@tauri-apps/api/core";

export type FilesRoot = { root: string; name: string };

export type FileEntry = {
  name: string;
  relPath: string;
  isDir: boolean;
  size: number;
  /** Milliseconds since Unix epoch, when available. */
  modified: number | null;
};

export type FileContent = { relPath: string; content: string };

/** Shape of an error rejected from a Tauri command (string or Error). */
export function formatIpcError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

export async function filesInfo(): Promise<FilesRoot> {
  return invoke<FilesRoot>("files_info");
}

/** Lists a directory relative to the repository root (`"."` for the root). */
export async function filesList(rel: string): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("files_list", { rel });
}

/** Reads a UTF-8 text file relative to the repository root, for the editor. */
export async function filesReadText(rel: string): Promise<FileContent> {
  return invoke<FileContent>("files_read_text", { rel });
}

/** Writes UTF-8 `content` to a file relative to the repository root. */
export async function filesWriteText(rel: string, content: string): Promise<string> {
  return invoke<string>("files_write_text", { rel, content });
}

export async function filesCreateFile(rel: string): Promise<string> {
  return invoke<string>("files_create_file", { rel });
}

export async function filesCreateDir(rel: string): Promise<string> {
  return invoke<string>("files_create_dir", { rel });
}

export async function filesRename(from: string, to: string): Promise<string> {
  return invoke<string>("files_rename", { from, to });
}

export async function filesDelete(rel: string): Promise<string> {
  return invoke<string>("files_delete", { rel });
}

export async function filesCopy(from: string, to: string): Promise<string> {
  return invoke<string>("files_copy", { from, to });
}

export async function filesReveal(rel: string): Promise<void> {
  return invoke("files_reveal", { rel });
}

/** Joins a repo-relative directory + child name into a relative path. */
export function joinRel(dir: string, name: string): string {
  return dir === "." || dir === "" ? name : `${dir}/${name}`;
}

/** Returns the parent of a repo-relative path (`"."` at the root). */
export function parentRel(rel: string): string {
  const idx = rel.lastIndexOf("/");
  return idx === -1 ? "." : rel.slice(0, idx);
}

