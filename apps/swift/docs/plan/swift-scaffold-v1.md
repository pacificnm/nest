# Swift scaffold v1

## Status: Implemented

Phase **0** of [swift-v1](./swift-v1.md). Scaffold `apps/swift/` from [desktop template](../../../templates/desktop/).

## Context

Swift source lives at `apps/swift/` (gitignored). Specs and plans live in `apps/swift/docs/`. Path patches wire Nest crates from the monorepo layout.

## Specs

- [ui-platform](../specs/ui-platform.md)
- [overview](../specs/overview.md)

## Tasks

| Task | Done when |
|------|-----------|
| Copy `templates/desktop/` → `apps/swift/` | `ui/` + `src-tauri/` present |
| Rename app id | `swift` in `TauriApp::new`, `tauri.conf.json`, `package.json` |
| Add `.cargo/config.toml` | Path patches: `../../../core/crates/*`, `../../modules/crates/*` |
| Add `config.example.toml` | `[tauri]`, `[agent]`, `[ollama]` stubs |
| Wire modules in `main.rs` | `ThemeModule`, `nest-tauri` features `runtime`, `async` |
| Replace demo `App.tsx` | Minimal shell placeholder matching [ui-platform](../specs/ui-platform.md) wireframe |
| Document run steps | `apps/swift/README.md` (local only, gitignored) |
| Smoke test | `npm run build` in `ui/`; `cargo check` in `src-tauri/` (may skip link on CI without dbus) |

## Crate layout (target)

```text
apps/swift/
├── ui/
├── src-tauri/
│   └── src/main.rs
└── crates/              # phase 1+
    └── swift-data/
```

## Config sketch

```toml
[tauri]
title = "Swift"
width = 1400
height = 900

[ollama]
base_url = "http://127.0.0.1:11434"
default_model = "qwen2.5-coder:3b"

[agent]
max_steps = 8
allow_writes = false
```

## Done when

- Empty Swift app window opens with themed placeholder shell
- Nest path patches resolve from `apps/swift/`
- [swift-v1](./swift-v1.md) phase 0 checked off

## Related

- [swift-data-v1](./swift-data-v1.md) — next phase
- [Desktop template](../../../templates/desktop/)
