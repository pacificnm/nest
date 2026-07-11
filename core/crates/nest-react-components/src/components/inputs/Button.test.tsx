import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders children', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole('button', { name: 'Click me' })).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click</Button>);
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies loading state', () => {
    render(<Button loading>Loading</Button>);
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('disabled');
    expect(screen.getByRole('img', { hidden: true })).toBeInTheDocument(); // spinner
  });

  it('applies disabled state', () => {
    render(<Button disabled>Disabled</Button>);
    expect(screen.getByRole('button')).toHaveAttribute('disabled');
  });

  it('applies fullWidth class', () => {
    render(<Button fullWidth>Full Width</Button>);
    expect(screen.getByRole('button')).toHaveClass('w-full');
  });

  it('renders startIcon', () => {
    render(<Button startIcon={<span data-testid="icon">🔍</span>}>Search</Button>);
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('renders endIcon', () => {
    render(<Button endIcon={<span data-testid="icon">→</span>}>Next</Button>);
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('applies variant styles', () => {
    const { rerender } = render(<Button variant="contained">Contained</Button>);
    expect(screen.getByRole('button')).toHaveClass('bg-nest-primary');

    rerender(<Button variant="outlined">Outlined</Button>);
    expect(screen.getByRole('button')).toHaveClass('border-nest-primary');

    rerender(<Button variant="text">Text</Button>);
    expect(screen.getByRole('button')).not.toHaveClass('bg-');
  });

  it('applies color styles', () => {
    const { rerender } = render(<Button color="primary">Primary</Button>);
    expect(screen.getByRole('button')).toHaveClass('bg-nest-primary');

    rerender(<Button color="error">Error</Button>);
    expect(screen.getByRole('button')).toHaveClass('bg-nest-error');
  });

  it('applies size styles', () => {
    const { rerender } = render(<Button size="small">Small</Button>);
    expect(screen.getByRole('button')).toHaveClass('h-8');

    rerender(<Button size="large">Large</Button>);
    expect(screen.getByRole('button')).toHaveClass('h-12');
  });
});
