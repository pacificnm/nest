import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ToggleButton, ToggleButtonGroup } from './ToggleButton';

describe('ToggleButton', () => {
  it('renders content', () => {
    render(
      <ToggleButtonGroup>
        <ToggleButton value="test" label="Test Label" />
      </ToggleButtonGroup>
    );
    expect(screen.getByText('Test Label')).toBeInTheDocument();
  });

  it('renders children', () => {
    render(
      <ToggleButtonGroup>
        <ToggleButton value="test">
          <span data-testid="icon">★</span>
        </ToggleButton>
      </ToggleButtonGroup>
    );
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('applies small size', () => {
    render(
      <ToggleButtonGroup size="small">
        <ToggleButton value="test" label="Test" data-testid="button" />
      </ToggleButtonGroup>
    );
    expect(screen.getByTestId('button')).toHaveClass('text-xs', 'px-2', 'py-1');
  });

  it('applies medium size (default)', () => {
    render(
      <ToggleButtonGroup>
        <ToggleButton value="test" label="Test" data-testid="button" />
      </ToggleButtonGroup>
    );
    expect(screen.getByTestId('button')).toHaveClass('text-sm', 'px-3', 'py-1.5');
  });

  it('applies large size', () => {
    render(
      <ToggleButtonGroup size="large">
        <ToggleButton value="test" label="Test" data-testid="button" />
      </ToggleButtonGroup>
    );
    expect(screen.getByTestId('button')).toHaveClass('text-base', 'px-4', 'py-2');
  });

  it('applies disabled state', () => {
    render(
      <ToggleButtonGroup>
        <ToggleButton value="test" label="Test" disabled data-testid="button" />
      </ToggleButtonGroup>
    );
    const button = screen.getByTestId('button');
    expect(button).toHaveClass('opacity-50', 'cursor-not-allowed');
    expect(button).toBeDisabled();
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLButtonElement | null };
    render(
      <ToggleButtonGroup>
        <ToggleButton value="test" ref={ref} data-testid="button" />
      </ToggleButtonGroup>
    );
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('BUTTON');
  });
});

describe('ToggleButtonGroup', () => {
  it('renders children buttons', () => {
    render(
      <ToggleButtonGroup>
        <ToggleButton value="one" label="One" />
        <ToggleButton value="two" label="Two" />
      </ToggleButtonGroup>
    );
    expect(screen.getByText('One')).toBeInTheDocument();
    expect(screen.getByText('Two')).toBeInTheDocument();
  });

  it('manages single selection (exclusive)', () => {
    const handleChange = vi.fn();
    render(
      <ToggleButtonGroup value="one" onChange={handleChange} exclusive>
        <ToggleButton value="one" label="One" data-testid="btn-one" />
        <ToggleButton value="two" label="Two" data-testid="btn-two" />
      </ToggleButtonGroup>
    );
    const btnOne = screen.getByTestId('btn-one');
    const btnTwo = screen.getByTestId('btn-two');
    expect(btnOne).toHaveAttribute('aria-pressed', 'true');
    expect(btnTwo).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(btnTwo);
    expect(handleChange).toHaveBeenCalledWith('two');
  });

  it('manages multiple selection (non-exclusive)', () => {
    const handleChange = vi.fn();
    render(
      <ToggleButtonGroup value={['one']} onChange={handleChange} exclusive={false}>
        <ToggleButton value="one" label="One" data-testid="btn-one" />
        <ToggleButton value="two" label="Two" data-testid="btn-two" />
      </ToggleButtonGroup>
    );
    const btnOne = screen.getByTestId('btn-one');
    const btnTwo = screen.getByTestId('btn-two');
    expect(btnOne).toHaveAttribute('aria-pressed', 'true');
    expect(btnTwo).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(btnTwo);
    expect(handleChange).toHaveBeenCalledWith(['one', 'two']);
  });

  it('deselects when clicking selected button (exclusive)', () => {
    const handleChange = vi.fn();
    render(
      <ToggleButtonGroup value="one" onChange={handleChange} exclusive>
        <ToggleButton value="one" label="One" data-testid="btn-one" />
        <ToggleButton value="two" label="Two" data-testid="btn-two" />
      </ToggleButtonGroup>
    );
    fireEvent.click(screen.getByTestId('btn-one'));
    expect(handleChange).toHaveBeenCalledWith('');
  });

  it('removes from selection when clicking selected button (non-exclusive)', () => {
    const handleChange = vi.fn();
    render(
      <ToggleButtonGroup value={['one', 'two']} onChange={handleChange} exclusive={false}>
        <ToggleButton value="one" label="One" data-testid="btn-one" />
        <ToggleButton value="two" label="Two" data-testid="btn-two" />
      </ToggleButtonGroup>
    );
    fireEvent.click(screen.getByTestId('btn-one'));
    expect(handleChange).toHaveBeenCalledWith(['two']);
  });

  it('applies row layout', () => {
    render(
      <ToggleButtonGroup row data-testid="group">
        <ToggleButton value="one" />
      </ToggleButtonGroup>
    );
    expect(screen.getByTestId('group')).toHaveClass('flex-row');
  });

  it('applies column layout (default)', () => {
    render(
      <ToggleButtonGroup data-testid="group">
        <ToggleButton value="one" />
      </ToggleButtonGroup>
    );
    expect(screen.getByTestId('group')).toHaveClass('flex-col');
  });

  it('applies disabled to group', () => {
    render(
      <ToggleButtonGroup disabled data-testid="group">
        <ToggleButton value="one" label="One" data-testid="btn-one" />
        <ToggleButton value="two" label="Two" data-testid="btn-two" />
      </ToggleButtonGroup>
    );
    expect(screen.getByTestId('btn-one')).toHaveClass('opacity-50');
    expect(screen.getByTestId('btn-two')).toHaveClass('opacity-50');
  });

  it('applies color to selected button', () => {
    render(
      <ToggleButtonGroup value="one" color="error" exclusive>
        <ToggleButton value="one" label="One" data-testid="btn-one" />
      </ToggleButtonGroup>
    );
    const btn = screen.getByTestId('btn-one');
    expect(btn.className).toContain('text-nest-error');
    expect(btn.className).toContain('border-nest-error');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <ToggleButtonGroup ref={ref} data-testid="group">
        <ToggleButton value="one" />
      </ToggleButtonGroup>
    );
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.role).toBe('group');
  });

  it('uncontrolled mode works', () => {
    render(
      <ToggleButtonGroup defaultValue="one" exclusive>
        <ToggleButton value="one" label="One" data-testid="btn-one" />
        <ToggleButton value="two" label="Two" data-testid="btn-two" />
      </ToggleButtonGroup>
    );
    const btnOne = screen.getByTestId('btn-one');
    const btnTwo = screen.getByTestId('btn-two');
    expect(btnOne).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(btnTwo);
    expect(btnTwo).toHaveAttribute('aria-pressed', 'true');
    expect(btnOne).toHaveAttribute('aria-pressed', 'false');
  });
});
