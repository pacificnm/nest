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
- [Font Awesome](https://fontawesome.com) icons (via [`Icon`](ui/src/components/Icon.tsx) + [`lib/fontawesome.ts`](ui/src/lib/fontawesome.ts))
- Default **`cbre-light`** theme (Nest framework default) via `nest-react-theme` CSS variables
- Shared desktop shell (see below)

## Shared shell components

Product-agnostic UI promoted from the Swift app. Import from the [`shell`](ui/src/shell/index.ts) barrel:

```tsx
import { AppShell, Ribbon, RibbonGroup, RibbonButton, ConfirmDialog, DatePicker, useToast, useStatusBar } from "./shell";
```

| Component / API | File | Notes |
|-----------------|------|-------|
| `AppShell` | [`components/AppShell.tsx`](ui/src/components/AppShell.tsx) | Ribbon slot + main + optional rail + status + toasts |
| `Ribbon`, `RibbonGroup`, `RibbonButton`, `RibbonButtonStack`, `RibbonMenuButton` | [`components/Ribbon.tsx`](ui/src/components/Ribbon.tsx) | Tabs are passed as `tabs` prop (not hardcoded) |
| `StatusBar` | [`components/StatusBar.tsx`](ui/src/components/StatusBar.tsx) | Slot-based (`left` / `right`); center shows live status |
| `ToastProvider` / `useToast` / `ToastViewport` | [`context/ToastContext.tsx`](ui/src/context/ToastContext.tsx), [`components/ToastViewport.tsx`](ui/src/components/ToastViewport.tsx) | success/info/warning/error |
| `StatusBarProvider` / `useStatusBar` | [`context/StatusBarContext.tsx`](ui/src/context/StatusBarContext.tsx) | Transient footer messages |
| `ConfirmDialog` | [`components/ConfirmDialog.tsx`](ui/src/components/ConfirmDialog.tsx) | Confirm/delete modal (Esc to cancel) |
| `DatePicker` | [`components/DatePicker.tsx`](ui/src/components/DatePicker.tsx) | Calendar popover; uses [`lib/date.ts`](ui/src/lib/date.ts) |
| `ErrorBoundary` | [`components/ErrorBoundary.tsx`](ui/src/components/ErrorBoundary.tsx) | Recovery screen for render errors |
| `Icon` | [`components/Icon.tsx`](ui/src/components/Icon.tsx) | Font Awesome wrapper |

Wrap the app in `ToastProvider` + `StatusBarProvider` (and `ErrorBoundary`) at the root — see [`main.tsx`](ui/src/main.tsx). `App.tsx` demos the full shell.

- Tailwind preset aligned with [`nest-react-theme`](../../core/crates/nest-react-theme)

## Quick start

From this directory (inside the Nest monorepo):

```bash
./build dev      # Tauri + Vite hot reload
./build run      # production build + launch
./build build    # production artifacts only
```

See [Nest build standard](../../docs/build.md) for the full command list (`test`, `check`, `clean`). Runtime layout: [app standard](../../docs/app-standard.md).

Legacy manual flow (optional):

```bash
cd ui && npm install && npm run dev
cd ../src-tauri && cargo run
```

## Copying to a product repo

1. Copy `ui/`, `src-tauri/`, and `build` into your app root.
2. Point `src-tauri/Cargo.toml` Nest dependencies at the monorepo via `.cargo/config.toml` path patches (see [apps/README.md](../../apps/README.md)), or use git dependencies on [pacificnm/nest](https://github.com/pacificnm/nest).
3. Rename the app id in `src-tauri/tauri.conf.json` and `TauriApp::new("…")` in `main.rs`.
4. Enable `nest-tauri` features: `runtime` + `images` when using `RemoteImage`.

## Related

- [nest-tauri docs](../../docs/nest-tauri/README.md)
- [nest-react-ui v1 plan](../../docs/plan/nest-react-ui-v1.md)
