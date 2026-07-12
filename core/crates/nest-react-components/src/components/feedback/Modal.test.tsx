import { render, screen, fireEvent } from '@testing-library/react';
import { Modal } from './Modal';
import { describe, it, expect, vi } from 'vitest';

describe('Modal', () => {
  it('renders when open', () => {
    render(
      <Modal open={true} onClose={vi.fn()}>
        <div data-testid="modal-content">Hello</div>
      </Modal>
    );
    expect(screen.getByTestId('modal-content')).toBeInTheDocument();
  });

  it('does not render when closed', () => {
    render(
      <Modal open={false} onClose={vi.fn()}>
        <div data-testid="modal-content">Hello</div>
      </Modal>
    );
    expect(screen.queryByTestId('modal-content')).not.toBeInTheDocument();
  });

  it('calls onClose when clicking outside', () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} onClose={onClose}>
        <div data-testid="modal-content">Hello</div>
      </Modal>
    );
    fireEvent.mouseDown(document.querySelector('.fixed.inset-0') || document.body);
    expect(onClose).toHaveBeenCalled();
  });

  it('calls onClose when pressing Escape', () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} onClose={onClose}>
        <div data-testid="modal-content">Hello</div>
      </Modal>
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
  });

  it('does not call onClose when pressing Escape with closeOnEscape=false', () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} onClose={onClose} closeOnEscape={false}>
        <div data-testid="modal-content">Hello</div>
      </Modal>
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).not.toHaveBeenCalled();
  });

  it('does not call onClose when clicking outside with closeOnOutsideClick=false', () => {
    const onClose = vi.fn();
    render(
      <Modal open={true} onClose={onClose} closeOnOutsideClick={false}>
        <div data-testid="modal-content">Hello</div>
      </Modal>
    );
    fireEvent.mouseDown(document.body);
    expect(onClose).not.toHaveBeenCalled();
  });

  it('applies custom className', () => {
    render(
      <Modal open={true} onClose={vi.fn()} className="custom-modal">
        <div>Content</div>
      </Modal>
    );
    const modal = screen.getByRole('dialog');
    expect(modal).toHaveClass('custom-modal');
  });

  it('applies custom backdropClassName', () => {
    render(
      <Modal open={true} onClose={vi.fn()} backdropClassName="custom-backdrop">
        <div>Content</div>
      </Modal>
    );
    const backdrop = document.querySelector('.fixed.inset-0');
    expect(backdrop).toHaveClass('custom-backdrop');
  });
});
