# Nest React Components

**Package:** [`@nest/components`](../../core/crates/nest-react-components) — the reusable
React component library for Nest desktop apps, styled with Nest design tokens + Tailwind.

**Part of the desktop frontend platform:** Tauri + React + TypeScript + Tailwind.
See [architecture](../architecture.md#desktop-frontend-platform).

Components follow MUI Material API conventions (prop names / behavior) but are restyled
with Nest design tokens. Themable values (colors, spacing, radius) come from the host at
runtime via the [`nest-react-theme`](../nest-react-theme/README.md) pipeline
(`nest_theme_css` → `:root` CSS variables); the library ships no palette of its own.

Each component below has a usage doc (When to Use, Variants, Props table, Examples,
Accessibility). Source, demo, and tests live alongside the implementation under
`core/crates/nest-react-components/src/components/<category>/`.

## Inputs

- [Autocomplete](inputs/Autocomplete.md)
- [Button](inputs/Button.md)
- [ButtonGroup](inputs/ButtonGroup.md)
- [Checkbox](inputs/Checkbox.md)
- [FormControl](inputs/FormControl.md)
- [IconButton](inputs/IconButton.md)
- [Radio](inputs/Radio.md)
- [Rating](inputs/Rating.md)
- [Select](inputs/Select.md)
- [Slider](inputs/Slider.md)
- [Switch](inputs/Switch.md)
- [TextField](inputs/TextField.md)
- [ToggleButton](inputs/ToggleButton.md)

## Feedback

- [Alert](feedback/Alert.md)
- [Backdrop](feedback/Backdrop.md)
- [CircularProgress](feedback/CircularProgress.md)
- [Dialog](feedback/Dialog.md)
- [Drawer](feedback/Drawer.md)
- [LinearProgress](feedback/LinearProgress.md)
- [Modal](feedback/Modal.md)
- [Popover](feedback/Popover.md)
- [Skeleton](feedback/Skeleton.md)
- [Snackbar](feedback/Snackbar.md)
- [Transitions](feedback/Transitions.md)

## Navigation

- [AppBar](navigation/AppBar.md)
- [Breadcrumbs](navigation/Breadcrumbs.md)
- [Link](navigation/Link.md)
- [Menu](navigation/Menu.md)
- [Pagination](navigation/Pagination.md)
- [Stepper](navigation/Stepper.md)
- [Tabs](navigation/Tabs.md)

## Surface

- [Accordion](surface/Accordion.md)
- [Card](surface/Card.md)
- [Paper](surface/Paper.md)

## Layout

- [Box](layout/Box.md)
- [Container](layout/Container.md)
- [Divider](layout/Divider.md)
- [Grid](layout/Grid.md)
- [Stack](layout/Stack.md)

## Data Display

- [Avatar](data-display/Avatar.md)
- [Badge](data-display/Badge.md)
- [Chip](data-display/Chip.md)
- [List](data-display/List.md)
- [Table](data-display/Table.md)
- [Tooltip](data-display/Tooltip.md)
- [Typography](data-display/Typography.md)

## Related

- [nest-react-theme](../nest-react-theme/README.md) — CSS var + Tailwind adapter
- [nest-design](../nest-design/README.md) — token schema
- [Port plan](../plan/nest-react-components-port-v1.md) — the MUI port execution plan
- [Theme integration plan](../plan/nest-react-components-theme-integration-v1.md)
