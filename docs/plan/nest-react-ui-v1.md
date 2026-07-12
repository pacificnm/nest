# nest-react-ui v1 — replacing nest-icon and nest-image

## Status: In progress (IPC + template done)

Companion to [nest-tauri v1](./nest-tauri-v1.md). Defines how desktop apps replace legacy egui presentation crates with **React + Tailwind** components while keeping Rust services where they belong.

## Problem

| Legacy crate | What it did | Why it does not fit Tauri + React |
|--------------|-------------|-----------------------------------|
| `nest-icon` | Font Awesome fonts + egui widgets | Icons are a webview concern; no Rust render loop |
| `nest-image` | HTTP fetch + disk cache **and** egui texture widget | Fetch/cache stays in Rust; display moves to React |

Both crates remain in the workspace temporarily for Kiwi migration. **New desktop apps must not depend on them.**

## Target split

```text
src-tauri/                    ui/
├── ImageService (nest-image)   ├── <Icon />     — React icon library
├── ImageModule                 ├── <RemoteImage /> — Tauri IPC + <img>
├── Tauri commands              └── Tailwind + nest-react-theme tokens
└── nest-tauri bootstrap
```

Presentation lives in `ui/`. Rust owns I/O, caching, and domain services.

---

## Replacing nest-icon

### Decision

**No Rust replacement crate.** Icons are rendered entirely in the React layer.

### Recommended stack

| Need | Library | Notes |
|------|---------|-------|
| General UI icons | [Lucide React](https://lucide.dev) | Tree-shakeable, matches modern desktop UX |
| Font Awesome parity | [@fortawesome/react-fontawesome](https://fontawesome.com/docs/web/use-with/react) | When migrating Kiwi views that used `nest-icon` constants |
| Sizing / color | Tailwind + `nest-react-theme` | e.g. `className="size-4 text-nest-muted-foreground"` |

### Migration checklist (per app)

1. Remove `IconModule` / `nest-icon` from `Cargo.toml`.
2. Add chosen icon package to `ui/package.json`.
3. Map common `nest_icon::Icon::*` usages to React components (maintain a small app-level alias table during Kiwi migration).
4. Drop Font Awesome `.ttf` embedding from the Rust binary.

### Future (deferred)

Shared npm package `@nest/ui` with `NestIcon` wrapper (token-aware default size/color). Not required for v1 — apps can use Lucide/FA directly.

---

## Replacing nest-image

### Decision

**Keep the Rust crate; replace only the egui widget layer.**

| Layer | Crate / path | Keep? |
|-------|--------------|-------|
| `ImageService` — HTTP fetch, `nest-cache` storage, tag invalidation | `nest-image` | **Yes** |
| `ImageModule`, cache keys, URL helpers | `nest-image` | **Yes** |
| `RemoteImage` egui widget, texture decode | `nest-image` (`egui` feature) | **Remove** (after migration) |
| React display + loading/error states | `ui/` or `@nest/ui` | **Add** |
| IPC bridge | `nest-tauri` commands | **Add** |

### Rust changes (v1)

1. **`nest-image`:** default features **empty** — `egui` widget behind optional feature only (legacy).
2. **`nest-tauri`:** optional built-in commands when `ImageService` is registered:

   | Command | Input | Output |
   |---------|-------|--------|
   | `nest_image_fetch` | `{ url, tags? }` | `{ bytes_base64, mime, cache_key }` |
   | `nest_image_invalidate_tag` | `{ tag }` | `{ removed_count }` |

   Uses existing `ImageService::fetch_bytes` and `invalidate_tag`.

3. **Mime detection:** infer from bytes (JPEG/PNG magic) or `Content-Type` when available.

### React changes (v1)

App template / shared hook pattern:

```tsx
// ui/src/components/RemoteImage.tsx (planned)
import { convertFileSrc, invoke } from "@tauri-apps/api/core";

type Props = { url: string; alt: string; className?: string; tags?: string[] };

export function RemoteImage({ url, alt, className, tags }: Props) {
  // invoke nest_image_fetch → data URL or asset path → <img />
}
```

Features to match legacy widget behavior:

- Placeholder while loading / on error
- Optional `rounded-*` via Tailwind (replaces egui corner radius)
- Cache tags for movie poster invalidation (pass through to command)

### Alternative: asset protocol (v2)

Register a Tauri asset protocol handler (`nest-image://{cache_key}`) so React uses normal `<img src="…">` without base64 over IPC. Deferred until v1 command path is proven.

---

## Implementation phases

| Phase | Work | Owner |
|-------|------|-------|
| **1 — Document** | This plan, legacy README stubs, `nest-image` default features | Done |
| **2 — IPC** | `nest_image_fetch` + `nest_image_invalidate_tag` in `nest-tauri` (`images` feature) | Done |
| **3 — React** | `RemoteImage` in [desktop template](../../templates/desktop/) + Lucide | Done |
| **4 — Kiwi** | Migrate workbench icons + artwork views | Blocked on GPU / shell migration |
| **5 — Remove legacy** | Drop `nest-icon` workspace member; remove `nest-image/egui` feature + widget module | After Kiwi off egui |

---

## Crate status summary

| Crate | Desktop status | Replacement |
|-------|----------------|-------------|
| `nest-icon` | **Deprecated** | React icon library in `ui/` |
| `nest-image` | **Partially kept** | `ImageService` + Tauri commands + React `<RemoteImage>` |
| `nest-gui` | **Deprecated** | `nest-tauri` + `ui/` |

---

## Related

- [Desktop frontend platform](../architecture.md#desktop-frontend-platform)
- [nest-tauri v1](./nest-tauri-v1.md)
- [nest-react-theme](../nest-react-theme/README.md)
