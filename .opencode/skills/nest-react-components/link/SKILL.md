---
name: nest-react-link
description: Use when working with the Link component from @nest/components – styled anchor or navigable button.
---

# Link
A styled anchor element component for navigation and external links.

## When to Use
Use `Link` for:
- Navigation links within the app
- External website links
- Inline text links
- Breadcrumb navigation
- Button‑like links (with custom component)
- Any clickable text that navigates somewhere

## Variants
| Underline | Description |
|-----------|-------------|
| `hover` (default) | Underline appears on hover |
| `none` | No underline |
| `always` | Always underlined |

## Props
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'a'` | HTML element to render as |
| `href` | `string` | - | URL for the link |
| `underline` | `'none' \| 'hover' \| 'always'` | `'hover'` | Underline behavior |
| `color` | `'primary' \| 'inherit'` | `'primary'` | Link color |
| `external` | `boolean` | `false` | Open in new tab with secure rel |
| `className` | `string` | - | Additional CSS classes |

## Examples
### Basic Link
```tsx
import { Link } from '@nest/components';

<Link href="/page"\u003eClick me</Link>
```
### Underline Variants
```tsx
<Link href="/page" underline="hover"\u003eHover to underline</Link>
<Link href="/page" underline="none"\u003eNo underline</Link>
<Link href="/page" underline="always"\u003eAlways underlined</Link>
```
### Color Variants
```tsx
<Link href="/page" color="primary"\u003ePrimary colored link</Link>
<Link href="/page" color="inherit"\u003eInherits parent color</Link>
```
### External Link
```tsx
<Link href="https://example.com" external\u003eExternal Link</Link\u003e
// Renders with target="_blank" and rel="noopener noreferrer"
```
### Inline Link
```tsx
<p\u003e
  Check out our <Link href="/docs">documentation</Link> for more information.
</p\u003e
```
### Link in Muted Text
```tsx
<p className="text-nest-muted"\u003e
  Need help? <Link href="/help" color="inherit">Contact support</Link>
</p\u003e
```
### Navigation Links
```tsx
<nav className="flex gap-4"\u003e
  <Link href="/" underline="none">Home</Link>
  <Link href="/about" underline="none">About</Link>
  <Link href="/contact" underline="none">Contact</Link>
</nav\u003e
```
### Breadcrumb Links
```tsx
<div className="flex items-center gap-2"\u003e
  <Link href="/">Home</Link>
  <span\u003e/\u003c/span\u003e
  <Link href="/products">Products</Link>
  <span\u003e/\u003c/span\u003e
  <span\u003eCurrent Page</span\u003e
</div\u003e
```
### Custom Component
```tsx
// As a button
<Link component="button" onClick={handleClick}\u003eButton Link</Link\u003e
// As a span (for non‑navigable clickable text)
<Link component="span" onClick={handleClick}\u003eClickable Text</Link\u003e
// With React Router
<Link component={RouterLink} to="/page"\u003eRouter Link</Link\u003e
```
### Link with Icon
```tsx
<Link href="/docs" className="inline-flex items-center gap-1"\u003e
  <DocumentIcon /\u003e
  <span\u003eDocumentation\u003c/span\u003e
</Link\u003e

<Link href="/external" external className="inline-flex items-center gap-1"\u003e
  <span\u003eExternal Resource\u003c/span\u003e
  <ExternalLinkIcon /\u003e
</Link\u003e
```
## Accessibility
- Always provide meaningful `href` for navigation links.
- Use `external` prop for external links (adds `rel="noopener noreferrer").
- Add `aria-label` for icon‑only links or ambiguous link text.
- Ensure link text is descriptive (e.g., "Read more about X" vs "Click here").
- Focus ring is applied for keyboard navigation.
## Styling
Link uses these base styles:
- `cursor-pointer` – Pointer cursor
- `font-nest-body` – Body font family
- `transition-colors duration-150` – Smooth color transitions
- `focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2` – Focus ring
- `rounded-nest-sm` – Rounded focus ring
### Color Tokens
| Color | Classes |
|-------|---------|
| `primary` | `text-nest-primary hover:text-nest-primary/80` |
| `inherit` | `text-inherit hover:text-inherit/80` |
## Custom Styling
```tsx
<Link href="/page" className="font-bold text-lg"\u003eBold Large Link</Link\u003e

<Link href="/page" className="flex items-center gap-2"\u003e
  <Icon /\u003e
  <span\u003eLink with Icon\u003c/span\u003e
</Link\u003e
```
## Best Practices
1. Use `external` for external links – automatically adds security attributes.
2. Choose underline wisely – `hover` for navigation, `none` for nav bars, `always` for emphasis.
3. Use `color="inherit"` in colored text – maintains visual hierarchy.
4. Provide descriptive link text – better accessibility and SEO.
5. Consider custom component for routing – works with React Router, Next.js Link, etc.
