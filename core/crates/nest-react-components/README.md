# @nest/components

Reusable React component library for Nest desktop applications.

## Installation

Not published — consuming apps reference it as a local path dependency
(this package is `"private": true`):

```json
// ui/package.json
"dependencies": {
  "@nest/components": "../../../core/crates/nest-react-components"
}
```

`npm install` in `ui/` alone is **not** enough the first time: this
package ships raw TypeScript source (`"main": "./src/index.ts"`, no
bundled dist), so its own bare imports (`clsx`, `tailwind-merge`, …) only
resolve once `npm install` has also been run directly inside this
package. `./build dev`/`./build` (see [docs/build.md](../../../docs/build.md))
does this automatically; a manual `npm run dev` does not — run
`npm install --prefix core/crates/nest-react-components` yourself first
in that case.

## Quick Start

1. No separate stylesheet import needed — `src/index.ts` already imports
   its own theme-independent motion CSS (`runtime.css`) as a side effect.
   Themable values (colors, spacing, radius) come from the host app at
   runtime via `nest_theme_css` (see [nest-tauri](../nest-tauri/README.md)),
   not from anything this package exports.

2. Wrap your app with ThemeProvider:

```tsx
import { ThemeProvider } from '@nest/components';

function App() {
  return (
    <ThemeProvider defaultMode="light">
      {/* Your app */}
    </ThemeProvider>
  );
}
```

3. Use components:

```tsx
import { Button, TextField, Alert } from '@nest/components';

function Example() {
  return (
    <div className="flex flex-col gap-4 p-4">
      <Alert severity="info">Welcome to Nest!</Alert>
      
      <TextField label="Email" placeholder="you@example.com" />
      
      <div className="flex gap-2">
        <Button variant="contained">Save</Button>
        <Button variant="outlined">Cancel</Button>
        <Button variant="text">Learn more</Button>
      </div>
    </div>
  );
}
```

## Components

### Inputs

| Component | Description |
|-----------|-------------|
| `Button` | Action button with variants (contained, outlined, text) |
| `IconButton` | Icon-only button for toolbar actions |
| `TextField` | Text input with label, helper text, and error states |
| `Select` | Dropdown selection |
| `Checkbox` / `Radio` / `Switch` / `ToggleButton` | Selection controls |
| `Slider` / `Rating` | Range/rating input |
| `Autocomplete` | Combobox with filtering |
| `ButtonGroup` / `FormControl` | Input grouping |

### Feedback

| Component | Description |
|-----------|-------------|
| `Dialog` | Modal overlay for confirmations and forms |
| `Alert` | Inline message with severity levels |
| `Snackbar` | Brief toast message with auto-dismiss |
| `Drawer` / `Modal` / `Popover` / `Backdrop` | Overlay primitives |
| `CircularProgress` / `LinearProgress` / `Skeleton` | Loading indicators |

### Navigation

`AppBar`, `Tabs`, `Menu`, `Stepper`, `Pagination`, `Link`, `Breadcrumbs`.

### Surface

`Card`, `Accordion`, `Paper`.

### Data Display

`Table` (+ `TableHead`/`TableBody`/`TableFooter`/`TableRow`/`TableCell`),
`List` (+ `ListItem`/`ListItemButton`/`ListItemText`/`ListItemIcon`/
`ListItemAvatar`), `Avatar`, `Badge`, `Chip`, `Tooltip`, `Typography`.

### Layout

`Box`, `Stack`, `Grid`, `Container`, `Divider`.

Every component above has a `.tsx` implementation, a `.test.tsx` suite, and
a `.demo.tsx` under `src/components/<category>/` — check there for exact
props rather than this README if in doubt (this file has drifted from the
real source before).

## Theming

Components use Nest design tokens via CSS custom properties. The `ThemeProvider` component manages light/dark mode:

```tsx
import { useTheme } from '@nest/components';

function ThemeToggle() {
  const { mode, toggleMode } = useTheme();
  return (
    <Button onClick={toggleMode}>
      Switch to {mode === 'light' ? 'dark' : 'light'}
    </Button>
  );
}
```

## API Conventions

This library follows MUI Material API conventions for familiarity:

- `variant` - Visual style (e.g., `contained`, `outlined`, `text`)
- `color` - Color scheme (e.g., `primary`, `secondary`, `error`)
- `size` - Component size (e.g., `small`, `medium`, `large`)
- `startIcon` / `endIcon` - Icon positions
- `onClose` - Close callback

See individual component documentation for full prop details.

## Development

```bash
# Install dependencies (required before anything below — see Installation)
npm install

# Run tests (vitest + @testing-library/react; no "test" script defined,
# run vitest directly)
npx vitest run

# Build
npm run build
```

## License

MIT
