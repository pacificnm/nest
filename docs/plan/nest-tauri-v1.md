# nest-tauri v1 Implementation Plan

## Status: Scaffolded

Bootstrap, config merge, and built-in IPC commands are implemented. See [nest-tauri docs](../nest-tauri/README.md).

Canonical product layout and host rules: [app standard](../app-standard.md).

## Desktop frontend platform

Nest desktop apps use a fixed **frontend platform**:

| Layer | Technology | Location |
|-------|------------|----------|
| Shell | **Tauri** (Rust) | `src-tauri/` + `nest-tauri` |
| UI | **React** + **TypeScript** | `ui/src/` |
| Styling | **Tailwind CSS** | `ui/tailwind.config.ts` + `nest-react-theme` |
| Design tokens | `nest-design` / `nest-theme` | Rust + CSS variables in webview |

Rust Nest modules run in-process via Tauri commands. The React app never replaces the module layer — it is the presentation tier only.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-app` | `NestApp`, metadata, lifecycle orchestration (unchanged) |
| `nest-tauri` | `TauriApp`, bootstrap, Tauri commands, events, window lifecycle |
| `nest-react-theme` | `ThemeAdapter` → CSS custom properties + Tailwind preset from `ThemeDefinition` |
| `nest-theme` | `ThemeModule`, `ThemeService` (user registers explicitly) |
| `nest-design` | Token schema and built-in themes (unchanged) |
| `nest-config` | `ConfigLoader`, `ConfigService` |
| `nest-logging` | File-only default via `LoggingConfig::for_tauri` |

## Legacy crates (do not use for new work)

| Crate | Use instead |
|-------|-------------|
| `nest-gui` | `nest-tauri` + React `ui/` |
| `nest-icon` | React icon library in `ui/` — see [nest-react-ui v1](./nest-react-ui-v1.md) |
| `nest-image` (egui widget) | Keep `ImageService` in Rust; React `<RemoteImage>` + Tauri IPC — see [nest-react-ui v1](./nest-react-ui-v1.md) |

## Target runtime stack

```text
nest-core → nest-app → nest-cli / nest-tui / nest-tauri
                              │
                              └── ui/ (React + TypeScript + Tailwind) via Tauri webview
```

Modules (`nest-file`, `nest-ai`, `nest-http`, …) register into `AppContext` in `src-tauri/`; the React front end calls Tauri commands and listens for events.

App template: [`templates/desktop/`](../../templates/desktop/) — copy `ui/` + `src-tauri/` into a product repo.

## Bootstrap order

1. Parse startup options (Rust + Tauri CLI)
2. Load config (`nest-config`)
3. Initialize logging (file only — no stdout while window is active)
4. Build `AppContext` via `AppBuilder`
5. Register modules/services
6. Start Tauri runtime with command/event bridge
7. Shutdown on window close / app exit

**Precedence:** defaults < config file < CLI flags

## nest-tauri responsibilities (v1)

- Mirror `nest-cli` / `nest-tui` bootstrap: config, logging, `AppContext` build
- Expose `TauriApp::builder()` with `.module()`, `.command()`, `.run()`
- Bridge `NestError` / `NestErrorReport` to the front end (structured IPC)
- Optional `ThemeService` sync: push active theme to `ui/` on change (v1 may poll on startup only)
- Window options from `[tauri]` config section + CLI overrides

## nest-react-theme responsibilities (v1)

- Implement `ThemeAdapter<CssTheme>` from `nest-design::ThemeDefinition`
- Emit CSS custom properties (`--nest-color-background`, …)
- Ship Tailwind preset mapping semantic tokens to utilities
- Document consumption in app `tailwind.config.ts`

## Migration notes (Kiwi)

- Replace legacy Kiwi workbench UI with `ui/` + `src-tauri/`
- Reuse existing Nest modules (`nest-file`, `nest-github`, `nest-agent`, …) from Tauri commands

## Deferred

- [nest-react-ui v1](./nest-react-ui-v1.md) — replace `nest-icon` and `nest-image` egui layers
- `nest-cli-theme` adapter
- Theme change events pushed live to webview (`nest-events`)
- In-app log viewer panel
- Shared React component library (`@nest/ui` npm package)
- `nest-http-server` host

## Related

- [Desktop frontend platform](../architecture.md#desktop-frontend-platform)
- [nest-tauri README](../nest-tauri/README.md)
- [nest-react-ui v1 plan](./nest-react-ui-v1.md)
- [nest-react-theme README](../nest-react-theme/README.md)
- [nest-design + nest-theme plan](./nest-design-theme-v1.md)
