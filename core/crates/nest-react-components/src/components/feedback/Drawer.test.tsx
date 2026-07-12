import { render, screen, fireEvent } from '@testing-library/react';
import { Drawer } from './Drawer';
import { describe, it, expect, vi } from 'vitest';

describe('Drawer', () => {
  it('renders when open', () => {
    render(
      <Drawer open={true} onClose={vi.fn()}>
        <div data-testid="drawer-content">Hello</div>
      </Drawer>
    );
    expect(screen.getByTestId('drawer-content')).toBeInTheDocument();
  });

  it('does not render when closed', () => {
    render(
      <Drawer open={false} onClose={vi.fn()}>
        <div data-testid="drawer-content">Hello</div>
      </Drawer>
    );
    expect(screen.queryByTestId('drawer-content')).not.toBeInTheDocument();
  });

  it('positions correctly for left anchor', () => {
    render(
      <Drawer open={true} onClose={vi.fn()} anchor="left">
        <div>Content</div>
      </Drawer>
    );
    const drawer = screen.getByRole('dialog');
    expect(drawer).toHaveStyle('left: 0px');
  });

  it('positions correctly for right anchor', () => {
    render(
      <Drawer open={true} onClose={vi.fn()} anchor="right">
        <div>Content</div>
      </Drawer>
    );
    const drawer = screen.getByRole('dialog');
    expect(drawer).toHaveStyle('right: 0px');
  });

  it('positions correctly for top anchor', () => {
    render(
      <Drawer open={true} onClose={vi.fn()} anchor="top">
        <div>Content</div>
      </Drawer>
    );
    const drawer = screen.getByRole('dialog');
    expect(drawer).toHaveStyle('top: 0px');
  });

  it('positions correctly for bottom anchor', () => {
    render(
      <Drawer open={true} onClose={vi.fn()} anchor="bottom">
        <div>Content</div>
      </Drawer>
    );
    const drawer = screen.getByRole('dialog');
    expect(drawer).toHaveStyle('bottom: 0px');
  });

  it('calls onClose when clicking outside', () => {
    const onClose = vi.fn();
    render(
      <Drawer open={true} onClose={onClose}>
        <div data-testid="drawer-content">Hello</div>
      </Drawer>
    );
    fireEvent.mouseDown(document.body);
    expect(onClose).toHaveBeenCalled();
  });

  it('calls onClose when pressing Escape', () => {
    const onClose = vi.fn();
    render(
      <Drawer open={true} onClose={onClose}>
        <div data-testid="drawer-content">Hello</div>
      </Drawer>
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
  });

  it('does not call onClose when pressing Escape with closeOnEscape=false', () => {
    const onClose = vi.fn();
    render(
      <Drawer open={true} onClose={onClose} closeOnEscape={false}>
        <div data-testid="drawer-content">Hello</div>
      </Drawer>
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).not.toHaveBeenCalled();
  });

  it('does not call onClose when clicking outside with closeOnOutsideClick=false', () => {
    const onClose = vi.fn();
    render(
      <Drawer open={true} onClose={onClose} closeOnOutsideClick={false}>
        <div data-testid="drawer-content">Hello</div>
      </Drawer>
    );
    fireEvent.mouseDown(document.body);
    expect(onClose).not.toHaveBeenCalled();
  });

  it('applies custom width for left/right anchors', () => {
    render(
      <Drawer open={true} onClose={vi.fn()} anchor="left" width={400}>
        <div>Content</div>
      </Drawer>
    );
    const drawer = screen.getByRole('dialog');
    expect(drawer).toHaveStyle('width: 400px');
  });

  it('applies custom height for top/bottom anchors', () => {
    render(
      <Drawer open={true} onClose={vi.fn()} anchor="top" height={300}>
        <div>Content</div>
      </Drawer>
    );
    const drawer = screen.getByRole('dialog');
    expect(drawer).toHaveStyle('height: 300px');
  });

  it('applies custom className', () => {
    render(
      <Drawer open={true} onClose={vi.fn()} className="custom-drawer">
        <div>Content</div>
      </Drawer>
    );
    const drawer = screen.getByRole('dialog');
    expect(drawer).toHaveClass('custom-drawer');
  });
});
