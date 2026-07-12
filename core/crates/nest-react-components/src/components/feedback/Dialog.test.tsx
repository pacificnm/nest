import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Dialog } from './Dialog';

describe('Dialog', () => {
  it('renders nothing when closed', () => {
    render(
      <Dialog open={false} onClose={vi.fn()}>
        Body
      </Dialog>
    );
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('renders as a modal dialog when open', () => {
    render(
      <Dialog open onClose={vi.fn()}>
        Body
      </Dialog>
    );
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveTextContent('Body');
  });

  it('renders the title and actions', () => {
    render(
      <Dialog open onClose={vi.fn()} title="Confirm" actions={<button>OK</button>}>
        Body
      </Dialog>
    );
    expect(screen.getByRole('heading', { name: 'Confirm' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'OK' })).toBeInTheDocument();
  });

  it('fires onClose from the close button', () => {
    const onClose = vi.fn();
    render(
      <Dialog open onClose={onClose}>
        Body
      </Dialog>
    );
    fireEvent.click(screen.getByRole('button', { name: 'Close dialog' }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('closes on backdrop click unless disabled', () => {
    const onClose = vi.fn();
    const { rerender } = render(
      <Dialog open onClose={onClose}>
        Body
      </Dialog>
    );
    fireEvent.click(screen.getByRole('dialog'));
    expect(onClose).toHaveBeenCalledTimes(1);

    onClose.mockClear();
    rerender(
      <Dialog open onClose={onClose} disableBackdropClick>
        Body
      </Dialog>
    );
    fireEvent.click(screen.getByRole('dialog'));
    expect(onClose).not.toHaveBeenCalled();
  });

  it('closes on Escape unless disabled', () => {
    const onClose = vi.fn();
    const { rerender } = render(
      <Dialog open onClose={onClose}>
        Body
      </Dialog>
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);

    onClose.mockClear();
    rerender(
      <Dialog open onClose={onClose} disableEscapeKeyDown>
        Body
      </Dialog>
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).not.toHaveBeenCalled();
  });
});
