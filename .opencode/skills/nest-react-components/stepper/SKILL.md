---
name: nest-react-stepper
description: Use when working with the Stepper component from @nest/components – multi‑step progress UI.
---

# Stepper, Step, StepLabel, StepIcon
A component for displaying progress through a sequence of steps.

## When to Use
- Users need to complete a multi‑step process (wizard, onboarding)
- Showing progress through a timeline or workflow
- Breaking complex tasks into manageable chunks
- Users need to understand where they are in a process

## Props
### Stepper
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `activeStep` | `number` | `0` | The currently active step (0-indexed) |
| `orientation` | `'horizontal' \| 'vertical'` | `'horizontal'` | The orientation of the stepper |
| `className` | `string` | — | Additional CSS classes |

### Step
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `completed` | `boolean` | — | If true, the step is completed |
| `disabled` | `boolean` | `false` | If true, the step is disabled |
| `className` | `string` | — | Additional CSS classes |

### StepLabel
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `completed` | `boolean` | — | If true, shows completed state |
| `active` | `boolean` | — | If true, shows active state |
| `disabled` | `boolean` | `false` | If true, the label is disabled |
| `className` | `string` | — | Additional CSS classes |

### StepIcon
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `completed` | `boolean` | — | If true, shows check icon |
| `active` | `boolean` | — | If true, shows active styling |
| `disabled` | `boolean` | `false` | If true, the icon is disabled |

## Examples
**Basic Usage**
```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

<Stepper activeStep={1}>
  <Step>
    <StepLabel>Step 1</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 2</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 3</StepLabel>
  </Step>
</Stepper>
```
**With Completed Steps**
```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

<Stepper activeStep={2}>
  <Step completed>
    <StepLabel completed>Account Created</StepLabel>
  </Step>
  <Step completed>
    <StepLabel completed>Email Verified</StepLabel>
  </Step>
  <Step active>
    <StepLabel active>Profile Setup</StepLabel>
  </Step>
  <Step>
    <StepLabel>Preferences</StepLabel>
  </Step>
</Stepper>
```
**Vertical Orientation**
```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

<Stepper activeStep={1} orientation="vertical">
  <Step>
    <StepLabel>Step 1</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 2</StepLabel>
  </Step>
  <Step>
    <StepLabel>Step 3</StepLabel>
  </Step>
</Stepper>
```
**Interactive Wizard (simplified)**
```tsx
import { useState } from 'react';
import { Stepper, Step, StepLabel, Button } from '@nest/components';

function Wizard() {
  const [activeStep, setActiveStep] = useState(0);
  const steps = ['Select Plan', 'Enter Details', 'Payment', 'Complete'];

  const handleNext = () => setActiveStep((prev) => prev + 1);
  const handleBack = () => setActiveStep((prev) => prev - 1);

  return (
    <div>
      <Stepper activeStep={activeStep}>
        {steps.map((label) => (
          <Step key={label}> <StepLabel>{label}</StepLabel> </Step>
        ))}
      </Stepper>
      <div className="mt-4 flex gap-2">
        <Button onClick={handleBack} disabled={activeStep === 0}>Back</Button>
        <Button onClick={handleNext} disabled={activeStep === steps.length - 1}> {activeStep === steps.length - 2 ? 'Finish' : 'Next'} </Button>
      </div>
    </div>
  );
}
```
**Onboarding Flow (simplified)**
```tsx
import { Stepper, Step, StepLabel } from '@nest/components';

function OnboardingFlow() {
  const currentStep = 0; // placeholder
}
```
## Accessibility
- Stepper uses `role="progressbar"` with `aria-valuenow` for the current step
- Each step should have a meaningful label
- Active step is visually highlighted
- Provide text alternatives for the indicator where appropriate
## Tips
- Use `activeStep` to control the flow
- Mark `completed` on Step/StepLabel to show checkmark
- For linear progress, disable navigation to incomplete steps
- For non‑linear, allow jumping
- Adjust orientation for layout needs
