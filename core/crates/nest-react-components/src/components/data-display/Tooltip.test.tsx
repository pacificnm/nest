import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Tooltip } from './Tooltip';

describe('Tooltip', () => {
  it('renders children', () => {
    render(
      <Tooltip title="Tooltip content">
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );
    expect(screen.getByTestId('trigger')).toBeInTheDocument();
  });

  it('shows tooltip on hover', async () => {
    render(
      <Tooltip title="Tooltip content">
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);

    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });
    expect(screen.getByRole('tooltip')).toHaveTextContent('Tooltip content');
  });

  it('hides tooltip on mouse leave', async () => {
    render(
      <Tooltip title="Tooltip content" leaveDelay={0}>
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    fireEvent.mouseLeave(trigger);
    await waitFor(() => {
      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });
  });

  it('shows tooltip on focus', async () => {
    render(
      <Tooltip title="Tooltip content">
        <button data-testid="trigger">Focus me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    trigger.focus();
    fireEvent.focus(trigger);

    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });
  });

  it('hides tooltip on blur', async () => {
    render(
      <Tooltip title="Tooltip content" leaveDelay={0}>
        <button data-testid="trigger">Focus me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    trigger.focus();
    fireEvent.focus(trigger);
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    trigger.blur();
    fireEvent.blur(trigger);
    await waitFor(() => {
      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    });
  });

  it('respects enterDelay', async () => {
    render(
      <Tooltip title="Tooltip content" enterDelay={500}>
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);

    // Should not show immediately
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();

    // Should show after delay
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    }, { timeout: 600 });
  });

  it('respects leaveDelay', async () => {
    render(
      <Tooltip title="Tooltip content" leaveDelay={500}>
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });

    fireEvent.mouseLeave(trigger);

    // Should still be visible during delay
    expect(screen.getByRole('tooltip')).toBeInTheDocument();

    // Should hide after delay
    await waitFor(() => {
      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
    }, { timeout: 600 });
  });

  it('applies custom placement', async () => {
    render(
      <Tooltip title="Tooltip content" placement="right">
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);

    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
    });
  });

  it('applies custom className', async () => {
    render(
      <Tooltip title="Tooltip content" className="custom-class">
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);

    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toHaveClass('custom-class');
    });
  });

  it('forwards ref to tooltip element', async () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Tooltip title="Tooltip content" ref={ref}>
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);

    await waitFor(() => {
      expect(ref.current).toBeInTheDocument();
    });
  });

  it('supports controlled open state', async () => {
    const onOpenChange = vi.fn();
    render(
      <Tooltip title="Tooltip content" open onOpenChange={onOpenChange}>
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    // Should be visible because open=true
    expect(screen.getByRole('tooltip')).toBeInTheDocument();

    // Try to trigger close (should call onOpenChange but not actually close since controlled)
    const trigger = screen.getByTestId('trigger');
    fireEvent.keyDown(trigger, { key: 'Escape' });

    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('has correct role for accessibility', async () => {
    render(
      <Tooltip title="Tooltip content">
        <button data-testid="trigger">Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByTestId('trigger');
    fireEvent.mouseEnter(trigger);

    await waitFor(() => {
      const tooltip = screen.getByRole('tooltip');
      expect(tooltip).toHaveAttribute('role', 'tooltip');
    });
  });
});
