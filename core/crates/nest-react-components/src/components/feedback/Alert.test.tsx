import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Alert } from './Alert';

describe('Alert', () => {
  it('renders children with the alert role', () => {
    render(<Alert>Heads up</Alert>);
    expect(screen.getByRole('alert')).toHaveTextContent('Heads up');
  });

  it('applies severity color styles', () => {
    const { rerender } = render(<Alert severity="info">Info</Alert>);
    expect(screen.getByRole('alert')).toHaveClass('text-nest-info');

    rerender(<Alert severity="error">Error</Alert>);
    expect(screen.getByRole('alert')).toHaveClass('text-nest-error');
  });

  it('applies variant styles', () => {
    render(<Alert variant="outlined" severity="success">Done</Alert>);
    expect(screen.getByRole('alert')).toHaveClass('border-nest-success');
  });

  it('renders a close button and fires onClose', () => {
    const onClose = vi.fn();
    render(<Alert onClose={onClose}>Closable</Alert>);
    fireEvent.click(screen.getByRole('button', { name: 'Close alert' }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('does not render a close button without onClose', () => {
    render(<Alert>No close</Alert>);
    expect(screen.queryByRole('button', { name: 'Close alert' })).not.toBeInTheDocument();
  });

  it('renders a custom icon and an action', () => {
    render(
      <Alert icon={<span data-testid="icon">!</span>} action={<button>Undo</button>}>
        With extras
      </Alert>
    );
    expect(screen.getByTestId('icon')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Undo' })).toBeInTheDocument();
  });
});
