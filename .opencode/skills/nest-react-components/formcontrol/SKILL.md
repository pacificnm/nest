---
name: nest-react-formcontrol
description: Use when working with the FormControl, FormLabel, and FormHelperText components from @nest/components – layout helpers for accessible form fields.
---

# FormControl, FormLabel, FormHelperText
Layout components for building accessible form fields with consistent styling.

## When to Use
Use these components together to create properly structured form fields:
- **FormControl**: Wraps a complete form field (label + input + helper text)
- **FormLabel**: The label that identifies what the field is for
- **FormHelperText**: Additional guidance or error messages below the field

## Components
### FormControl
Wraps form inputs and their associated labels/helper text, providing consistent spacing and state handling.
### FormLabel
A label component that supports required indicators, error states, and proper `htmlFor` association.
### FormHelperText
Helper or error text displayed below form fields.

## Props
### FormControl
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `error` | `boolean` | `false` | If true, indicates an error state |
| `disabled` | `boolean` | `false` | If true, the field is disabled |
... (content truncated)
