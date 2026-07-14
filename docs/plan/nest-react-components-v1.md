# nest-react-components v1 — Reusable React Component Library

## Status: Approved — Library Implementation Only

**Decisions confirmed:**
- Icon library: **Lucide React**
- Demo app: **Custom Nest Shell** (no Storybook)
- Package name: **`@nest/components`**
- Scope: **Library only** — no app migrations in v1

A shared React component library for all Nest desktop applications, providing MUI-equivalent components styled with Nest design tokens and Tailwind CSS.

---

## Context

**Problem:** Every Nest desktop app (Kiwi, Loon, future apps) needs common UI components. Currently:
- Components are duplicated across apps (Kiwi has `AppShell`, `Ribbon`, `ConfirmDialog`, etc. in `apps/kiwi/ui/src/components/`)
- The desktop template (`templates/desktop/`) has some shared components but no formal component library
- No centralized documentation or demo app for components
- No MUI-equivalent coverage for common components (Button, TextField, Dialog, etc.)

**Goal:** Create a reusable, well-documented component library that:
1. Lives in the Nest monorepo (shared across all apps)
2. Uses Nest design tokens from `nest-design` / `nest-react-theme`
3. Matches MUI Material API conventions for familiarity
4. Includes a demo app in the Nest Desktop Shell showing each component with usage docs

---

## Architecture

```text
core/crates/nest-react-components/    # npm package source (TypeScript + React)
├── src/
│   ├── components/                   # All components
│   │   ├── inputs/                   # Button, TextField, Checkbox, Radio, Select, etc.
│   │   ├── feedback/                 # Dialog, Alert, Snackbar, Progress, Skeleton
│   │   ├── navigation/               # AppBar, Drawer, Tabs, Breadcrumbs, Pagination
│   │   ├── surface/                  # Card, Accordion, Paper, Divider
│   │   ├── data-display/             # List, Table, Avatar, Badge, Tooltip
│   │   └── layout/                   # Box, Grid, Stack, Container, Divider
│   ├── hooks/                        # Shared hooks (useToast, useSnackbar, etc.)
│   ├── context/                      # Context providers (ThemeProvider, etc.)
│   ├── lib/                          # Utilities (cn class merger, etc.)
│   └── index.ts                      # Public barrel exports
├── package.json
├── tsconfig.json
├── tailwind.config.ts                # Extends nest-react-theme preset
└── README.md                         # Component API docs

apps/nest-shell/                      # Demo app / component gallery
├── ui/                               # React app showing all components
│   ├── src/
│   │   ├── components/               # Demo pages for each component
│   │   └── App.tsx                   # Navigation + component showcase
│   └── package.json (depends on nest-react-components)
└── src-tauri/                        # Tauri host
```

---

## Component Catalog (MUI Reference)

Priority order: **Phase 1** (immediate needs) → **Phase 4** (specialized)

### Phase 1: Foundation + Kiwi Migration Needs

