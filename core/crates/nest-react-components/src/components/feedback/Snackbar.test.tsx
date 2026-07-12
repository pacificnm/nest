import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Snackbar } from './Snackbar';

afterEach(() => {
  vi.useRealTimers();
});

describe('Snackbar', () => {
  it('renders nothing when closed', () => {
    render(
      <Snackbar open={false} onClose={vi.fn()}>
        Saved
      </Snackbar>
    );
    expect(screen.queryByText('Saved')).not.toBeInTheDocument();
  });

  it('renders content with a live-region status role when open', () => {
    render(
      <Snackbar open onClose={vi.fn()}>
        Saved
      </Snackbar>
    );
    expect(screen.getByRole('status')).toHaveTextContent('Saved');
  });

  it('applies position styles', () => {
    render(
      <Snackbar open onClose={vi.fn()} position="top-right">
        Positioned
      </Snackbar>
    );
    expect(screen.getByRole('status')).toHaveClass('top-4', 'right-4');
  });

  it('auto-closes after autoHideDuration', () => {
    vi.useFakeTimers();
    const onClose = vi.fn();
    render(
      <Snackbar open onClose={onClose} autoHideDuration={1000}>
        Auto
      </Snackbar>
    );
    expect(onClose).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1000);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('does not auto-close when autoHideDuration is 0', () => {
    vi.useFakeTimers();
    const onClose = vi.fn();
    render(
      <Snackbar open onClose={onClose} autoHideDuration={0}>
        Sticky
      </Snackbar>
    );
    vi.advanceTimersByTime(10000);
    expect(onClose).not.toHaveBeenCalled();
  });

  it('renders an action', () => {
    render(
      <Snackbar open onClose={vi.fn()} action={<button>Undo</button>}>
        Deleted
      </Snackbar>
    );
    expect(screen.getByRole('button', { name: 'Undo' })).toBeInTheDocument();
  });
});
