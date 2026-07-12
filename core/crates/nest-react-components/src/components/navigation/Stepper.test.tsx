import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Stepper, Step, StepLabel, StepIcon } from './Stepper';

describe('Stepper', () => {
  it('renders stepper with steps', () => {
    render(
      <Stepper activeStep={1}>
        <Step data-testid="step1"><StepLabel>Step 1</StepLabel></Step>
        <Step data-testid="step2"><StepLabel>Step 2</StepLabel></Step>
        <Step data-testid="step3"><StepLabel>Step 3</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByText('Step 1')).toBeInTheDocument();
    expect(screen.getByText('Step 2')).toBeInTheDocument();
    expect(screen.getByText('Step 3')).toBeInTheDocument();
  });

  it('applies horizontal orientation (default)', () => {
    render(
      <Stepper activeStep={1} data-testid="stepper">
        <Step><StepLabel>Step</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByTestId('stepper')).toHaveClass('flex-row');
  });

  it('applies vertical orientation', () => {
    render(
      <Stepper activeStep={1} orientation="vertical" data-testid="stepper">
        <Step><StepLabel>Step</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByTestId('stepper')).toHaveClass('flex-col');
  });

  it('has progressbar role', () => {
    render(
      <Stepper activeStep={1} data-testid="stepper">
        <Step><StepLabel>Step</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByTestId('stepper')).toHaveAttribute('role', 'progressbar');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Stepper ref={ref} activeStep={1}>
        <Step><StepLabel>Step</StepLabel></Step>
      </Stepper>
    );
    expect(ref.current).toBeInTheDocument();
  });
});

describe('Step', () => {
  it('renders step content', () => {
    render(
      <Stepper activeStep={1}>
        <Step data-testid="step"><StepLabel>Label</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByTestId('step')).toBeInTheDocument();
  });

  it('applies completed state', () => {
    render(
      <Stepper activeStep={2}>
        <Step completed data-testid="step"><StepLabel completed>Label</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByTestId('step')).toBeInTheDocument();
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Stepper activeStep={1}>
        <Step ref={ref} data-testid="step"><StepLabel>Label</StepLabel></Step>
      </Stepper>
    );
    expect(ref.current).toBeInTheDocument();
  });
});

describe('StepLabel', () => {
  it('renders label text', () => {
    render(
      <Stepper activeStep={0}>
        <Step><StepLabel>Test Label</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByText('Test Label')).toBeInTheDocument();
  });

  it('applies active styles', () => {
    render(
      <Stepper activeStep={0}>
        <Step><StepLabel active>Active Label</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByText('Active Label')).toHaveClass('text-nest-primary');
  });

  it('applies completed styles', () => {
    render(
      <Stepper activeStep={1}>
        <Step completed><StepLabel completed>Completed Label</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByText('Completed Label')).toHaveClass('text-nest-foreground');
  });

  it('applies disabled styles', () => {
    render(
      <Stepper activeStep={0}>
        <Step disabled><StepLabel disabled data-testid="label">Disabled Label</StepLabel></Step>
      </Stepper>
    );
    expect(screen.getByTestId('label')).toHaveClass('opacity-50');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Stepper activeStep={0}>
        <Step><StepLabel ref={ref}>Label</StepLabel></Step>
      </Stepper>
    );
    expect(ref.current).toBeInTheDocument();
  });
});

describe('StepIcon', () => {
  it('renders completed icon with check', () => {
    const { container } = render(<StepIcon completed />);
    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('renders active state', () => {
    render(<StepIcon active data-testid="icon" />);
    expect(screen.getByTestId('icon')).toHaveClass('border-nest-primary', 'text-nest-primary');
  });

  it('renders inactive state', () => {
    render(<StepIcon data-testid="icon" />);
    expect(screen.getByTestId('icon')).toHaveClass('border-nest-border', 'text-nest-muted');
  });

  it('renders completed state', () => {
    render(<StepIcon completed data-testid="icon" />);
    expect(screen.getByTestId('icon')).toHaveClass('bg-nest-primary', 'border-nest-primary', 'text-white');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<StepIcon ref={ref} data-testid="icon" />);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('Stepper integration', () => {
  it('shows correct step states', () => {
    render(
      <Stepper activeStep={1}>
        <Step completed data-testid="step1"><StepLabel completed>Step 1</StepLabel></Step>
        <Step active data-testid="step2"><StepLabel active>Step 2</StepLabel></Step>
        <Step data-testid="step3"><StepLabel>Step 3</StepLabel></Step>
      </Stepper>
    );
    const step1Label = screen.getByText('Step 1');
    const step2Label = screen.getByText('Step 2');
    const step3Label = screen.getByText('Step 3');

    expect(step1Label).toHaveClass('text-nest-foreground');
    expect(step2Label).toHaveClass('text-nest-primary');
    expect(step3Label).toHaveClass('text-nest-muted');
  });
});
