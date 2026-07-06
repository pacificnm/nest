# Nest desktop app template

Starter layout for Nest **desktop** apps: **Tauri + React + TypeScript + Tailwind**.

```text
templates/desktop/
├── ui/                 # React front end (Vite)
└── src-tauri/          # Tauri shell + Nest modules
```

## Features

- [`nest-tauri`](../../core/crates/nest-tauri) bootstrap with `ThemeModule` + `ImageModule`
- Built-in IPC: `nest_app_metadata`, `nest_theme_css`, `nest_image_fetch`, `nest_image_invalidate_tag`
- [`RemoteImage`](ui/src/components/RemoteImage.tsx) React component (cached remote images via Rust)
- [Lucide React](https://lucide.dev) icons
- Tailwind preset aligned with [`nest-react-theme`](../../core/crates/nest-react-theme)

## Quick start

From this directory (inside the Nest monorepo):

```bash
# Front end
cd ui
npm install
npm run dev

# Shell (separate terminal — requires Tauri system deps on Linux)
cd ../src-tauri
cargo run
```

Or use the Tauri CLI from `ui/` after `npm install`:

```bash
cd ui
npm run tauri dev
```

## Copying to a product repo

1. Copy `ui/` and `src-tauri/` into your app root.
2. Point `src-tauri/Cargo.toml` Nest dependencies at the monorepo via `.cargo/config.toml` path patches (see [apps/README.md](../../apps/README.md)), or use git dependencies on [pacificnm/nest](https://github.com/pacificnm/nest).
3. Rename the app id in `src-tauri/tauri.conf.json` and `TauriApp::new("…")` in `main.rs`.
4. Enable `nest-tauri` features: `runtime` + `images` when using `RemoteImage`.

## Related

- [nest-tauri docs](../../docs/nest-tauri/README.md)
- [nest-react-ui v1 plan](../../docs/plan/nest-react-ui-v1.md)
