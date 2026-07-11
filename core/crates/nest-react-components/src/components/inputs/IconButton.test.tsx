import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { IconButton } from './IconButton';

describe('IconButton', () => {
  it('renders children', () => {
    render(<IconButton aria-label="test"><span>🔍</span></IconButton>);
    expect(screen.getByRole('button', { name: 'test' })).toBeInTheDocument();
    expect(screen.getByText('🔍')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(
      <IconButton aria-label="test" onClick={handleClick}>
        <span>🔍</span>
      </IconButton>
    );
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies disabled state', () => {
    render(<IconButton aria-label="test" disabled><span>🔍</span></IconButton>);
    expect(screen.getByRole('button')).toHaveAttribute('disabled');
  });

  it('applies size styles', () => {
    const { rerender } = render(<IconButton aria-label="test" size="small"><span>🔍</span></IconButton>);
    expect(screen.getByRole('button')).toHaveClass('size-8');

    rerender(<IconButton aria-label="test" size="large"><span>🔍</span></IconButton>);
    expect(screen.getByRole('button')).toHaveClass('size-12');
  });

  it('applies color styles', () => {
    const { rerender } = render(<IconButton aria-label="test" color="primary"><span>🔍</span></IconButton>);
    expect(screen.getByRole('button')).toHaveClass('text-nest-primary');

    rerender(<IconButton aria-label="test" color="error"><span>🔍</span></IconButton>);
    expect(screen.getByRole('button')).toHaveClass('text-nest-error');
  });
});
