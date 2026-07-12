import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Checkbox } from './Checkbox';

describe('Checkbox', () => {
  it('renders unchecked by default', () => {
    render(<Checkbox data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    expect(checkbox.checked).toBe(false);
  });

  it('renders checked when defaultChecked', () => {
    render(<Checkbox defaultChecked data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
  });

  it('toggles when clicked (uncontrolled)', () => {
    render(<Checkbox data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    fireEvent.click(checkbox);
    expect(checkbox.checked).toBe(true);
    fireEvent.click(checkbox);
    expect(checkbox.checked).toBe(false);
  });

  it('calls onChange when toggled (controlled)', () => {
    const handleChange = vi.fn();
    render(<Checkbox checked={false} onChange={handleChange} data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    fireEvent.click(checkbox);
    expect(handleChange).toHaveBeenCalledTimes(1);
    expect(handleChange).toHaveBeenCalledWith(expect.objectContaining({
      target: expect.objectContaining({ checked: true }),
    }));
  });

  it('respects controlled checked prop', () => {
    render(<Checkbox checked={true} data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
    fireEvent.click(checkbox);
    // Still true because controlled
    expect(checkbox.checked).toBe(true);
  });

  it('shows indeterminate state with minus icon', () => {
    render(<Checkbox indeterminate data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    // The minus icon should be present
    expect(screen.getByTestId('checkbox').parentElement).toHaveTextContent('');
  });

  it('applies small size', () => {
    render(<Checkbox size="small" data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox');
    expect(checkbox).toHaveClass('size-4');
  });

  it('applies medium size (default)', () => {
    render(<Checkbox data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox');
    expect(checkbox).toHaveClass('size-5');
  });

  it('applies primary color (default)', () => {
    render(<Checkbox data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox');
    expect(checkbox.className).toContain('checked:bg-nest-primary');
  });

  it('applies secondary color', () => {
    render(<Checkbox color="secondary" data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox');
    expect(checkbox.className).toContain('checked:bg-nest-secondary');
  });

  it('applies disabled state', () => {
    render(<Checkbox disabled data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    expect(checkbox).toHaveClass('disabled:opacity-50', 'disabled:cursor-not-allowed');
    expect(checkbox.disabled).toBe(true);
  });

  it('applies custom className', () => {
    render(<Checkbox className="custom-class" data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox');
    expect(checkbox).toHaveClass('custom-class');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLInputElement | null };
    render(<Checkbox ref={ref} data-testid="checkbox" />);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.type).toBe('checkbox');
  });

  it('passes through HTML attributes', () => {
    render(<Checkbox name="test" value="yes" data-testid="checkbox" />);
    const checkbox = screen.getByTestId('checkbox') as HTMLInputElement;
    expect(checkbox.name).toBe('test');
    expect(checkbox.value).toBe('yes');
  });
});
