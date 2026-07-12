import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Popover } from './Popover';

describe('Popover', () => {
  it('renders trigger element', () => {
    render(
      <Popover trigger={<button data-testid="trigger">Open</button>}>
        <div>Content</div>
      </Popover>
    );
    expect(screen.getByTestId('trigger')).toBeInTheDocument();
  });

  it('does not render content when closed', () => {
    render(
      <Popover trigger={<button>Open</button>} open={false}>
        <div data-testid="content">Content</div>
      </Popover>
    );
    expect(screen.queryByTestId('content')).not.toBeInTheDocument();
  });

  it('renders content when open', () => {
    render(
      <Popover trigger={<button>Open</button>} open>
        <div data-testid="content">Content</div>
      </Popover>
    );
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('toggles on trigger click (uncontrolled)', async () => {
    render(
      <Popover trigger={<button data-testid="trigger">Open</button>}>
        <div data-testid="content">Content</div>
      </Popover>
    );

    // Initially closed
    expect(screen.queryByTestId('content')).not.toBeInTheDocument();

    // Click to open
    fireEvent.click(screen.getByTestId('trigger'));
    await waitFor(() => {
      expect(screen.getByTestId('content')).toBeInTheDocument();
    });

    // Click to close
    fireEvent.click(screen.getByTestId('trigger'));
    await waitFor(() => {
      expect(screen.queryByTestId('content')).not.toBeInTheDocument();
    });
  });

  it('calls onOpenChange when toggled', async () => {
    const onOpenChange = vi.fn();
    render(
      <Popover
        trigger={<button data-testid="trigger">Open</button>}
        onOpenChange={onOpenChange}
      >
        <div>Content</div>
      </Popover>
    );

    fireEvent.click(screen.getByTestId('trigger'));
    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(true);
    });

    fireEvent.click(screen.getByTestId('trigger'));
    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  it('closes on Escape key', async () => {
    const onOpenChange = vi.fn();
    render(
      <Popover
        trigger={<button data-testid="trigger">Open</button>}
        open
        onOpenChange={onOpenChange}
      >
        <div data-testid="content">Content</div>
      </Popover>
    );

    expect(screen.getByTestId('content')).toBeInTheDocument();

    fireEvent.keyDown(screen.getByTestId('content'), { key: 'Escape' });
    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  it('respects closeOnOutsideClick=false', async () => {
    render(
      <Popover
        trigger={<button data-testid="trigger">Open</button>}
        open
        closeOnOutsideClick={false}
      >
        <div data-testid="content">Content</div>
      </Popover>
    );

    fireEvent.mouseDown(document.body);
    // Should still be visible
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('respects closeOnOutsideClick=false', async () => {
    render(
      <Popover
        trigger={<button data-testid="trigger">Open</button>}
        open
        closeOnOutsideClick={false}
      >
        <div data-testid="content">Content</div>
      </Popover>
    );

    fireEvent.mouseDown(document.body);
    // Should still be visible
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('respects closeOnEscape=false', async () => {
    render(
      <Popover
        trigger={<button data-testid="trigger">Open</button>}
        open
        closeOnEscape={false}
      >
        <div data-testid="content">Content</div>
      </Popover>
    );

    fireEvent.keyDown(screen.getByTestId('content'), { key: 'Escape' });
    // Should still be visible
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('applies custom placement', () => {
    render(
      <Popover trigger={<button>Open</button>} open placement="left">
        <div data-testid="content">Content</div>
      </Popover>
    );
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    render(
      <Popover trigger={<button>Open</button>} open className="custom-class">
        <div>Content</div>
      </Popover>
    );
    // The popover container should have the custom class
    expect(screen.getByRole('dialog')).toHaveClass('custom-class');
  });

  it('forwards ref to popover content', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Popover trigger={<button>Open</button>} open ref={ref}>
        <div>Content</div>
      </Popover>
    );
    expect(ref.current).toBeInTheDocument();
  });

  it('has dialog role', () => {
    render(
      <Popover trigger={<button>Open</button>} open>
        <div>Content</div>
      </Popover>
    );
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});
