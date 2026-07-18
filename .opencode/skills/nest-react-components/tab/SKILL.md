---
name: nest-react-tabs
description: Use when working with the Tabs component family (@nest/components) – organizes content into switchable views.
---

# Tabs, Tab, TabPanel
A container for organizing content into multiple sections that can be switched between.

## When to Use
- You have related content that should be organized into separate views
- Users need to switch between views frequently
- Reduce scrolling by grouping content logically
- Settings pages, dashboards, or data views

## Props
### Tabs
| Prop          | Type                     | Default      | Description |
|---------------|--------------------------|--------------|-------------|
| `value`       | `string \| number`       | —            | Selected tab value (controlled) |
| `defaultValue`| `string \| number`       | —            | Default selected tab (uncontrolled) |
| `onChange`    | `(value)=>void`          | —            | Callback when tab changes |
| `variant`     | `'standard' \| 'fullWidth'` | `'standard'` | Tab variant |
| `orientation` | `'horizontal' \| 'vertical'` | `'horizontal'` | Orientation |
| `className`   | `string`                 | —            | Extra CSS |

### Tab
| Prop      | Type                | Default | Description |
|-----------|---------------------|---------|-------------|
| `value`   | `string \| number`  | —       | Tab value |
| `label`   | `ReactNode`         | —       | Label displayed |
| `icon`    | `ReactNode`         | —       | Icon before label |
| `disabled`| `boolean`           | `false` | Disabled state |
| `className`| `string`            | —        | Extra CSS |

### TabPanel
| Prop      | Type                | Default | Description |
|-----------|---------------------|---------|-------------|
| `value`   | `string \| number`  | —       | Corresponding tab value |
| `className`| `string`           | —        | Extra CSS |

## Examples
**Basic Usage**
```tsx
import { Tabs, Tab } from '@nest/components';

<Tabs defaultValue="one">
  <Tab value="one" label="Tab One" />
  <Tab value="two" label="Tab Two" />
  <Tab value="three" label="Tab Three" />
</Tabs>
```
**With Panels (simplified)**
```tsx
import { Tabs, Tab, TabPanel } from '@nest/components';

<Tabs defaultValue="overview">
  <Tab value="overview" label="Overview" />
  <Tab value="features" label="Features" />
  <Tab value="pricing" label="Pricing" />
  <TabPanel value="overview"><p>Overview content</p></TabPanel>
  <TabPanel value="features"><p>Features content</p></TabPanel>
  <TabPanel value="pricing"><p>Pricing content</p></TabPanel>
</Tabs>
```
**Controlled (simplified)**
```tsx
import { useState } from 'react';
import { Tabs, Tab, TabPanel } from '@nest/components';

function Demo() {
  const [value, setValue] = useState('one');
  return (
    <Tabs value={value} onChange={(v)=>setValue(v)}>
      <Tab value="one" label="First" />
      <Tab value="two" label="Second" />
      <TabPanel value="one">First panel</TabPanel>
      <TabPanel value="two">Second panel</TabPanel>
    </Tabs>
  );
}
```
**With Icons (simplified)**
```tsx
import { Tabs, Tab } from '@nest/components';
import { Settings, User, Bell } from 'lucide-react';

<Tabs defaultValue="settings">
  <Tab value="settings" icon={<Settings className="size-4" />} label="Settings" />
  <Tab value="profile" icon={<User className="size-4" />} label="Profile" />
  <Tab label="Notifications" />
</Tabs>
```
**Full Width & Orientation (illustrative)**
```tsx
import { Tabs, Tab } from '@nest/components';

<Tabs variant="fullWidth">
   <Tab value="first" label="First" />
   ...
</Tabs>
```
## Accessibility
- Container uses `role="tablist"
- Each tab uses role = `&quot;..` (not fully displayed due to length)