| Component | MUI Reference | Nest Equivalent | Priority |
|-----------|---------------|-----------------|----------|
| `Button` | [mui.com/material-ui/react-button](https://mui.com/material-ui/react-button/) | — | **P1** |
| `TextField` | [mui.com/material-ui/react-text-field](https://mui.com/material-ui/react-text-field/) | — | **P1** |
| `IconButton` | [mui.com/material-ui/react-icon-button](https://mui.com/material-ui/react-icon-button/) | — | **P1** |
| `Dialog` | [mui.com/material-ui/react-dialog](https://mui.com/material-ui/react-dialog/) | `ConfirmDialog` (kiwi) | **P1** |
| `AppBar` | [mui.com/material-ui/react-app-bar](https://mui.com/material-ui/react-app-bar/) | `AppShell` ribbon area | **P1** |
| `Drawer` | [mui.com/material-ui/react-drawer](https://mui.com/material-ui/react-drawer/) | `AppShell` rail | **P1** |
| `Snackbar` | [mui.com/material-ui/react-snackbar](https://mui.com/material-ui/react-snackbar/) | `ToastViewport` (kiwi) | **P1** |
| `Alert` | [mui.com/material-ui/react-alert](https://mui.com/material-ui/react-alert/) | — | **P1** |

### Phase 2: Navigation + Selection

| Component | MUI Reference | Notes |
|-----------|---------------|-------|
| `Tabs` | [mui.com/material-ui/react-tabs](https://mui.com/material-ui/react-tabs/) | Alternative to Ribbon for simpler nav |
| `Menu` | [mui.com/material-ui/react-menu](https://mui.com/material-ui/react-menu/) | Context menus |
| `Select` | [mui.com/material-ui/react-select](https://mui.com/material-ui/react-select/) | Dropdown selection |
| `Checkbox` | [mui.com/material-ui/react-checkbox](https://mui.com/material-ui/react-checkbox/) | Boolean input |
| `Radio` | [mui.com/material-ui/react-radio-button](https://mui.com/material-ui/react-radio-button/) | Mutually exclusive |
| `Switch` | [mui.com/material-ui/react-switch](https://mui.com/material-ui/react-switch/) | Toggle on/off |
| `Slider` | [mui.com/material-ui/react-slider](https://mui.com/material-ui/react-slider/) | Range input |
| `Autocomplete` | [mui.com/material-ui/react-autocomplete](https://mui.com/material-ui/react-autocomplete/) | Search with suggestions |

### Phase 3: Data Display

| Component | MUI Reference | Notes |
|-----------|---------------|-------|
| `Card` | [mui.com/material-ui/react-card](https://mui.com/material-ui/react-card/) | Content containers |
| `List` | [mui.com/material-ui/react-list](https://mui.com/material-ui/react-list/) | Vertical lists |
| `Table` | [mui.com/material-ui/react-table](https://mui.com/material-ui/react-table/) | Data grids |
| `Avatar` | [mui.com/material-ui/react-avatar](https://mui.com/material-ui/react-avatar/) | User images |
| `Badge` | [mui.com/material-ui/react-badge](https://mui.com/material-ui/react-badge/) | Status counters |
| `Tooltip` | [mui.com/material-ui/react-tooltip](https://mui.com/material-ui/react-tooltip/) | Hover info |
| `Typography` | [mui.com/material-ui/react-typography](https://mui.com/material-ui/react-typography/) | Text styles |
| `Progress` | [mui.com/material-ui/react-progress](https://mui.com/material-ui/react-progress/) | Linear/circular loaders |
| `Skeleton` | [mui.com/material-ui/react-skeleton](https://mui.com/material-ui/react-skeleton/) | Loading placeholders |

### Phase 4: Specialized

| Component | MUI Reference | Notes |
|-----------|---------------|-------|
| `Accordion` | [mui.com/material-ui/react-accordion](https://mui.com/material-ui/react-accordion/) | Expandable panels |
| `Breadcrumbs` | [mui.com/material-ui/react-breadcrumbs](https://mui.com/material-ui/react-breadcrumbs/) | Navigation path |
| `Pagination` | [mui.com/material-ui/react-pagination](https://mui.com/material-ui/react-pagination/) | Page navigation |
| `SpeedDial` | [mui.com/material-ui/react-speed-dial](https://mui.com/material-ui/react-speed-dial/) | FAB with actions |
| `Stepper` | [mui.com/material-ui/react-stepper](https://mui.com/material-ui/react-stepper/) | Wizard steps |
| `Timeline` | [mui.com/material-ui/react-timeline](https://mui.com/material-ui/react-timeline/) | Vertical timeline |

---

## Component API Design

Follow MUI conventions for familiarity, adapted for Nest tokens:

### Example: Button

```tsx
// Usage matches MUI API
<Button
  variant="contained"           // "text" | "outlined" | "contained"
  color="primary"               // "primary" | "secondary" | "accent" | "error"
  size="medium"                 // "small" | "medium" | "large"
  disabled
  loading                       // v6+ loading state
  startIcon={<SaveIcon />}
  onClick={handleSave}
>
  Save
</Button>
```

### Example: TextField

```tsx
<TextField
  label="Email"
  value={email}
  onChange={(e) => setEmail(e.target.value)}
  error={!!error}
  helperText={error}
  multiline
  rows={4}
  size="medium"                 // "small" | "medium"
  variant="outlined"            // "outlined" | "filled" | "standard"
  InputProps={{ startAdornment: <SearchIcon /> }}
/>
```

### Token Integration

Components use Nest design tokens via Tailwind:

```tsx
// Button uses theme tokens
className={[
  "rounded-nest-md px-4 py-2 font-nest-body",
  variant === "contained" && "bg-nest-primary text-white hover:bg-nest-primary/90",
  variant === "outlined" && "border border-nest-primary text-nest-primary hover:bg-nest-primary/10",
  size === "small" && "text-xs px-2 py-1",
  disabled && "opacity-50 cursor-not-allowed",
].join(" ")}
```

---

## nest-react-theme Integration

The component library extends the existing theme system:

1. **`nest-design`** (Rust): Token schema (`ThemeDefinition`, `ColorTokens`, `TypographyTokens`)
2. **`nest-react-theme`** (Rust): Converts tokens → CSS custom properties + Tailwind preset JSON
3. **`nest-react-components`** (TypeScript): Components consume CSS variables via Tailwind

```ts
// nest-tailwind-preset.json (generated by nest-react-theme Rust crate)
{
  "theme": {
    "extend": {
      "colors": {
        "nest-background": "var(--nest-color-background)",
        "nest-primary": "var(--nest-color-primary)",
        ...
      },
      "borderRadius": {
        "nest-sm": "var(--nest-radius-sm)",
        ...
      }
    }
  }
}
```

Components reference these in Tailwind classes: `bg-nest-primary`, `rounded-nest-md`, etc.

---

## Demo App: Nest Shell

A dedicated desktop app (`apps/nest-shell/`) showcasing all components:

### Features

- **Component Gallery**: Grid view of all components with live previews
- **API Documentation**: Props table, usage examples, copy-paste snippets
- **Theme Toggle**: Switch between light/dark themes to preview components
- **Interactive Playground**: Adjust props and see live updates
- **Search**: Find components by name or category

### Layout

```text
┌─────────────────────────────────────────────────────────────┐
│  Nest Components                    [Theme: Light ▼]  🔍    │
├──────────┬──────────────────────────────────────────────────┤
│ Inputs   │  ┌────────────────────────────────────────────┐  │
│ Button   │  │  Button Preview                            │  │
│ TextField│  │  [Primary] [Secondary] [Accent]            │  │
│ Feedback │  │  [Outlined] [Text]                         │  │
│ Surface  │  └────────────────────────────────────────────┘  │
│ Navigation│                                                  │
│ Data     │  Props:                                           │
│          │  - variant: "contained" | "outlined" | "text"    │
│          │  - color: "primary" | "secondary" | "accent"     │
│          │  - size: "small" | "medium" | "large"            │
│          │  - disabled, loading, startIcon, endIcon         │
│          │                                                   │
│          │  Usage:                                           │
│          │  <Button variant="contained">Click me</Button>   │
└──────────┴──────────────────────────────────────────────────┘
```

### Scaffold from Template

```bash
# Copy desktop template as starting point
cp -r templates/desktop apps/nest-shell
cd apps/nest-shell

# Update app identity
# - src-tauri/tauri.conf.json: identifier = "com.pacificnm.nest-shell"
# - src-tauri/Cargo.toml: name = "nest-shell"
# - ui/package.json: name = "@nest/shell"
```

---

## Implementation Phases

### Phase 0: Infrastructure Setup (Week 1)

| Task | Deliverable |
|------|-------------|
| Create `core/crates/nest-react-components/` npm package | Package structure with build config |
| Set up Tailwind config extending `nest-react-theme` preset | `tailwind.config.ts` |
| Configure TypeScript, ESLint, Prettier | Linting + formatting |
| Set up Storybook (optional) or custom demo app | Component development environment |
| Create barrel exports (`index.ts`) | Public API surface |

### Phase 1: Foundation Components (Week 2-3)

| Task | Deliverable |
|------|-------------|
| Implement Button, IconButton, TextField | Input components with all variants |
| Implement Dialog, Alert, Snackbar | Feedback components |
| Migrate Kiwi's `ConfirmDialog`, `ToastViewport` | Reuse existing tested code |
| Migrate Kiwi's `AppShell`, `Ribbon` | Layout components |
| Write unit tests (Vitest + React Testing Library) | Test coverage |
| Add to Nest Shell demo app | Working gallery pages |

### Phase 2: Navigation + Selection (Week 4-5)

| Task | Deliverable |
|------|-------------|
| Implement Tabs, Menu, Select | Navigation components |
| Implement Checkbox, Radio, Switch, Slider | Selection components |
| Implement Autocomplete | Search input |
| Document all components in Nest Shell | API docs + examples |

### Phase 3: Data Display (Week 6-7)

| Task | Deliverable |
|------|-------------|
| Implement Card, List, Table | Container components |
| Implement Avatar, Badge, Tooltip | Decorative components |
| Implement Typography, Progress, Skeleton | Content components |
| Kiwi migration: replace local imports | Kiwi uses `@nest/components` |

### Phase 4: Specialized + Polish (Week 8+)

| Task | Deliverable |
|------|-------------|
| Implement Accordion, Breadcrumbs, Pagination | Advanced components |
| Implement SpeedDial, Stepper, Timeline | Wizard/flow components |
| Accessibility audit (WCAG 2.1 AA) | ARIA compliance |
| Performance optimization (bundle size, memoization) | Production ready |

---

## Package Publishing Strategy

### Option A: Monorepo Path Dependency (Recommended for v1)

Apps depend on the local package via workspace:

```json
// apps/kiwi/ui/package.json
{
  "dependencies": {
    "@nest/components": "*"
  }
}

// Root package.json (if using npm workspaces)
{
  "workspaces": ["core/crates/nest-react-components", "apps/*/ui"]
}
```

**Pros:** Simple, no publishing needed, changes reflect immediately  
**Cons:** Requires monorepo checkout

### Option B: npm Package (Future)

Publish to npm or GitHub Packages:

```bash
cd core/crates/nest-react-components
npm publish --scope @nest
```

Apps consume:

```json
{
  "@nest/components": "^1.0.0"
}
```

**Pros:** Works for external apps, versioned releases  
**Cons:** Publishing overhead, sync lag

### Decision: Start with Option A, migrate to B after v1 stabilizes.

---

## Migration Path for Kiwi

Kiwi currently has components in `apps/kiwi/ui/src/components/`:

```tsx
// Before (Kiwi local)
import { ConfirmDialog, AppShell, Ribbon } from "./components";

// After (shared library)
import { ConfirmDialog, AppShell, Ribbon } from "@nest/components";
```

### Migration Steps

1. **Audit Kiwi components**: List all components Kiwi uses
2. **Port to library**: Move each to `nest-react-components`, preserve API
3. **Update Kiwi imports**: Change to `@nest/components`
4. **Delete local copies**: Remove from `apps/kiwi/ui/src/components/`
5. **Verify tests pass**: Ensure no regressions

---

## Testing Strategy

### Unit Tests (Vitest + React Testing Library)

```tsx
// src/components/Button/Button.test.tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { Button } from "./Button";

describe("Button", () => {
  it("renders children", () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText("Click me")).toBeInTheDocument();
  });

  it("calls onClick when clicked", () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click</Button>);
    fireEvent.click(screen.getByRole("button"));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("shows loading state", () => {
    render(<Button loading>Loading</Button>);
    expect(screen.getByRole("button")).toHaveAttribute("disabled");
  });
});
```

### Visual Regression (Optional)

Use Chromatic or Percy for screenshot testing across themes.

---

## Accessibility Requirements

All components must meet **WCAG 2.1 AA**:

- **Keyboard navigation**: Tab, Enter, Space, Escape work correctly
- **ARIA attributes**: `aria-label`, `aria-expanded`, `aria-selected` as appropriate
- **Focus management**: Focus traps in Dialogs, focus restoration
- **Screen reader support**: Announce state changes (loading, errors)
- **Color contrast**: Meet 4.5:1 ratio for text

Reference: [MUI Accessibility](https://mui.com/material-ui/guides/accessibility/)

---

## File Structure

```
core/crates/nest-react-components/
├── src/
│   ├── components/
│   │   ├── Button/
│   │   │   ├── Button.tsx
│   │   │   ├── Button.test.tsx
│   │   │   └── index.ts
│   │   ├── TextField/
│   │   │   ├── TextField.tsx
│   │   │   ├── TextField.test.tsx
│   │   │   └── index.ts
│   │   └── index.ts
│   ├── hooks/
│   │   ├── useToast.ts
│   │   └── index.ts
│   ├── context/
│   │   ├── ThemeProvider.tsx
│   │   └── index.ts
│   └── index.ts
├── package.json
├── tsconfig.json
├── vitest.config.ts
├── tailwind.config.ts
└── README.md

apps/nest-shell/
├── ui/
│   └── src/
│       ├── components/
│       │   ├── ButtonDemo.tsx
│       │   ├── TextFieldDemo.tsx
│       │   └── App.tsx
│       └── main.tsx
└── src-tauri/
    ├── Cargo.toml
    └── src/
        └── main.rs
```

---

## Dependencies

### nest-react-components

```json
{
  "name": "@nest/components",
  "version": "0.1.0",
  "peerDependencies": {
    "react": "^18.x",
    "react-dom": "^18.x",
    "tailwindcss": "^3.x"
  },
  "dependencies": {
    "@fortawesome/react-fontawesome": "^0.2.x",
    "clsx": "^2.x",
    "tailwind-merge": "^2.x"
  },
  "devDependencies": {
    "@testing-library/react": "^14.x",
    "@types/react": "^18.x",
    "typescript": "^5.x",
    "vitest": "^1.x"
  }
}
```

### nest-shell (demo app)

```json
{
  "dependencies": {
    "@nest/components": "*",
    "@nest/icons": "*",
    "react-router-dom": "^6.x"
  }
}
```

---

## Risks + Mitigations

| Risk | Mitigation |
|------|------------|
| Component API drift from MUI | Document deviations explicitly; follow MUI v6+ patterns |
| Bundle size bloat | Tree-shakeable exports; code split by category |
| Theme token gaps | Extend `nest-design` tokens as needed; backward compatible |
| Kiwi migration complexity | Port components with identical APIs first; refactor later |
| Demo app maintenance burden | Generate docs from TypeScript JSDoc comments |

---

## Success Criteria

- [ ] All Phase 1-2 components implemented and tested
- [ ] Nest Shell demo app running with component gallery
- [ ] Kiwi successfully migrated to `@nest/components`
- [ ] Documentation for each component (props, examples, usage)
- [ ] Theme toggle works (light/dark preview)
- [ ] Accessibility audit passes (keyboard nav, ARIA)

---

## Related

- [nest-react-ui v1](./nest-react-ui-v1.md) — Icon/Image replacement strategy
- [nest-tauri v1](./nest-tauri-v1.md) — Desktop host architecture
- [nest-design](../nest-design/README.md) — Token schema
- [nest-react-theme](../nest-react-theme/README.md) — Theme → CSS/Tailwind adapter
- [MUI Material Components](https://mui.com/material-ui/getting-started/)
- [templates/desktop](../../templates/desktop/) — App scaffold
