---
name: nest-react-accordion
description: Use when working with the Accordion component family (@nest/components) – collapsible panels for FAQs, settings, etc.
---

# Accordion, AccordionItem, AccordionSummary, AccordionDetails
A collection of collapsible panels that allow users to show/hide sections of content.

## When to Use
- Too much content to display at once
- Users need multiple sections while working
- FAQs, settings, step instructions
- Reduce visual clutter while keeping content accessible

## Props
### Accordion
| Prop             | Type                    | Default | Description |
|------------------|-------------------------|---------|-------------|
| `expanded`      | `string \| string[]`    | —       | Expanded panel value(s) (controlled) |
| `defaultExpanded`| `string \| string[]`   | —       | Default expanded panel(s) (uncontrolled) |
| `onChange`      | `(value)=>void`         | —       | Callback on change |
| `disabled`      | `boolean`               | `false` | All panels disabled |
| `exclusive`     | `boolean`               | `false` | Only one panel expanded |
| `className`     | `string`                | —       | Extra CSS |

### AccordionItem
| Prop      | Type    | Default | Description |
|-----------|---------|---------|-------------|
| `value`   | `string`| —       | Unique value |
| `summary` | `ReactNode`| —     | Header content |
| `disabled`| `boolean`| `false`| Disabled state |
| `className`| `string`| —    | Extra CSS |

### AccordionSummary
| Prop      | Type          | Default | Description |
|-----------|---------------|---------|-------------|
| `expanded`| `boolean`     | —       | Expanded? |
| `disabled`| `boolean`     | `false` | Disabled |
| `onClick` | `()=&#62;void` | —      | Click handler |
| `className`| `string`     | —      | Extra CSS |

### AccordionDetails
| Prop      | Type    | Default | Description |
|-----------|---------|---------|-------------|
| `expanded`| `boolean`| —       | Expanded? |
| `className`| `string`| —      | Extra CSS |

## Examples (basic snippets)
**Basic Usage**
```tsx
import { Accordion, AccordionItem } from '@nest/components';

<Accordion defaultExpanded="panel1">
  <AccordionItem value="panel1" summary="Panel 1">
    <p>Content for panel 1</p>
  </AccordionItem>
  <AccordionItem value="panel2" summary="Panel 2">
    <p>Content for panel 2</p>
  </AccordionItem>
</Accordion>
```
**Exclusive (single expanded)**
```tsx
import { useState } from 'react';
import { Accordion, AccordionItem } from '@nest/components';

const [expanded, setExpanded] = useState('panel1');

<Accordion expanded={expanded} onChange={(v)=>setExpanded(v)} exclusive>
  ...
</Accordion>
```
**Multiple Expanded** (illustrative)
```tsx
import { useState } from 'react';
const [expanded, setExpanded] = useState<string[]>(['panel1']);
```
## Accessibility & Keyboard
- Uses `aria-expanded` and proper roles.
- Keyboard: Tab, Enter/Space toggle; Arrow keys navigate panels.
