import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Badge } from './Badge';

describe('Badge', () => {
  it('renders children', () => {
    render(
      <Badge badgeContent={4}>
        <span data-testid="child">Child</span>
      </Badge>
    );
    expect(screen.getByTestId('child')).toBeInTheDocument();
  });

  it('renders wrapper as span by default', () => {
    render(
      <Badge badgeContent={4} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    expect(screen.getByTestId('badge').tagName).toBe('SPAN');
  });

  it('renders wrapper as custom component', () => {
    render(
      <Badge badgeContent={4} component="div" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    expect(screen.getByTestId('badge').tagName).toBe('DIV');
  });

  it('shows badge with number content', () => {
    render(
      <Badge badgeContent={5} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveTextContent('5');
  });

  it('shows badge with string content', () => {
    render(
      <Badge badgeContent="New" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveTextContent('New');
  });

  it('applies max to number content', () => {
    render(
      <Badge badgeContent={999} max={99} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveTextContent('99+');
  });

  it('does not show badge when content is 0 and showZero is false', () => {
    render(
      <Badge badgeContent={0} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badgeElement = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badgeElement).not.toBeInTheDocument();
  });

  it('shows badge when content is 0 and showZero is true', () => {
    render(
      <Badge badgeContent={0} showZero wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveTextContent('0');
  });

  it('does not show badge when invisible is true', () => {
    render(
      <Badge badgeContent={5} invisible wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badgeElement = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badgeElement).not.toBeInTheDocument();
  });

  it('applies default color (default)', () => {
    render(
      <Badge badgeContent={1} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('bg-nest-muted', 'text-nest-foreground');
  });

  it('applies primary color', () => {
    render(
      <Badge badgeContent={1} color="primary" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('bg-nest-primary', 'text-white');
  });

  it('applies error color', () => {
    render(
      <Badge badgeContent={1} color="error" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('bg-nest-error', 'text-white');
  });

  it('applies default variant (standard)', () => {
    render(
      <Badge badgeContent={1} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('min-w-[1.25rem]', 'h-5', 'px-1.5');
  });

  it('applies dot variant', () => {
    render(
      <Badge badgeContent={1} variant="dot" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('h-2.5', 'w-2.5', 'min-w-0', 'p-0');
  });

  it('dot variant does not show content', () => {
    render(
      <Badge badgeContent={5} variant="dot" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge?.textContent).toBe('');
  });

  it('applies default anchor (top right)', () => {
    render(
      <Badge badgeContent={1} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('top-0', 'right-0', '-translate-y-1/2', 'translate-x-1/2');
  });

  it('applies custom anchor (bottom left)', () => {
    render(
      <Badge
        badgeContent={1}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
        wrapperDataTestId="badge"
      >
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('bottom-0', 'left-0', 'translate-y-1/2', '-translate-x-1/2');
  });

  it('applies custom className to badge', () => {
    render(
      <Badge badgeContent={1} className="custom-badge" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveClass('custom-badge');
  });

  it('applies wrapperClassName to wrapper', () => {
    render(
      <Badge badgeContent={1} wrapperClassName="custom-wrapper" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    expect(screen.getByTestId('badge')).toHaveClass('custom-wrapper');
  });

  it('forwards ref to badge element', () => {
    const ref = { current: null as HTMLSpanElement | null };
    render(
      <Badge badgeContent={1} ref={ref} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    expect(ref.current).toBeInTheDocument();
  });

  it('passes through additional props to badge', () => {
    render(
      <Badge badgeContent={1} id="test-id" aria-label="Notification" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badge = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badge).toHaveAttribute('id', 'test-id');
    expect(badge).toHaveAttribute('aria-label', 'Notification');
  });

  it('renders with React node children', () => {
    render(
      <Badge badgeContent={1} wrapperDataTestId="badge">
        <div data-testid="complex-child">Complex</div>
      </Badge>
    );
    expect(screen.getByTestId('complex-child')).toBeInTheDocument();
  });

  it('hides badge when content is null', () => {
    render(
      <Badge badgeContent={null} wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badgeElement = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badgeElement).not.toBeInTheDocument();
  });

  it('hides badge when content is empty string', () => {
    render(
      <Badge badgeContent="" wrapperDataTestId="badge">
        <span>Child</span>
      </Badge>
    );
    const badgeElement = screen.getByTestId('badge').querySelector('[data-badge]');
    expect(badgeElement).not.toBeInTheDocument();
  });
});
