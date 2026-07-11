import { invoke } from "@tauri-apps/api/core";

/**
 * Mirrors nest-design's `ThemeDefinition` serde shape exactly. `ThemeId` and
 * `ColorToken` are `#[serde(transparent)]` in Rust, so they arrive as plain
 * strings; other fields stay snake_case since nest-design doesn't rename
 * them for JSON.
 */
export type ThemeMode = "light" | "dark";

export type TypographyStyle = {
  font_family: string;
  size: number;
  line_height: number;
  weight: number;
};

export type ThemeDefinition = {
  id: string;
  mode: ThemeMode;
  colors: {
    background: string;
    foreground: string;
    primary: string;
    secondary: string;
    border: string;
    surface: string;
    accent?: string;
    muted?: string;
  };
  spacing: {
    xs: number;
    sm: number;
    md: number;
    lg: number;
    xl: number;
    xxl?: number;
  };
  radius: {
    sm: number;
    md: number;
    lg: number;
    full?: number;
  };
  typography: {
    body: TypographyStyle;
    heading: TypographyStyle;
    caption?: TypographyStyle;
    mono?: TypographyStyle;
  };
  status: {
    success: string;
    warning: string;
    error: string;
    info: string;
  };
};

/** Lists every built-in Nest theme with full token data. */
export async function themesList(): Promise<ThemeDefinition[]> {
  return invoke<ThemeDefinition[]>("themes_list");
}

/** Human-readable label derived from a theme id (`"cursor-dark"` -> `"Cursor Dark"`). */
export function themeLabel(id: string): string {
  return id
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}
