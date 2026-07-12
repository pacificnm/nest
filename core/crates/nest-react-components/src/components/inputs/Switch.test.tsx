import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Switch } from './Switch';

describe('Switch', () => {
  it('renders unchecked by default', () => {
    render(<Switch data-testid="switch" />);
    const switchEl = screen.getByTestId('switch') as HTMLInputElement;
    expect(switchEl.checked).toBe(false);
  });

  it('renders checked when defaultChecked', () => {
    render(<Switch defaultChecked data-testid="switch" />);
    const switchEl = screen.getByTestId('switch') as HTMLInputElement;
    expect(switchEl.checked).toBe(true);
  });

  it('toggles when clicked (uncontrolled)', () => {
    render(<Switch data-testid="switch" />);
    const switchEl = screen.getByTestId('switch') as HTMLInputElement;
    fireEvent.click(switchEl);
    expect(switchEl.checked).toBe(true);
    fireEvent.click(switchEl);
    expect(switchEl.checked).toBe(false);
  });

  it('calls onChange when toggled (controlled)', () => {
    const handleChange = vi.fn();
    render(<Switch checked={false} onChange={handleChange} data-testid="switch" />);
    const switchEl = screen.getByTestId('switch') as HTMLInputElement;
    fireEvent.click(switchEl);
    expect(handleChange).toHaveBeenCalledTimes(1);
    expect(handleChange).toHaveBeenCalledWith(expect.objectContaining({
      target: expect.objectContaining({ checked: true }),
    }));
  });

  it('respects controlled checked prop', () => {
    render(<Switch checked={true} data-testid="switch" />);
    const switchEl = screen.getByTestId('switch') as HTMLInputElement;
    expect(switchEl.checked).toBe(true);
  });

  it('applies small size', () => {
    render(<Switch size="small" data-testid="switch" />);
    const switchEl = screen.getByTestId('switch');
    expect(switchEl).toHaveClass('h-4', 'w-7');
  });

  it('applies medium size (default)', () => {
    render(<Switch data-testid="switch" />);
    const switchEl = screen.getByTestId('switch');
    expect(switchEl).toHaveClass('h-6', 'w-11');
  });

  it('applies primary color (default)', () => {
    render(<Switch defaultChecked data-testid="switch" />);
    const switchEl = screen.getByTestId('switch');
    expect(switchEl.className).toContain('checked:bg-nest-primary');
  });

  it('applies secondary color', () => {
    render(<Switch defaultChecked color="secondary" data-testid="switch" />);
    const switchEl = screen.getByTestId('switch');
    expect(switchEl.className).toContain('checked:bg-nest-secondary');
  });

  it('applies disabled state', () => {
    render(<Switch disabled data-testid="switch" />);
    const switchEl = screen.getByTestId('switch') as HTMLInputElement;
    expect(switchEl).toHaveClass('disabled:opacity-50', 'disabled:cursor-not-allowed');
    expect(switchEl.disabled).toBe(true);
  });

  it('applies custom className', () => {
    render(<Switch className="custom-class" data-testid="switch" />);
    const switchEl = screen.getByTestId('switch');
    expect(switchEl).toHaveClass('custom-class');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLInputElement | null };
    render(<Switch ref={ref} data-testid="switch" />);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.type).toBe('checkbox');
  });

  it('has role="switch"', () => {
    render(<Switch data-testid="switch" />);
    const switchEl = screen.getByTestId('switch');
    expect(switchEl).toHaveAttribute('role', 'switch');
  });

  it('passes through HTML attributes', () => {
    render(<Switch name="test" aria-label="Toggle" data-testid="switch" />);
    const switchEl = screen.getByTestId('switch') as HTMLInputElement;
    expect(switchEl.name).toBe('test');
    expect(switchEl).toHaveAttribute('aria-label', 'Toggle');
  });
});
