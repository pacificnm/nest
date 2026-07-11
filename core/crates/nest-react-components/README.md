# @nest/components

Reusable React component library for Nest desktop applications.

## Installation

```bash
npm install @nest/components
```

## Quick Start

1. Import the styles in your app's entry point:

```ts
// main.tsx
import '@nest/components/styles.css';
```

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

### Feedback

| Component | Description |
|-----------|-------------|
| `Dialog` | Modal overlay for confirmations and forms |
| `Alert` | Inline message with severity levels |
| `Snackbar` | Brief toast message with auto-dismiss |

### More Coming Soon

- Navigation: AppBar, Drawer, Tabs, Menu
- Surface: Card, Accordion, Paper
- Data Display: List, Table, Avatar, Badge, Tooltip
- Layout: Box, Stack, Grid, Container

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
# Install dependencies
npm install

# Run tests
npm test

# Build
npm run build

# Lint
npm run lint

# Format
npm run format
```

## License

MIT
