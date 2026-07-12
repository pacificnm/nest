import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Radio, RadioGroup } from './Radio';

describe('Radio', () => {
  it('renders unchecked by default', () => {
    render(<Radio value="test" data-testid="radio" />);
    const radio = screen.getByTestId('radio') as HTMLInputElement;
    expect(radio.checked).toBe(false);
  });

  it('renders checked when defaultChecked', () => {
    render(<Radio value="test" defaultChecked data-testid="radio" />);
    const radio = screen.getByTestId('radio') as HTMLInputElement;
    expect(radio.checked).toBe(true);
  });

  it('renders checked when controlled', () => {
    render(<Radio value="test" checked={true} data-testid="radio" />);
    const radio = screen.getByTestId('radio') as HTMLInputElement;
    expect(radio.checked).toBe(true);
  });

  it('calls onChange when clicked', () => {
    const handleChange = vi.fn();
    render(<Radio value="test" onChange={handleChange} data-testid="radio" />);
    const radio = screen.getByTestId('radio') as HTMLInputElement;
    fireEvent.click(radio);
    expect(handleChange).toHaveBeenCalledTimes(1);
  });

  it('displays label', () => {
    render(<Radio value="test" label="Radio Label" />);
    expect(screen.getByText('Radio Label')).toBeInTheDocument();
  });

  it('applies small size', () => {
    render(<Radio value="test" size="small" data-testid="radio" />);
    const radio = screen.getByTestId('radio');
    expect(radio).toHaveClass('size-4');
  });

  it('applies medium size (default)', () => {
    render(<Radio value="test" data-testid="radio" />);
    const radio = screen.getByTestId('radio');
    expect(radio).toHaveClass('size-5');
  });

  it('applies primary color (default)', () => {
    render(<Radio value="test" data-testid="radio" />);
    const radio = screen.getByTestId('radio');
    expect(radio.className).toContain('checked:bg-nest-primary');
  });

  it('applies custom color', () => {
    render(<Radio value="test" color="secondary" data-testid="radio" />);
    const radio = screen.getByTestId('radio');
    expect(radio.className).toContain('checked:bg-nest-secondary');
  });

  it('applies disabled state', () => {
    render(<Radio value="test" disabled data-testid="radio" />);
    const radio = screen.getByTestId('radio') as HTMLInputElement;
    expect(radio).toHaveClass('disabled:opacity-50');
    expect(radio.disabled).toBe(true);
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLInputElement | null };
    render(<Radio value="test" ref={ref} data-testid="radio" />);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.type).toBe('radio');
  });
});

describe('RadioGroup', () => {
  it('renders children radios', () => {
    render(
      <RadioGroup>
        <Radio value="one" label="One" />
        <Radio value="two" label="Two" />
      </RadioGroup>
    );
    expect(screen.getByText('One')).toBeInTheDocument();
    expect(screen.getByText('Two')).toBeInTheDocument();
  });

  it('manages selection state (controlled)', () => {
    const handleChange = vi.fn();
    render(
      <RadioGroup value="one" onChange={handleChange}>
        <Radio value="one" label="One" data-testid="radio-one" />
        <Radio value="two" label="Two" data-testid="radio-two" />
      </RadioGroup>
    );
    const radioOne = screen.getByTestId('radio-one') as HTMLInputElement;
    const radioTwo = screen.getByTestId('radio-two') as HTMLInputElement;
    expect(radioOne.checked).toBe(true);
    expect(radioTwo.checked).toBe(false);
    fireEvent.click(radioTwo);
    expect(handleChange).toHaveBeenCalledWith('two', expect.objectContaining({ target: expect.objectContaining({ value: 'two' }) }));
  });

  it('manages selection state (uncontrolled)', () => {
    render(
      <RadioGroup defaultValue="one">
        <Radio value="one" label="One" data-testid="radio-one" />
        <Radio value="two" label="Two" data-testid="radio-two" />
      </RadioGroup>
    );
    const radioOne = screen.getByTestId('radio-one') as HTMLInputElement;
    const radioTwo = screen.getByTestId('radio-two') as HTMLInputElement;
    expect(radioOne.checked).toBe(true);
    expect(radioTwo.checked).toBe(false);
    fireEvent.click(radioTwo);
    expect(radioTwo.checked).toBe(true);
    expect(radioOne.checked).toBe(false);
  });

  it('applies row layout', () => {
    render(
      <RadioGroup row data-testid="group">
        <Radio value="one" />
      </RadioGroup>
    );
    const group = screen.getByTestId('group');
    expect(group).toHaveClass('flex-row');
  });

  it('applies column layout (default)', () => {
    render(
      <RadioGroup data-testid="group">
        <Radio value="one" />
      </RadioGroup>
    );
    const group = screen.getByTestId('group');
    expect(group).toHaveClass('flex-col');
  });

  it('passes disabled to children', () => {
    render(
      <RadioGroup disabled>
        <Radio value="one" label="One" data-testid="radio-one" />
        <Radio value="two" label="Two" data-testid="radio-two" />
      </RadioGroup>
    );
    const radioOne = screen.getByTestId('radio-one') as HTMLInputElement;
    const radioTwo = screen.getByTestId('radio-two') as HTMLInputElement;
    expect(radioOne.disabled).toBe(true);
    expect(radioTwo.disabled).toBe(true);
  });

  it('passes color to children', () => {
    render(
      <RadioGroup color="error">
        <Radio value="one" data-testid="radio-one" />
      </RadioGroup>
    );
    const radio = screen.getByTestId('radio-one');
    expect(radio.className).toContain('checked:bg-nest-error');
  });

  it('passes name to children', () => {
    render(
      <RadioGroup name="test-group">
        <Radio value="one" data-testid="radio-one" />
      </RadioGroup>
    );
    const radio = screen.getByTestId('radio-one') as HTMLInputElement;
    expect(radio.name).toBe('test-group');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <RadioGroup ref={ref} data-testid="group">
        <Radio value="one" />
      </RadioGroup>
    );
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.role).toBe('radiogroup');
  });

  it('allows individual radio to override group disabled', () => {
    render(
      <RadioGroup disabled>
        <Radio value="one" disabled={false} data-testid="radio-one" />
      </RadioGroup>
    );
    const radio = screen.getByTestId('radio-one') as HTMLInputElement;
    // Individual override should work
    expect(radio.disabled).toBe(false);
  });
});
