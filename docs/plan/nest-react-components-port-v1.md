# nest-react-components — MUI Port Execution Plan (v1)

> **Executor:** this plan is written to be run by a low-cost model (qwen3.5) one
> component at a time. Every entry is a full recipe. **Do not improvise.** If a
> recipe is ambiguous, stop and leave a `// TODO(port): question` comment rather
> than guessing.

**Goal:** Port the remaining Material UI (`mui-material/src`) component surface into
`core/crates/nest-react-components` (`@nest/components`), keeping MUI-compatible prop
names but restyling with Nest design tokens + Tailwind. This makes future app
migration (Kiwi, Swift, airtable-sync) mechanical.

**Reference (MUI source):** https://github.com/mui/material-ui/tree/master/packages/mui-material/src
Use MUI only for **prop names and behavior**, never for styling (MUI uses Emotion;
we use Tailwind + tokens).

**Already built (do not re-port):** `Button`, `IconButton`, `TextField` (inputs);
`Dialog`, `Alert`, `Snackbar` (feedback); `AppBar`, `Menu` (navigation);
`ThemeProvider`, `cn()`, `useControllableState`.

---

## Part A — The Contract (every component MUST follow this)

Read this once. It is the invariant shape of every file you create. `Button.tsx`
is the canonical reference implementation — open it before writing anything.

### A1. File layout

Each component ships **four** files. Three live in its category folder under source;
the usage doc lives under the repo `docs/` tree so the knowledge base and the Nest
Desktop Help app pick it up:

```
src/components/<category>/<Name>.tsx        # implementation (required)
src/components/<category>/<Name>.test.tsx    # vitest + RTL (required)
src/components/<category>/<Name>.demo.tsx    # gallery demo (required)
docs/nest-react-components/<category>/<Name>.md   # usage doc (required) — NOT colocated
```

Categories (existing folders — do not invent new ones without a recipe saying so):
`inputs/`, `feedback/`, `navigation/`, `surface/`, `data-display/`, `layout/`.

**Docs location:** usage docs are **not** colocated with source (an earlier pass put
`<Name>.docs.md` next to the component; those were relocated). Write the doc to
`docs/nest-react-components/<category>/<Name>.md` and add a link to it under the right
category heading in `docs/nest-react-components/README.md`.

Compound components (e.g. `Card` + `CardHeader` + `CardContent`) all go in **one**
`Card.tsx` file and are all exported from it.

### A2. Implementation rules

1. **Imports:** `import { cn } from '../../lib/cn';` (adjust `../../` to folder depth).
   Icons come from `lucide-react` only. Never import from `@mui/*`.
2. **Ref forwarding:** use `forwardRef` with a named function, exactly like `Button`:
   ```tsx
   export const Name = forwardRef<HTMLXElement, NameProps>(function Name(
     { className, variant = '...', ...props }, ref) { ... });
   ```
3. **Props interface:** `export interface NameProps extends XHTMLAttributes<HTMLXElement> {...}`.
   Every prop gets a JSDoc block with `@default` where applicable. Prop **names and
   union values must match MUI** (e.g. `variant`, `color`, `size`, `severity`).
4. **Style maps:** put variant/size/color class strings in top-level `const` records
   typed `Record<Union, string>` (see `VARIANT_STYLES`, `SIZE_STYLES` in `Button.tsx`).
   Never build class strings with template literals inside JSX — compose with `cn(...)`.
5. **className passthrough:** always accept `className` and pass it **last** into `cn(...)`
   so callers can override.
6. **Only Nest tokens** for color/spacing/radius (see Part B). Never hardcode hex,
   never use raw Tailwind palette colors (`bg-blue-500` is forbidden; use `bg-nest-primary`).
7. **Controlled/uncontrolled:** any stateful input (checked, open, value, expanded)
   uses `useControllableState` from `../../hooks`. Support both `value`/`onChange`
   (controlled) and `defaultValue` (uncontrolled), MUI-style.
8. **Accessibility:** correct semantic element or `role`, `aria-*` wired to state,
   keyboard support per the WAI-ARIA pattern. Each recipe lists the required a11y.

