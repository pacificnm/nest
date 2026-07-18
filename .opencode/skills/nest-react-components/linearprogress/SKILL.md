---
name: nest-react-linearprogress
description: Use when working with the LinearProgress React component from @nest/components – an animated bar for indeterminate or determinate progress.
---

# LinearProgress

A linear progress indicator component showing either indeterminate (animated) or determinate (percentage) progress.

## When to Use

Use `LinearProgress` for:
- Page loading indicators (indeterminate)
- File upload/download progress (determinate)
- Form submission progress (determinate)
- Multi‑step process progress (determinate)
- Buffer/loading ahead (buffer variant)

## Variants

| Variant | Description |
|---------|-------------|
| `indeterminate` (default) | Continuous left-to-right animation |
| `determinate` | Shows progress percentage (0‑100) |
| `buffer` | Shows progress with buffer ahead |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'span'` | HTML element to render as |
| `variant` | `'indeterminate' \| 'determinate' \| 'buffer'` | `'indeterminate'` | Progress type |
| `color` | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info' \| 'inherit'` | `'primary'` | Color of the indicator |
| `value` | `number` | `0` | Progress value (0‑100) |
| `bufferValue` | `number` | `0` | Buffer value (0‑100) for buffer variant |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Indeterminate (Default)

```tsx
import { LinearProgress } from '@nest/components';

// Page loading
<LinearProgress />
```

### Determinate

```tsx
<LinearProgress variant="determinate" value={50} />
```

### Buffer

```tsx
<LinearProgress variant="buffer" value={50} bufferValue={75} />
```

### Color Variants

```tsx
<LinearProgress color="primary" />
<LinearProgress color="secondary" />
<LinearProgress color="success" />
<LinearProgress color="warning" />
<LinearProgress color="error" />
<LinearProgress color="info" />
// Inherit parent color
<div className="text-nest-success"
  <LinearProgress color="inherit" />
</div>
```

### File Upload

```tsx
function FileUpload({ progress }) {
  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <s<span>Uploading file.txt</span>
        <s span>{progress}%)>
      </div>
      <LinearProgress variant="determinate" value={progress} />
    </div>
  );
}
```

### Multiple Downloads

```tsx
<div className="space-y-2"
  {downloads.map(file => (
    <div key={file.id}>
      <div className="flex justify-between text-xs mb-1">
        <s<span>{file.name}</span>
        <s span>{file.progress}%>
      </div>
      <LinearProgress variant="determinate" value={file.progress}
        color={file.progress === 100 ? 'success' : 'primary'} />
    </div>
  ))}
</div>
```

### Custom Height

```tsx
<LinearProgress />
<LinearProgress className="h-2" />
<LinearProgress className="h-4" />
```

### Page Top Loading

```tsx
<div className="fixed top-0 left-0 right-0 z-50"
  <LinearProgress />
</div>
```

### Card Loading

```tsx
<Card>
  <LinearProgress />
  <CardContent>
    <p>Card content loading...</p>
  </CardContent>
</Card>
```

## Accessibility

- `role="progressbar"` applied automatically.
- `aria-valuenow` set for determinate and buffer variants.
- Indeterminate progress has no `aria-valuenow`.
- Add `aria-label` for context (e.g., "File upload progress").
```tsx
<LinearProgress aria-label="Page loading" />
<LinearProgress variant="determinate" value={50} aria-label="Upload progress" />
```

## Styling

- Uses `h-1 w-full rounded-nest-full` for default size.
- Track: `bg-nest-surface`, fill: `bg-nest-primary` (or color variant).
- Transition: `transition-all duration-300 ease-out`.
- Custom keyframes animate indeterminate state.

### Custom Styling

```tsx
<LinearProgress className="h-2" />
<LinearProgress className="[&>div]:bg-purple-500" />
<LinearProgress className="rounded-none" />
```

## Buffer Variant

Shows two layers: buffer ahead and actual progress, useful for streaming or download indications.
```tsx
<LinearProgress variant="buffer" value={40} bufferValue={80} />
```

## Best Practices

1. Use indeterminate when duration unknown.
2. Use determinate for measurable tasks.
3. Show percentage alongside the bar.
4. Pick colors that match status (success, warning).
5. Position appropriately: top of page for global loading, inline for items.
