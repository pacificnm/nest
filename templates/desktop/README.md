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

Product-agnostic UI promoted from an app's desktop shell. Import from the [`shell`](ui/src/shell/index.ts) barrel:

```tsx
import { AppShell, Ribbon, RibbonGroup, RibbonButton, ConfirmDialog, DatePicker, useToast, useStatusBar } from "./shell";
```

| Component / API | File | Notes |
|-----------------|------|-------|
| `AppShell` | [`components/AppShell.tsx`](ui/src/components/AppShell.tsx) | Ribbon slot + main + optional left nav + optional right rail + status + toasts |
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

`ui/package.json`'s `build` script is `vite build` only (no `tsc -b`): a
standalone `tsc -b` type-checks `@nest/components`'s own source through the
symlink, and since it has its own separately-installed `node_modules` (see
[docs/build.md](../../docs/build.md)), that pulls in a second, structurally
identical but nominally distinct copy of `@types/react`, producing spurious
"two different types with this name exist" errors on ref callback types.
This isn't a real type error in your code — `vite build` (esbuild-based, no
cross-package type unification) builds fine. Run `npm run typecheck`
(`tsc -b`) manually if you want project-wide type-checking and are aware of
this limitation.

## Scaffolding a new app

From the Nest repo root:

```bash
scripts/scaffold-desktop-app.sh apps/<name> "Display Title"
```

This copies `ui/`, `src-tauri/`, `build`, `nest-app.toml`, and `.gitignore`
into `apps/<name>/`, and renames every template placeholder (Cargo package
name, Tauri bundle identifier, window title, UI package name, cache dir)
to match. Then:

```bash
cd apps/<name>
./build dev
```

Manual copy is still possible (copy `ui/`, `src-tauri/`, `build`, rename the
app id in `src-tauri/tauri.conf.json` / `TauriApp::new("…")` in `main.rs`,
point `src-tauri/Cargo.toml`'s Nest dependencies at the monorepo) but the
script above does all of that consistently in one step.

Enable `nest-tauri` features: `runtime` + `images` when using `RemoteImage`.

## Related

- [nest-tauri docs](../../docs/nest-tauri/README.md)
- [nest-react-ui v1 plan](../../docs/plan/nest-react-ui-v1.md)
