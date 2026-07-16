# Desktop UI workflow

When building or modifying desktop UI (`ui/`, `src-tauri/`, Tauri commands),
search indexed API docs **before** guessing at Tauri, React, or Tailwind
behavior:

**`search_knowledge_base`** (`nest-knowledge` MCP) with collections `tauri`,
`react`, `tailwind`.

Run at least one search per session before writing new UI or IPC code, and
re-search when moving to a new API area (e.g. from Tauri command setup to a
Tauri event stream).

When editing `apps/loon/client/` specifically, use collection `webos-tv`
instead — see [.cursor/rules/webos-tv-knowledge.mdc](../../.cursor/rules/webos-tv-knowledge.mdc).

## IPC boundary rules

```text
ui/src/          invoke("scan_library", { … })
       ───────── Tauri IPC (serialize args / results) ─────────
src-tauri/       #[tauri::command] fn scan_library(ctx: State<…>, …)
       ───────── in-process ─────────
crates/core/     pub async fn scan_library(svc: &LibraryService, …) -> NestResult<…>
```

- **Do** keep `#[tauri::command]` handlers thin: validate input, resolve
  services from `AppContext`, delegate to `crates/core`.
- **Do** use structured errors (`NestError` / `NestErrorReport`) bridged to
  the webview.
- **Do not** put business rules in `ui/` beyond form validation and display
  logic.
- **Do not** use Tauri IPC between CLI/TUI and core — they already run
  in-process and call services directly.

React is presentation only: it calls Tauri commands and listens for events;
it does not replace Nest modules or duplicate domain logic.

See [docs/app-standard.md](../../docs/app-standard.md) (IPC boundary
section) and [tauri-apps.md](tauri-apps.md) for the broader desktop stack.
