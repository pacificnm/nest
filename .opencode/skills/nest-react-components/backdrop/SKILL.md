---
name: nest-react-backdrop
description: Use when working with the Backdrop React component from @nest/components – an overlay that can be used behind modals, drawers, loading states, or any content that should capture outside clicks.
---

# Backdrop

A backdrop overlay component for modals, drawers, and other overlays.

## When to Use

Use Backdrop when:
- Creating modal dialogs that need to capture clicks outside
- Building slide‑out drawers that need an overlay
- Showing loading states over content
- Dimming background content to focus attention
- Creating a clickable area to close overlays

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `boolean` | `false` | If true, the backdrop is shown |
| `onClick` | `(event) => void` | - | Callback fired when backdrop is clicked |
| `invisible` | `boolean` | `false` | If true, backdrop is invisible but clickable |
| `children` | `ReactNode` | - | Content to render inside the backdrop |
| `className` | `string` | - | Additional CSS classes |
| `aria-hidden` | `boolean` | `false` | If true, hidden from screen readers |

## Examples

### Basic Usage

```tsx
import { Backdrop } from '@nest/components';
import { useState } from 'react';

const [open, setOpen] = useState(false);

<Backdrop open={open} />
```

### Invisible Backdrop

```tsx
<Backdrop open={open} invisible onClick={() => setOpen(false)} />
```

### With Modal

```tsx
import { Backdrop } from '@nest/components';
import { Paper, Button } from '@nest/components';

<Backdrop open={open} onClick={() => setOpen(false)}>
  <div className="fixed inset-0 flex items-center justify-center">
    <Paper className="max-w-sm w-full p-6">
      <h3>Modal Title</h3>
      <p>Modal content...</p>
      <Button onClick={() => setOpen(false)}>Close</Button>
    </Paper>
  </div>
</Backdrop>
```

### Loading State

```tsx
<Backdrop open>
  <div className="fixed inset-0 flex items-center justify-center">
    <div className="flex items-center gap-2">
      <div className="w-5 h-5 border-2 border-primary border-t-transparent rounded-full animate-spin" />
      <span>Loading...</span>
    </div>
  </div>
</Backdrop>
```

### Custom Styling

```tsx
<Backdrop open className="bg-primary/30 backdrop-blur-sm">
  <div className="fixed inset-0 flex items-center justify-center">
    <p>Custom content</p>
  </div>
</Backdrop>
```

### Click to Close

```tsx
<Backdrop open={open} onClick={() => setOpen(false)}>
  {/* Click anywhere on backdrop to close */}
</Backdrop>
```

## Accessibility

- `aria-hidden` hides the backdrop from screen readers when appropriate.
- Focus should be trapped inside modal content when used with a dialog.
- Provide keyboard Escape handling for closing overlays.
- Ensure sufficient contrast between backdrop and overlay content.

## Tips

- Use `invisible` to capture clicks without visual obstruction.
- Combine with `Paper` or other surface components for modal content.
- Use a high z‑index (e.g., `z-50`) for stacking context.
- Add `backdrop-blur` for modern frosted glass effects.
- Always provide a way to close the overlay – click backdrop, Escape key, or a button.

## Common Patterns

### Modal Dialog

```tsx
<Backdrop open={isOpen} onClick={onClose}>
  <div className="fixed inset-0 flex items-center justify-center">
    <Paper className="p-6 max-w-md">
      {/* Dialog content */}
    </Paper>
  </div>
</Backdrop>
```

### Drawer Overlay

```tsx
<Backdrop open={drawerOpen} onClick={closeDrawer}>
  <div className="fixed inset-y-0 right-0 w-80 bg-surface shadow-lg">
    {/* Drawer content */}
  </div>
</Backdrop>
```

### Loading Overlay

```tsx
<Backdrop open={isLoading}>
  <Spinner />
  <span>Loading...</span>
</Backdrop>
```
