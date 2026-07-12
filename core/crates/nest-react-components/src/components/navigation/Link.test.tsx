import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Link } from './Link';

describe('Link', () => {
  it('renders children', () => {
    render(<Link href="/test">Link Text</Link>);
    expect(screen.getByText('Link Text')).toBeInTheDocument();
  });

  it('renders as anchor by default', () => {
    render(<Link href="/test" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link').tagName).toBe('A');
  });

  it('renders as custom component', () => {
    render(<Link href="/test" component="button" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link').tagName).toBe('BUTTON');
  });

  it('applies href attribute', () => {
    render(<Link href="/test" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveAttribute('href', '/test');
  });

  it('applies default underline (hover)', () => {
    render(<Link href="/test" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('underline', 'hover:underline');
  });

  it('applies underline="none"', () => {
    render(<Link href="/test" underline="none" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('no-underline');
    expect(screen.getByTestId('link')).not.toHaveClass('underline');
  });

  it('applies underline="hover"', () => {
    render(<Link href="/test" underline="hover" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('underline', 'hover:underline');
  });

  it('applies underline="always"', () => {
    render(<Link href="/test" underline="always" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('underline');
  });

  it('applies default color (primary)', () => {
    render(<Link href="/test" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('text-nest-primary');
  });

  it('applies color="primary"', () => {
    render(<Link href="/test" color="primary" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('text-nest-primary');
  });

  it('applies color="inherit"', () => {
    render(<Link href="/test" color="inherit" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('text-inherit');
  });

  it('applies external props (target and rel)', () => {
    render(<Link href="https://example.com" external data-testid="link">External</Link>);
    expect(screen.getByTestId('link')).toHaveAttribute('target', '_blank');
    expect(screen.getByTestId('link')).toHaveAttribute('rel', 'noopener noreferrer');
  });

  it('does not apply external props when external=false', () => {
    render(<Link href="/test" external={false} data-testid="link">Internal</Link>);
    expect(screen.getByTestId('link')).not.toHaveAttribute('target', '_blank');
    expect(screen.getByTestId('link')).not.toHaveAttribute('rel', 'noopener noreferrer');
  });

  it('applies base styles', () => {
    render(<Link href="/test" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass(
      'cursor-pointer',
      'font-body',
      'transition-colors',
      'duration-150',
      'focus:outline-none',
      'focus:ring-2',
      'focus:ring-nest-primary/50',
      'focus:ring-offset-2',
      'rounded-nest-sm'
    );
  });

  it('applies custom className', () => {
    render(<Link href="/test" className="custom-link" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('custom-link');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLAnchorElement | null };
    render(<Link href="/test" ref={ref}>Link</Link>);
    expect(ref.current).toBeInTheDocument();
  });

  it('passes through additional props', () => {
    render(<Link href="/test" id="test-id" aria-label="Test link" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveAttribute('id', 'test-id');
    expect(screen.getByTestId('link')).toHaveAttribute('aria-label', 'Test link');
  });

  it('passes through onClick handler', () => {
    const handleClick = vi.fn();
    render(<Link href="/test" onClick={handleClick} data-testid="link">Link</Link>);
    screen.getByTestId('link').click();
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('renders with React node children', () => {
    render(
      <Link href="/test" data-testid="link">
        <span data-testid="child">Child</span>
      </Link>
    );
    expect(screen.getByTestId('child')).toBeInTheDocument();
  });

  it('applies hover color for primary', () => {
    render(<Link href="/test" color="primary" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('hover:text-nest-primary/80');
  });

  it('applies hover color for inherit', () => {
    render(<Link href="/test" color="inherit" data-testid="link">Link</Link>);
    expect(screen.getByTestId('link')).toHaveClass('hover:text-inherit/80');
  });
});
