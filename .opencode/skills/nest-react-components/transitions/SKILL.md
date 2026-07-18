---
name: nest-react-transitions
description: Use when working with the Fade, Grow, and Collapse React components from @nest/components – animation helpers for showing and hiding content.
---

# Fade, Grow, Collapse

Transition components for showing and hiding content with animations.

## When to Use
Use transition components when:
- Revealing or hiding content dynamically
- Creating smooth state changes
- Building accordions, expandable sections, or collapsible panels
- Adding polish to UI interactions
- Improving perceived performance during content changes

## Components
### Fade
Fades content in and out using opacity.
### Grow
Grows content in and out using scale and opacity.
### Collapse
Collapses content vertically (height) or horizontally (width).

## Props
### Common Props (all components)
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `in` | `boolean` | `false` | If true, the component is shown |
| `timeout` | `number` | `300` | Duration of transition in milliseconds |
| `unmountOnExit` | `boolean` | `false` | Unmounts after exit transition |
| `className` | `string` | - | Additional CSS classes |

### Collapse Additional Props
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `orientation` | `'vertical' \| 'horizontal'` | `'vertical'` | The orientation of the collapse |

## Examples
### Fade
```tsx
import { Fade } from '@nest/components';
const [open, setOpen] = useState(false);
<Fade in={open}>
  <div>Fading content</div>
</Fade>
```
### Grow
```tsx
import { Grow } from '@nest/components';
const [open, setOpen] = useState(false);
<Grow in={open}>
  <div>Growing content</div>
</Grow>
```
### Collapse
```tsx
import { Collapse } from '@nest/components';
const [open, setOpen] = useState(false);
<Collapse in={open}>
  <div>Collapsing content</div>
</Collapse>
```
### With Custom Timeout
```tsx
import { Fade } from '@nest/components';
<Fade in={open} timeout={500}>
  <div>Slow fade (500ms)</div>
</Fade>
```
### Unmount on Exit
```tsx
import { Grow } from '@nest/components';
<Grow in={open} unmountOnExit>
  <div>Unmounted after exit transition</div>
</Grow>
```
### Horizontal Collapse
```tsx
import { Collapse } from '@nest-components';
<Collapse in={open} orientation="horizontal">
  <div>Collapses width instead of height</div>
</Collapse>
```
### Expandable Section
```tsx
import { useState } from 'react';
import { Collapse, Button } from '@nest/components';
function ExpandableSection() {
  const [open, setOpen] = useState(false);
  return (
    <div className="border rounded-lg overflow-hidden">
      <div className="p-4 bg-surface"><h3>Click to expand</h3></div>
      <Collapse in={open}>
        <div className="p-4 bg-background border-t"><p>Expanded content goes here...</p></div>
      </Collapse>
      <div className="p-2 border-t">
        <Button variant="text" size="small" onClick={() => setOpen(!open)}>{open ? 'Show less' : 'Show more'}</Button>
      </div>
    </div>
  );
}
```
### Chained Transitions
```tsx
import { Fade, Grow } from '@nest/components';
<Fade in={open}>
  <Grow in={open}>
    <div>Combined fade and grow effect</div>
  </Grow>
</Fade>
```
### Accordion Pattern
```tsx
import { useState } from 'react';
import { Collapse } from '@nest/components';
function Accordion() {
  const [expanded, setExpanded] = useState(null);
  return (
    <div>
      {items.map((item) => (
        <div key={item.id}>
          <button onClick={() => setExpanded(expanded === item.id ? null : item.id)}>{item.title}</button>
          <Collapse in={expanded === item.id}>
            <div>{item.content}</div>
          </Collapse>
        </div>
      ))}
    </div>
  );
}
```
## Accessibility
- Transitions should not be too fast (min 200ms) or too slow (max 500ms recommended).
- Respect `prefers-reduced-motion`.
- Announce content changes via ARIA when needed.
- Use `unmountOnExit` to remove hidden content from the accessibility tree.

## Tips
- Fade: simple opacity show/hide.
- Grow: for dialogs, tooltips, popovers.
- Collapse: accordions and expandable sections.
- Chain transitions for dramatic effects.
- Unmount on exit cleans DOM and improves performance.
- 300ms default works well generally.