### A3. Barrel exports (do this every time — the build fails silently otherwise)

After creating a component, add to the category barrel
`src/components/<category>/index.ts`:
```ts
export { Name } from './Name';
export type { NameProps, NameVariant, NameSize } from './Name';   // whatever types exist
```
The category folder must be re-exported from `src/components/index.ts` (already true
for existing folders; if you create a component in an as-yet-unexported folder, add
`export * from './<category>';` to `src/components/index.ts` **and** confirm
`src/index.ts` re-exports it).

### A4. Demo file (`<Name>.demo.tsx`)

Export `function <Name>Demos()` returning `<div className="space-y-8 p-6">` with one
`<section>` per prop dimension (variants, sizes, colors, states, interactive). Copy the
structure of `Button.demo.tsx` exactly, including the section heading class
`"mb-4 text-lg font-semibold text-nest-foreground"`.

### A5. Docs file (`docs/nest-react-components/<category>/<Name>.md`)

Headings in this order: `# Name`, one-line intro, `## When to Use`, `## Variants`
(table, if any), `## Props` (table: Prop / Type / Default / Description), `## Examples`
(fenced `tsx` blocks importing from `@nest/components`), `## Accessibility` (bullet list).
Copy `docs/nest-react-components/inputs/Button.md` structure. Then add the component to the
`docs/nest-react-components/README.md` index under its category.

### A6. Test file (`<Name>.test.tsx`)

`vitest` + `@testing-library/react`. Cover: renders children/content; each variant maps
to its expected class (`toHaveClass('bg-nest-...')`); each size maps to its class;
event handlers fire; disabled/loading/error states apply; controlled + uncontrolled
behavior for stateful components; a11y attributes present. Mirror `Button.test.tsx`.

### A7. Definition of Done (per component)

- [ ] 4 files created, matching the contract.
- [ ] Added to category `index.ts` barrel.
- [ ] `npm run build` (tsc) passes with **zero** type errors.
- [ ] `npx vitest run <Name>` passes.
- [ ] No `@mui/*` import, no hardcoded colors, no template-literal class building.
- [ ] Prop names match MUI.

Verification commands (run from `core/crates/nest-react-components/`):
```bash
npm run build                 # tsc + vite build — must be clean
npx vitest run src/components/<category>/<Name>.test.tsx
```

---

## Part B — Design tokens (the ONLY styling vocabulary)

Source: `nest-tailwind-preset.json`. These Tailwind class fragments are the complete
allowed palette. Do not use any others for themable values.

**Colors** — `bg-`, `text-`, `border-`, `ring-` + one of:
`nest-background`, `nest-foreground`, `nest-primary`, `nest-secondary`, `nest-border`,
`nest-surface`, `nest-accent`, `nest-muted`, `nest-success`, `nest-warning`,
`nest-error`, `nest-info`.
Opacity modifiers are allowed and encouraged for hover/tint: `bg-nest-primary/10`,
`hover:bg-nest-primary/90` (this is the established pattern).

**Spacing** — `nest-xs sm md lg xl xxl` (e.g. `p-nest-md`, `gap-nest-sm`). Plain
Tailwind spacing (`p-4`, `gap-2`, `h-10`) is fine for intrinsic component sizing, as
`Button` does; use `nest-*` spacing where a recipe calls for it.

**Radius** — `rounded-nest-sm | -md | -lg | -full`. Default for interactive surfaces:
`rounded-nest-md`.

**Typography** — the body font is applied globally on `body {}` (Inter), so most
components need no font class. If you need to set it explicitly, use `font-body`
(also `font-heading`, `font-mono`). **Note:** existing components use `font-nest-body`,
which is a **no-op** (that key is not defined in the config) — harmless, but prefer
`font-body` in new code and do not rely on `font-nest-*` doing anything.

**Standard focus ring** (reuse verbatim on all focusable custom elements):
`focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2`.

**Standard disabled** (reuse verbatim):
`disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none`.

---

## Part C — Dependencies

