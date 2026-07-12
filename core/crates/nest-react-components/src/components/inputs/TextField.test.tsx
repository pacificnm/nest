import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { TextField } from './TextField';

describe('TextField', () => {
  it('renders a labelled input', () => {
    render(<TextField label="Email" />);
    expect(screen.getByLabelText('Email')).toBeInTheDocument();
  });

  it('fires onChange when typing', () => {
    const onChange = vi.fn();
    render(<TextField label="Name" onChange={onChange} />);
    fireEvent.change(screen.getByLabelText('Name'), { target: { value: 'Ada' } });
    expect(onChange).toHaveBeenCalledTimes(1);
  });

  it('renders an error message with the alert role', () => {
    render(<TextField label="Email" error="Invalid email" />);
    const message = screen.getByRole('alert');
    expect(message).toHaveTextContent('Invalid email');
    expect(screen.getByLabelText('Email')).toHaveAttribute('aria-invalid', 'true');
  });

  it('renders helper text when there is no error', () => {
    render(<TextField label="Email" helperText="We never share it" />);
    expect(screen.getByText('We never share it')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('renders a textarea when multiline', () => {
    render(<TextField label="Bio" multiline />);
    expect(screen.getByLabelText('Bio').tagName).toBe('TEXTAREA');
  });

  it('renders start and end adornments', () => {
    render(
      <TextField
        label="Search"
        startAdornment={<span data-testid="start">@</span>}
        endAdornment={<span data-testid="end">x</span>}
      />
    );
    expect(screen.getByTestId('start')).toBeInTheDocument();
    expect(screen.getByTestId('end')).toBeInTheDocument();
  });

  it('applies the disabled attribute', () => {
    render(<TextField label="Email" disabled />);
    expect(screen.getByLabelText('Email')).toBeDisabled();
  });
});
