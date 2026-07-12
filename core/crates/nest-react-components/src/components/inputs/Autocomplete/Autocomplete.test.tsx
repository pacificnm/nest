import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Autocomplete } from './Autocomplete';

const OPTIONS = [
  { value: 'react', label: 'React' },
  { value: 'vue', label: 'Vue' },
  { value: 'svelte', label: 'Svelte' },
];

describe('Autocomplete', () => {
  it('renders a combobox input', () => {
    render(<Autocomplete options={OPTIONS} placeholder="Search" />);
    expect(screen.getByRole('combobox')).toBeInTheDocument();
  });

  it('renders the label', () => {
    render(<Autocomplete options={OPTIONS} label="Framework" />);
    expect(screen.getByText('Framework')).toBeInTheDocument();
  });

  it('filters options as the user types', () => {
    render(<Autocomplete options={OPTIONS} />);
    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'vu' } });
    expect(screen.getByRole('option', { name: 'Vue' })).toBeInTheDocument();
    expect(screen.queryByRole('option', { name: 'React' })).not.toBeInTheDocument();
  });

  it('fires onChange when an option is selected', () => {
    const onChange = vi.fn();
    render(<Autocomplete options={OPTIONS} onChange={onChange} />);
    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'Re' } });
    fireEvent.click(screen.getByRole('option', { name: 'React' }));
    expect(onChange).toHaveBeenCalledWith('react');
  });

  it('reports input changes', () => {
    const onInputChange = vi.fn();
    render(<Autocomplete options={OPTIONS} onInputChange={onInputChange} />);
    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'sv' } });
    expect(onInputChange).toHaveBeenCalledWith('sv');
  });

  it('applies the disabled attribute', () => {
    render(<Autocomplete options={OPTIONS} disabled />);
    expect(screen.getByRole('combobox')).toBeDisabled();
  });
});