Tier 1 and Tier 2 add **no** runtime dependencies.

Tier 3 (behavior-heavy) uses a headless primitive layer instead of reimplementing
MUI's Popper/Modal/focus internals. Add these to `package.json` `dependencies`
**once, before starting Tier 3**, then `npm install`:

```json
"@floating-ui/react": "^0.27.0",   // positioning + interactions (Tooltip, Popover, Menu, Select, Autocomplete)
"react-aria": "^3.x"                // focus-trap, dismiss, slider/aria hooks (Modal, Drawer, Slider)
```

Rules for Tier 3:
- Positioning (anchor → floating element) → `@floating-ui/react` (`useFloating`,
  `offset`, `flip`, `shift`, `autoUpdate`, `FloatingPortal`, `FloatingFocusManager`).
- Focus trap / dismiss / aria wiring for overlays → `@floating-ui/react` interaction
  hooks, falling back to `react-aria` hooks where a full ARIA pattern is needed
  (Slider especially: use `react-aria`'s `useSlider`).
- Still style everything with Nest tokens. The primitive provides **behavior only**.

---

## Part D — Component catalog & recipes

Port in tier order (1 → 3). Within a tier, top-to-bottom. Each recipe: MUI ref,
target folder, prop interface, style maps, structure, a11y, tests.

Legend: **MUI** = source path under `mui-material/src/`.

### TIER 1 — Trivial (markup + Tailwind, no state, no deps)

These are pure presentational. Each is ~30–80 lines. Follow the contract; below are
the specifics that differ per component.

#### T1.1 `Typography` → `data-display/Typography.tsx`
- **MUI:** `Typography/`
- **Props:** `variant?: 'h1'|'h2'|'h3'|'h4'|'h5'|'h6'|'subtitle1'|'subtitle2'|'body1'|'body2'|'caption'|'overline'` (default `'body1'`); `align?: 'inherit'|'left'|'center'|'right'|'justify'`; `color?: 'primary'|'secondary'|'foreground'|'muted'|'error'|'success'|'warning'` (default `'foreground'`); `gutterBottom?: boolean`; `noWrap?: boolean`; `component?: ElementType` (override rendered tag). Extends `HTMLAttributes<HTMLElement>`.
- **Style maps:** `VARIANT_STYLES` maps each variant to size/weight classes (e.g. `h1: 'text-4xl font-bold'`, `body1: 'text-sm'`, `overline: 'text-xs uppercase tracking-wide'`). `COLOR_STYLES` → `text-nest-*`. `align` → `text-left|center|right|justify`.
- **Structure:** render `component ?? defaultTagForVariant` (h1→`h1`, body1→`p`, etc). `noWrap` → `truncate`. `gutterBottom` → `mb-2`.
- **A11y:** semantic heading tags for h1–h6.
- **Tests:** variant→class, `component` override renders correct tag, `noWrap`→`truncate`.

#### T1.2 `Box` → `layout/Box.tsx`
- **MUI:** `Box/` (simplified — no `sx`). Props: `component?: ElementType` (default `'div'`), plus `className`. Just renders `component` with `cn(className)` and forwards ref + children. This is the escape-hatch primitive. ~15 lines.

#### T1.3 `Stack` → `layout/Stack.tsx`
- **MUI:** `Stack/`
- **Props:** `direction?: 'row'|'column'` (default `'column'`); `spacing?: 0|1|2|3|4|5|6|8` (maps to `gap-<n>`, default `2`); `align?: 'start'|'center'|'end'|'stretch'`; `justify?: 'start'|'center'|'end'|'between'|'around'`; `wrap?: boolean`; `component?: ElementType`.
- **Structure:** `flex` + `flex-row|flex-col` + `gap-*` + `items-*` + `justify-*` + optional `flex-wrap`.
- **Tests:** direction→flex class, spacing→gap class.

#### T1.4 `Grid` → `layout/Grid.tsx`
- **MUI:** `Grid/` (v2 API). **Props:** `container?: boolean`; `columns?: number` (default 12); `spacing?: number` (→ `gap-*`); item sizing `size?: number | 'auto' | { xs?, sm?, md?, lg? }`. Implement with CSS grid: container → `grid grid-cols-12 gap-*`; item → `col-span-<n>` (map size → `col-span-{1..12}`, responsive prefixes `sm:col-span-`, `md:col-span-`). Keep the responsive map explicit (static class strings; no dynamic `col-span-${n}` — Tailwind can't see those). Provide a `SPAN_CLASS: Record<number,string>` lookup 1–12.
- **Tests:** container→`grid`, size→`col-span-6` etc.

#### T1.5 `Container` → `layout/Container.tsx`
- **Props:** `maxWidth?: 'sm'|'md'|'lg'|'xl'|false` (default `'lg'`) → `max-w-screen-*` map; `disableGutters?: boolean` (default false → `px-4`). Centered via `mx-auto w-full`.

#### T1.6 `Paper` → `surface/Paper.tsx`
- **MUI:** `Paper/`. **Props:** `elevation?: 0|1|2|3|4` (default 1) → `shadow-none|shadow-sm|shadow|shadow-md|shadow-lg`; `variant?: 'elevation'|'outlined'` (outlined → `border border-nest-border shadow-none`); `square?: boolean` (false → `rounded-nest-md`). Base: `bg-nest-surface text-nest-foreground`.

#### T1.7 `Divider` → `layout/Divider.tsx` (also re-export from surface if a recipe needs it)
- **Props:** `orientation?: 'horizontal'|'vertical'` (default horizontal); `flexItem?: boolean`; `children?` (text divider). Horizontal → `border-t border-nest-border w-full`; vertical → `border-l border-nest-border h-full`. With children: flex row with lines on each side.
- **A11y:** `role="separator"`, `aria-orientation`.

#### T1.8 `Card` (+ subcomponents) → `surface/Card.tsx`
- **MUI:** `Card/`, `CardHeader/`, `CardContent/`, `CardActions/`, `CardMedia/`. Export all five from this file.
- `Card`: wraps `Paper` styling — `bg-nest-surface border border-nest-border rounded-nest-lg overflow-hidden`, `elevation?` like Paper.
- `CardHeader`: props `title?: ReactNode`, `subheader?: ReactNode`, `avatar?: ReactNode`, `action?: ReactNode`. Flex row: avatar | (title/subheader stack) | action. `p-4`.
- `CardContent`: `p-4` wrapper.
- `CardActions`: flex row `gap-2 p-2`, `disableSpacing?`.
- `CardMedia`: `component?: 'img'|'div'`, `image?: string`, `height?`. `img` → `object-cover w-full`.
- **Tests:** renders title/subheader/action; content children render.

#### T1.9 `Chip` → `data-display/Chip.tsx`
- **Props:** `label: ReactNode`; `variant?: 'filled'|'outlined'` (default filled); `color?: ButtonColor-style union` (default `'primary'`... but MUI default is `'default'` → map `'default'` to `nest-surface/nest-foreground`); `size?: 'small'|'medium'`; `onDelete?: () => void` (renders an `X` from lucide-react); `icon?: ReactNode`; `clickable?: boolean`. Rounded-full pill.
- **A11y:** if `onDelete`, delete affordance is a `button` with `aria-label="delete"`.
- **Tests:** label renders; `onDelete` fires; delete icon present when handler given.

#### T1.10 `Avatar` → `data-display/Avatar.tsx`
- **Props:** `src?: string`, `alt?: string`, `variant?: 'circular'|'rounded'|'square'` (default circular), `size?: 'small'|'medium'|'large'` (map to `size-8|size-10|size-12`), `children?` (initials/icon fallback). If `src` fails or absent, render children on `bg-nest-muted text-nest-background`. Circular → `rounded-full`, rounded → `rounded-nest-md`, square → `rounded-none`.
- **Tests:** renders img with src; renders fallback children when no src.

#### T1.11 `Badge` → `data-display/Badge.tsx`
- **Props:** `badgeContent?: ReactNode`, `color?: color union` (default `'primary'`), `max?: number` (default 99 → show `99+`), `showZero?: boolean` (default false), `variant?: 'standard'|'dot'`, `anchorOrigin?` (default top-right), `children` (the wrapped element). Absolute-positioned pill/dot in a `relative inline-flex` wrapper.
- **Tests:** hides when content 0 and !showZero; caps at max.

#### T1.12 `Link` → `navigation/Link.tsx`
- **Props extend `AnchorHTMLAttributes`.** `underline?: 'none'|'hover'|'always'` (default hover); `color?: 'primary'|'inherit'` (default primary). `text-nest-primary`, hover underline. Focus ring.

#### T1.13 `Breadcrumbs` → `navigation/Breadcrumbs.tsx`
- **Props:** `separator?: ReactNode` (default `'/'`), `children` (array of `Link`/`Typography`). Renders a `nav[aria-label="breadcrumb"] > ol` with separators between items. `maxItems?` optional collapse (defer if complex — leave TODO).

#### T1.14 `List` (+ items) → `data-display/List.tsx`
- **MUI:** `List/`, `ListItem/`, `ListItemButton/`, `ListItemText/`, `ListItemIcon/`, `ListItemAvatar/`. Export all.
- `List`: `ul` with `role="list"`, `dense?: boolean`.
- `ListItem`: `li` flex row `gap-3 px-3 py-2`.
- `ListItemButton`: interactive `button` variant, `selected?` → `bg-nest-primary/10`, hover `bg-nest-surface`, focus ring.
- `ListItemText`: `primary?: ReactNode`, `secondary?: ReactNode` stacked.
- `ListItemIcon`/`ListItemAvatar`: shrink-0 leading slot `text-nest-muted`.
- **Tests:** selected class; button onClick.

#### T1.15 `CircularProgress` → `feedback/CircularProgress.tsx`
- **Props:** `size?: 'small'|'medium'|'large'|number`, `color?: color union` (default primary), `variant?: 'indeterminate'|'determinate'`, `value?: number` (0–100 for determinate). Indeterminate: reuse the spinner SVG from `Button.tsx` (`animate-spin`), colored via `text-nest-*`. Determinate: stroke-dasharray from value.
- **A11y:** `role="progressbar"`, `aria-valuenow` for determinate.

#### T1.16 `LinearProgress` → `feedback/LinearProgress.tsx`
- **Props:** `variant?: 'indeterminate'|'determinate'`, `value?`, `color?`. Track `bg-nest-surface h-1 rounded-nest-full`, bar `bg-nest-*`. Indeterminate → an `animate-pulse`/translate keyframe (use an inline `animate-` utility; if none exists, use a simple pulsing bar and leave TODO for keyframe).
- **A11y:** `role="progressbar"`.

#### T1.17 `Skeleton` → `feedback/Skeleton.tsx`
- **Props:** `variant?: 'text'|'circular'|'rectangular'|'rounded'`, `width?`, `height?`, `animation?: 'pulse'|'wave'|false` (default pulse → `animate-pulse`). `bg-nest-muted/30`. text→`h-4 rounded`, circular→`rounded-full`, rectangular→`rounded-none`, rounded→`rounded-nest-md`.

#### T1.18 `ButtonGroup` → `inputs/ButtonGroup.tsx`
- **Props:** `variant?`, `color?`, `size?`, `orientation?: 'horizontal'|'vertical'`, `fullWidth?`, `children` (Buttons). Wrapper joins children with shared borders: `inline-flex` + strip inner radii (`[&>button]:rounded-none [&>button:first-child]:rounded-l-nest-md [&>button:last-child]:rounded-r-nest-md`). Passes `variant/color/size` down via cloneElement.

#### T1.19 `FormControl` / `FormLabel` / `FormHelperText` → `inputs/FormControl.tsx`
- Thin layout wrappers. `FormControl`: `fieldset`-like `div` flex-col gap-1, `error?`, `disabled?`, provides context (optional — can be prop-drilled; if context adds complexity, keep them presentational and leave a TODO). `FormLabel`: `label` `text-sm font-medium`, error→`text-nest-error`. `FormHelperText`: `p text-xs text-nest-muted`, error→`text-nest-error`.

### TIER 2 — Stateful but self-contained (use `useControllableState`, no deps)

#### T2.1 `Checkbox` → `inputs/Checkbox.tsx`
- **MUI:** `Checkbox/`. **Props extend `InputHTMLAttributes` minus `size`.** `checked?`, `defaultChecked?`, `onChange?`, `indeterminate?`, `color?` (default primary), `size?: 'small'|'medium'`, `disabled?`. Render a visually-hidden native `input[type=checkbox]` (for a11y/forms) plus a styled box showing a lucide `Check`/`Minus`(indeterminate) icon. Use `useControllableState<boolean>`. Checked box → `bg-nest-primary border-nest-primary text-white`.
- **A11y:** real checkbox input, `aria-checked` handled natively; `indeterminate` set via ref.
- **Tests:** toggles (controlled + uncontrolled); indeterminate shows minus; onChange fires.

#### T2.2 `Radio` / `RadioGroup` → `inputs/Radio.tsx`
- `RadioGroup`: `value?`, `defaultValue?`, `onChange?`, `name`, `row?`. Provides selected value via context to `Radio`s. `Radio`: `value`, styled dot. `useControllableState` at group level.
- **A11y:** `role="radiogroup"`; native `input[type=radio]` per option.

#### T2.3 `Switch` → `inputs/Switch.tsx`
- Like Checkbox but toggle track+thumb. `checked/defaultChecked/onChange/color/size/disabled`. Track `bg-nest-muted` off / `bg-nest-primary` on; thumb translate. `role="switch"` via native checkbox.

#### T2.4 `ToggleButton` / `ToggleButtonGroup` → `inputs/ToggleButton.tsx`
- Group manages single or multiple (`exclusive?`) selection via `value`/`onChange`. Buttons show pressed state `bg-nest-primary/15 text-nest-primary border-nest-primary`. `aria-pressed`.

#### T2.5 `Accordion` (+ Summary/Details) → `surface/Accordion.tsx`
- **MUI:** `Accordion/`, `AccordionSummary/`, `AccordionDetails/`. `Accordion`: `expanded?`/`defaultExpanded?`/`onChange?` (`useControllableState<boolean>`), `disabled?`. `AccordionSummary`: button header with a lucide `ChevronDown` that rotates when expanded (`rotate-180 transition-transform`). `AccordionDetails`: collapsible region (`grid grid-rows-[0fr]`→`[1fr]` transition, or simple show/hide). 
- **A11y:** summary is a `button` with `aria-expanded`, `aria-controls`; details `role="region"`.
- **Tests:** toggles open/close; controlled mode.

#### T2.6 `Tabs` (+ Tab) → `navigation/Tabs.tsx`
- `Tabs`: `value`, `onChange(event, newValue)`, `variant?: 'standard'|'fullWidth'`, `orientation?`. `Tab`: `value`, `label`, `icon?`, `disabled?`. Active tab → `text-nest-primary border-b-2 border-nest-primary`; inactive `text-nest-muted`. Manage active via controllable value.
- **A11y:** `role="tablist"`, tabs `role="tab"` + `aria-selected`, panels `role="tabpanel"`. Arrow-key navigation between tabs.
- **Tests:** clicking tab fires onChange; active tab styled.

#### T2.7 `Pagination` → `navigation/Pagination.tsx`
- `count`, `page`, `onChange(event, page)`, `siblingCount?`, `boundaryCount?`, `size?`, `disabled?`. Compute the page-button list (with `…` ellipsis). Buttons reuse Button/IconButton styling; current page → `bg-nest-primary text-white`.
- **A11y:** `nav[aria-label="pagination"]`; current → `aria-current="page"`.

#### T2.8 `Stepper` (+ Step/StepLabel) → `navigation/Stepper.tsx`
- `activeStep`, `orientation?`, `alternativeLabel?`. Step index circle: completed → `bg-nest-primary text-white` w/ Check icon, active → ring, upcoming → `bg-nest-muted`. Connector line between.

#### T2.9 `Rating` → `inputs/Rating.tsx`
- `value`/`defaultValue`/`onChange`, `max?` (default 5), `precision?` (0.5/1), `readOnly?`, `size?`. lucide `Star` filled/empty; hover preview state. `useControllableState<number>`.
- **A11y:** `role="radiogroup"` semantics or a labelled slider; keep it simple with radios; leave TODO if precision<1 gets hard.

#### T2.10 Transitions `Fade` / `Grow` / `Collapse` → `feedback/Transitions.tsx`
- Props: `in: boolean`, `timeout?`, `children`. CSS transition wrappers (opacity for Fade, scale for Grow, height/`grid-rows` for Collapse). Unmount after exit via a small `useState` delay. Keep minimal; these support overlays later.

#### T2.11 `Backdrop` → `feedback/Backdrop.tsx`
- `open: boolean`, `onClick?`, `invisible?`. Fixed inset overlay `bg-black/50` with fade. Used by Modal/Drawer.

### TIER 3 — Behavior-heavy (uses `@floating-ui/react` / `react-aria` — see Part C)

Do **not** start Tier 3 until Part C deps are installed. Each of these builds behavior
on the primitive and styles with tokens.

#### T3.1 `Tooltip` → `data-display/Tooltip.tsx`
- **Primitive:** `@floating-ui/react` — `useFloating` + `offset(8)` + `flip()` + `shift()`, `useHover`, `useFocus`, `useDismiss`, `useRole({role:'tooltip'})`, `FloatingPortal`.
- **Props:** `title: ReactNode`, `placement?`, `arrow?`, `enterDelay?`, `children` (single element that becomes the anchor via `getReferenceProps`). Panel: `bg-nest-foreground text-nest-background text-xs rounded-nest-sm px-2 py-1`.
- **A11y:** `aria-describedby` wired by the role hook.
- **Tests:** shows on hover/focus, hides on blur (use `userEvent`).

#### T3.2 `Popover` → `feedback/Popover.tsx`
- **Primitive:** `useFloating` + `FloatingPortal` + `FloatingFocusManager` + `useDismiss` (outside click + escape). **Props:** `open`, `anchorEl` (or render-prop anchor), `onClose`, `anchorOrigin?`, `placement?`. Panel `bg-nest-surface border border-nest-border rounded-nest-md shadow-lg`. This is the base for Menu positioning, Select, Autocomplete.

#### T3.3 `Menu` (upgrade existing) → `navigation/Menu.tsx`
- The existing `Menu.tsx` uses **naive CSS-absolute positioning** (`absolute left-0 top-full`) under a `position:relative` ancestor — no collision handling, no portal, no focus management. **Re-implement on `Popover`/floating-ui** so it anchors correctly (flip/shift), portals to body, traps focus, closes on outside-click/escape, and supports arrow-key navigation (`useListNavigation`). Items: `MenuItem` with `selected?`, `disabled?`, `onClick`. Keep existing prop names; note any breaking change in the docs file.
- **A11y:** `role="menu"`/`menuitem`, roving tabindex.

#### T3.4 `Select` → `inputs/Select.tsx`
- Built on floating-ui listbox (`useFloating` + `useListNavigation` + `useTypeahead` + `FloatingFocusManager`). **Props:** `value`/`defaultValue`/`onChange`, `label?`, `options` or `MenuItem` children, `multiple?`, `size?`, `error?`, `disabled?`. Trigger styled like `TextField` outlined; dropdown like `Popover`. `useControllableState`.
- **A11y:** `role="combobox"` + `listbox`, `aria-expanded`, `aria-activedescendant`.

#### T3.5 `Autocomplete` → `inputs/Autocomplete.tsx`
- Extends Select with a text `input` filter. Props (MUI subset): `options`, `value`/`onChange`, `inputValue`/`onInputChange`, `getOptionLabel?`, `multiple?`, `freeSolo?`, `loading?`, `renderOption?`. Filter options by input; highlight active via `useListNavigation`. This is the hardest component — if `multiple`+`freeSolo` combined gets ambiguous, implement single-select first and leave `// TODO(port): multiple/freeSolo`.

#### T3.6 `Modal` / `Dialog` (align) → `feedback/Modal.tsx`
- `Dialog` already exists. Add a lower-level `Modal` (`open`, `onClose`, `children`) using `FloatingPortal` + `FloatingFocusManager` (focus trap + restore) + `useDismiss` + `Backdrop`. If `Dialog` currently hand-rolls focus, refactor it to use `Modal` underneath **without changing Dialog's public props**. Document the internal change.
- **A11y:** `role="dialog"`, `aria-modal="true"`, focus trapped, escape closes, focus restored on close.

#### T3.7 `Drawer` → `navigation/Drawer.tsx`
- **Primitive:** `Modal` base + slide transition. **Props:** `open`, `onClose`, `anchor?: 'left'|'right'|'top'|'bottom'` (default left), `variant?: 'temporary'|'permanent'`. Temporary → Backdrop + slide-in panel `bg-nest-surface`. Permanent → static, no backdrop.

#### T3.8 `Slider` → `inputs/Slider.tsx`
- **Primitive:** `react-aria` `useSlider` + `useSliderThumb` (handles keyboard, drag, aria). **Props:** `value`/`defaultValue`/`onChange`, `min?`, `max?`, `step?`, `marks?`, `disabled?`, `size?`, `orientation?`, `valueLabelDisplay?`. Track `bg-nest-muted`, filled `bg-nest-primary`, thumb `bg-nest-primary rounded-full shadow` w/ focus ring.
- **A11y:** provided by react-aria; verify `aria-valuenow/min/max`.

#### T3.9 `SpeedDial` → `navigation/SpeedDial.tsx`
- FAB (`IconButton`-based) that expands a stack of actions on hover/click using floating-ui positioning + Grow transitions. Lower priority — port after everything else.

#### T3.10 `Table` (sortable) → `data-display/Table.tsx`
- Start with the **static** primitives (`Table`, `TableHead`, `TableBody`, `TableRow`, `TableCell`, `TableContainer`) as Tier-1-style markup (`table`/`thead`/`tbody`, tokened borders, `TableCell` `align?`, `padding?`). Sorting/selection/pagination are **out of scope for v1** — consumers compose `Pagination` + their own sort. Leave `// TODO(port): TableSortLabel` if needed.

---

## Part E — Execution protocol (for the executor)

1. **One component per commit.** Branch off `main`. Commit message:
   `Port <Name> to nest-react-components`.
2. Before writing: open the MUI source file for prop names and `Button.tsx` /
   `TextField.tsx` for the house style.
3. Create the 4 files → update the category `index.ts` barrel.
4. Run `npm run build` and `npx vitest run <path>`. Both must be green.
5. If anything in the recipe is ambiguous or a component needs behavior the recipe
   didn't specify, **stop and leave a `// TODO(port): <question>` comment** — do not
   invent API surface. A human will resolve TODOs.
6. Never: import `@mui/*`, hardcode colors, use raw Tailwind palette colors for themable
   values, build class strings with template literals, or skip the barrel export.
7. Write the usage doc to `docs/nest-react-components/<category>/<Name>.md` and add a link
   to it under the correct category heading in `docs/nest-react-components/README.md`.

## Ordering summary (recommended sequence)

Tier 1: Typography → Box → Stack → Grid → Container → Paper → Divider → Card →
Chip → Avatar → Badge → Link → Breadcrumbs → List → CircularProgress →
LinearProgress → Skeleton → ButtonGroup → FormControl.

Tier 2: Checkbox → Radio → Switch → ToggleButton → Accordion → Tabs → Pagination →
Stepper → Rating → Transitions → Backdrop.

Tier 3 (after Part C deps): Tooltip → Popover → Menu(upgrade) → Modal(+Dialog align)
→ Drawer → Select → Autocomplete → Slider → SpeedDial → Table.

---

*See `docs/plan/nest-react-components-v1.md` for the original library plan and
`docs/architecture.md` for the desktop frontend platform decision.*
