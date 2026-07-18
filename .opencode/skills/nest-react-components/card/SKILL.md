---
name: nest-react-card
description: Use when working with the Card component family (@nest/components). Offers container, header, media, content and actions sections.
---

# Card
A container component for grouping related content with optional header, media, content, and actions sections.

## When to Use
- Article or blog previews
- Product cards
- User profile widgets
- Dashboard widgets
- Media galleries
- Form containers
- Any grouped content needing visual separation

## Props
### Card (extends Paper props)
| Prop      | Type                      | Default | Description |
|-----------|--------------------------|---------|-------------|
| `component` | `ElementType`            | `'div'` | Rendered element |
| `elevation`  | `0\|1\|2\|3\|4`       | `1`     | Shadow depth |
| `variant`    | `'elevation'\|'outlined'` | `'elevation'` | Visual style |
| `className` | `string`                 | —       | CSS classes |

### CardHeader
| Prop      | Type        | Default | Description |
|-----------|-------------|---------|-------------|
| `component` | `ElementType`| `'div'` | Rendered element |
| `avatar`    | `ReactNode`| —      | Avatar before title |
| `action`    | `ReactNode`| —      | Action after title |
| `title`     | `ReactNode`| —      | Title content |
| `subheader` | `ReactNode`| —      | Subtitle |
| `className` | `string`   | —      | CSS classes |

### CardContent, CardActions, CardMedia …
— (prop tables omitted for brevity; see source)

## Examples (selected snippets)
**Basic Card**
```tsx
import { Card, CardContent } from '@nest/components';

<Card>
  <CardContent>
    <p>Simple card content</p>
  </CardContent>
</Card>
```
**Header & Actions**
```tsx
<Card>
  <CardHeader title="Title" subheader="Subtitle" />
  <CardContent>Body text</CardContent>
  <CardActions>
    <Button>Learn More</Button>
  </CardActions>
</Card>
```
**With Media**
```tsx
<Card>
  <CardMedia image="/img.jpg" alt="thumb" />
  <CardHeader title="Article" />
  <CardContent>excerpt</CardContent>
  <CardActions><Button size="small">Read More</Button></CardActions>
</Card>
```
**Clickable Card**
```tsx
<Card component="button" className="cursor-pointer hover:shadow-md">
  <CardHeader title="Clickable" />
  <CardContent>Click anywhere</CardContent>
</Card>
```
## Accessibility & Patterns
- Use semantic `component` values (`article`, `section`).
- When `component="button"`, wrap entire content.
- Provide `aria-label` on header actions.
- Ensure meaningful `alt` for images and sufficient contrast.
