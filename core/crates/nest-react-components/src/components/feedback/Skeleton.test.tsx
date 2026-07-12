import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Skeleton } from './Skeleton';

describe('Skeleton', () => {
  it('renders default text skeleton', () => {
    render(<Skeleton data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveClass('h-4', 'rounded', 'animate-pulse');
  });

  it('renders circular variant', () => {
    render(<Skeleton variant="circular" width={40} height={40} data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveClass('rounded-full');
  });

  it('renders rectangular variant', () => {
    render(<Skeleton variant="rectangular" width={200} height={100} data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveClass('rounded-none');
  });

  it('renders rounded variant', () => {
    render(<Skeleton variant="rounded" width={300} height={150} data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveClass('rounded-nest-md');
  });

  it('applies custom width and height', () => {
    render(<Skeleton width={100} height={50} data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveStyle({ width: '100px', height: '50px' });
  });

  it('applies string width and height', () => {
    render(<Skeleton width="50%" height="2rem" data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveStyle({ width: '50%', height: '2rem' });
  });

  it('disables animation when animation=false', () => {
    render(<Skeleton animation={false} data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).not.toHaveClass('animate-pulse');
  });

  it('applies custom className', () => {
    render(<Skeleton className="custom-class" data-testid="skeleton" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toHaveClass('custom-class');
  });

  it('renders custom component', () => {
    render(<Skeleton component="div" data-testid="custom" />);
    const skeleton = screen.getByTestId('custom');
    expect(skeleton.tagName).toBe('DIV');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Skeleton ref={ref} data-testid="skeleton" />);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current).toHaveClass('bg-nest-muted/30');
  });
});
