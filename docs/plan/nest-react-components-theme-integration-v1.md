# nest-react-components — Theme Integration (v1)

## Goal

Make `@nest/components` a **pure consumer** of Nest theme CSS variables. Delete the
independent palette the port introduced (`src/styles.css`) and route all themable
values through the existing pipeline:

```
nest-design (Rust)          semantic token values + built-in themes
  cbre_light | light | dark | cursor_dark      (cbre-light = framework default)
      │
nest-react-theme (Rust)     CssTheme::from_definition() → :root{ --nest-color-*, --nest-spacing-*, --nest-radius-* }
      │                     tailwind_preset_json()       → nest-* utilities = var(--nest-*)
      │
Tauri host  nest_theme_css  → { id, mode, variables, root_block }
      │
React app   applyThemeRootBlock() → injects <style id="nest-theme-vars"> at runtime
```

The library uses Tailwind utilities (`bg-nest-primary`, `rounded-nest-md`) that resolve
to `var(--nest-*)`; **the host supplies the values**, and theme switching is the host
swapping the `:root` block (it supports all four themes, not just light/dark).

## Decisions (confirmed 2026-07-12)

- **No library fallbacks.** The library ships **zero** palette. Values come from the host
  (`nest_theme_css`) or the consuming app's own `index.css` (as `templates/desktop/ui`
  already does). Vitest asserts classes, not computed values, so tests need no palette.
- **ThemeProvider stays only as a host-mode reflector.** Remove all color coupling
  (`[data-theme='dark']` value block). `ThemeProvider` keeps a light/dark UI-mode hint but
  never defines colors. Kept exported so consumers don't break.

## What's wrong today (`core/crates/nest-react-components/src/styles.css`)

1. **Divergent independent palette** — hardcodes `--nest-color-primary: #2563eb` (blue),
   `#7c3aed` accent, etc. Real cbre-light default is `#003f2d` green / `#69be28` accent.
   The library defines a *different look* than the framework.
2. **Competing theming mechanism** — `[data-theme='dark']` value block + `ThemeProvider`
   is a second theme system that fights the host's runtime var-block injection (the host
   does not use `[data-theme=dark]` value selectors).
3. **Invented CSS vars** — `--nest-font-body`, `--nest-font-size-body`,
   `--nest-line-height-body` are referenced as fallbacks but `nest-react-theme` emits no
   font variables. Dead contract.
4. **Duplicated spacing/radius values** owned by nest-design — will drift.
5. **Never imported** — `styles.css` is only exposed as the `@nest/components/styles.css`
   subpath and nothing imports it. So the one legitimately component-owned bit — the
   `@keyframes linear-progress-indeterminate1/2` that `LinearProgress` depends on — never
   loads, and the indeterminate bar animation is silently dead in real apps.

The components themselves are otherwise well-behaved: ~1600 `nest-*` token usages via the
preset, only one stray raw color (in an `Avatar.docs.md` example).

## Plan

### Step 1 — Delete the independent palette
- Delete `core/crates/nest-react-components/src/styles.css`.
- Remove the `"./styles.css": "./src/styles.css"` entry from `package.json` `exports`.
- The invented `--nest-font-*` fallbacks die with the file (fonts come from Tailwind
  `fontFamily: Inter` in `tailwind.config.ts`, applied globally by the host `body`).

### Step 2 — Preserve LinearProgress motion as component-owned, theme-independent CSS
The two keyframes are real and required, but are **motion**, not a themable value, and
must survive the deletion. Do **not** rely on Tailwind emitting them from the preset —
Tailwind only emits keyframes for `animate-*` classes it *sees*, and no consuming app's
Tailwind `content` glob currently scans the component source (see Adjacent Finding A).

- Create `core/crates/nest-react-components/src/runtime.css` containing **only** the two
  `@keyframes linear-progress-indeterminate1/2` (no `:root`, no colors, no vars).
- Import it once so it is always delivered: `import './runtime.css';` at the top of
  `src/index.ts` (processed by the consumer's bundler; ignored by vitest's css handling).
- Keep `LinearProgress.tsx` as-is (it already references the keyframe names via
  `animate-[linear-progress-indeterminate1_…]`).
- Verify both bars now animate (they do not today).

### Step 3 — ThemeProvider: remove color coupling, keep as mode reflector
- With `styles.css` gone, `ThemeProvider`'s `data-theme` attribute no longer swaps any
  colors. Update its JSDoc to state that **theming (colors) is host-driven via
  `nest_theme_css`**; `ThemeProvider` only tracks a light/dark UI-mode hint.
- Keep it exported. (Follow-up, out of scope: have it read `mode` from `nest_theme_css`
  instead of localStorage so the hint reflects the actual host theme.)

### Step 4 — Confirm the host/app owns app-shell CSS
- `*:focus-visible`, scrollbar styling, and `body` typography from the old file are
  host/app-shell concerns. `templates/desktop/ui/src/index.css` already owns `body` +
  the cbre-light `:root` fallbacks. Components already do per-element `focus:ring-*` per
  the port contract, so a global `*:focus-visible` is optional. **No library action** —
  just verify the template still looks right without the library file.

### Step 5 — Verify
- `cd core/crates/nest-react-components && npm run build` → tsc + vite clean.
- `npx vitest run` → green (LinearProgress, Button, Alert especially).
- `cargo test -p nest-react-theme` → preset JSON + css-var tests still pass.
- Drive it (desktop template or Nest Desktop): (a) components take colors from the host
  theme — switch the theme and colors change; (b) LinearProgress indeterminate animates;
  (c) no residual blue `#2563eb` palette anywhere.

## Adjacent findings (discovered during review — accept or defer)

**A. Consuming apps don't scan component source in Tailwind `content`.**
Every app/template `tailwind.config.ts` has `content: ["./index.html", "./src/**/*.{ts,tsx}"]`
— it does **not** include `@nest/components` source. Since the package is consumed as
source (`main: ./src/index.ts`), the components' `nest-*` utility classes are only emitted
if the app happens to author the same classes. This affects **all** component styling, not
just the keyframes. Fix: add the components glob to each consumer, e.g.
`"../../core/crates/nest-react-components/src/**/*.{ts,tsx}"` (path adjusted per app), or
publish a prebuilt CSS. Tracked separately from theming but blocks correct rendering.

**B. Package-root barrel is incomplete.**
`src/index.ts` re-exports only `inputs`, `feedback`, `navigation`. `surface`, `layout`,
`data-display` (Typography, Box, Card, Chip, Avatar, Badge, List, Table, …) are
unreachable from `@nest/components`. Fix: `export * from './components';` (which already
re-exports all six folders) or add the three missing lines. Not a theming issue, but the
ported components can't be imported without it.

**C. Preset duplication (7 copies).**
`nest_react_theme::tailwind_preset_json()` (source of truth) is hand-copied into 7
`nest-tailwind-preset.json` files (`ui/`, `templates/desktop/ui/`, `core/crates/nest-react-components/`,
and `apps/{finch,kiwi,airtable-sync}/ui/`). Any preset change must touch all of
them. Consider generating the JSON from the Rust source in a build step. Out of scope
here; noted so the keyframe/token work stays in sync if the preset is ever extended.

---

*Depends on `docs/plan/nest-react-components-port-v1.md` (the port that introduced
`styles.css`). See `docs/nest-react-theme/README.md` and `docs/architecture.md` for the
desktop frontend platform.*
