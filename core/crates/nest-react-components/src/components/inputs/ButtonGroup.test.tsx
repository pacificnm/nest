import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ButtonGroup } from './ButtonGroup';
import { Button } from './Button';

describe('ButtonGroup', () => {
  it('renders children buttons', () => {
    render(
      <ButtonGroup data-testid="group">
        <Button>One</Button>
        <Button>Two</Button>
      </ButtonGroup>
    );
    expect(screen.getByText('One')).toBeInTheDocument();
    expect(screen.getByText('Two')).toBeInTheDocument();
  });

  it('applies variant styles', () => {
    render(
      <ButtonGroup variant="outlined" data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group).toHaveClass('border', 'border-nest-border');
  });

  it('applies color styles', () => {
    render(
      <ButtonGroup color="secondary" data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group.className).toContain('[&>button:not(:disabled)]:bg-nest-secondary');
  });

  it('applies size styles', () => {
    render(
      <ButtonGroup size="small" data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group.className).toContain('[&>button]:text-xs');
  });

  it('applies vertical orientation', () => {
    render(
      <ButtonGroup orientation="vertical" data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group).toHaveClass('flex-col');
  });

  it('applies horizontal orientation (default)', () => {
    render(
      <ButtonGroup data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group).toHaveClass('flex-row');
  });

  it('applies fullWidth', () => {
    render(
      <ButtonGroup fullWidth data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group).toHaveClass('w-full');
  });

  it('renders custom component', () => {
    render(
      <ButtonGroup component="section" data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group.tagName).toBe('SECTION');
  });

  it('applies custom className', () => {
    render(
      <ButtonGroup className="custom-class" data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group).toHaveClass('custom-class');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <ButtonGroup ref={ref} data-testid="group">
        <Button>Test</Button>
      </ButtonGroup>
    );
    expect(ref.current).toBeInTheDocument();
  });

  it('applies border stripping classes', () => {
    render(
      <ButtonGroup data-testid="group">
        <Button>First</Button>
        <Button>Last</Button>
      </ButtonGroup>
    );
    const group = screen.getByTestId('group');
    expect(group.className).toContain('[&>button]:rounded-none');
    expect(group.className).toContain('[&>button:first-child]:rounded-tl-nest-md');
  });
});
