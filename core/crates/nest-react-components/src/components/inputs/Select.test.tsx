import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Select } from './Select';

const OPTIONS = [
  { value: 'a', label: 'Option A' },
  { value: 'b', label: 'Option B' },
  { value: 'c', label: 'Option C', disabled: true },
];

describe('Select', () => {
  it('shows the placeholder when nothing is selected', () => {
    render(<Select options={OPTIONS} placeholder="Pick one" />);
    expect(screen.getByRole('combobox')).toHaveTextContent('Pick one');
  });

  it('renders the label', () => {
    render(<Select options={OPTIONS} label="Choice" />);
    expect(screen.getByText('Choice')).toBeInTheDocument();
  });

  it('shows the selected value', () => {
    render(<Select options={OPTIONS} value="b" />);
    expect(screen.getByRole('combobox')).toHaveTextContent('Option B');
  });

  it('opens the listbox on click', () => {
    render(<Select options={OPTIONS} />);
    fireEvent.click(screen.getByRole('combobox'));
    expect(screen.getByRole('listbox')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Option A' })).toBeInTheDocument();
  });

  it('fires onChange when an option is selected', () => {
    const onChange = vi.fn();
    render(<Select options={OPTIONS} onChange={onChange} />);
    fireEvent.click(screen.getByRole('combobox'));
    fireEvent.click(screen.getByRole('option', { name: 'Option A' }));
    expect(onChange).toHaveBeenCalledWith('a');
  });

  it('does not select a disabled option', () => {
    const onChange = vi.fn();
    render(<Select options={OPTIONS} onChange={onChange} />);
    fireEvent.click(screen.getByRole('combobox'));
    fireEvent.click(screen.getByRole('option', { name: 'Option C' }));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('applies error styling', () => {
    render(<Select options={OPTIONS} error />);
    expect(screen.getByRole('combobox')).toHaveClass('border-nest-error');
  });
});
