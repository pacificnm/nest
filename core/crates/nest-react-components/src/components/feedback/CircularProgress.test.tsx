import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CircularProgress } from './CircularProgress';

describe('CircularProgress', () => {
  it('renders without crashing', () => {
    render(<CircularProgress data-testid="progress" />);
    expect(screen.getByTestId('progress')).toBeInTheDocument();
  });

  it('renders as span by default', () => {
    render(<CircularProgress data-testid="progress" />);
    expect(screen.getByTestId('progress').tagName).toBe('SPAN');
  });

  it('renders as custom component', () => {
    render(<CircularProgress component="div" data-testid="progress" />);
    expect(screen.getByTestId('progress').tagName).toBe('DIV');
  });

  it('applies default size (medium)', () => {
    render(<CircularProgress data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('h-8', 'w-8');
  });

  it('applies small size', () => {
    render(<CircularProgress size="small" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('h-4', 'w-4');
  });

  it('applies medium size', () => {
    render(<CircularProgress size="medium" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('h-8', 'w-8');
  });

  it('applies large size', () => {
    render(<CircularProgress size="large" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('h-12', 'w-12');
  });

  it('applies numeric size', () => {
    render(<CircularProgress size={24} data-testid="progress" />);
    const svg = screen.getByTestId('progress').querySelector('svg');
    expect(svg).toHaveAttribute('width', '24');
    expect(svg).toHaveAttribute('height', '24');
  });

  it('applies default color (primary)', () => {
    render(<CircularProgress data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('text-nest-primary');
  });

  it('applies primary color', () => {
    render(<CircularProgress color="primary" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('text-nest-primary');
  });

  it('applies secondary color', () => {
    render(<CircularProgress color="secondary" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('text-nest-secondary');
  });

  it('applies error color', () => {
    render(<CircularProgress color="error" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('text-nest-error');
  });

  it('applies success color', () => {
    render(<CircularProgress color="success" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('text-nest-success');
  });

  it('applies warning color', () => {
    render(<CircularProgress color="warning" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('text-nest-warning');
  });

  it('applies inherit color', () => {
    render(<CircularProgress color="inherit" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('text-inherit');
  });

  it('renders indeterminate variant with spin animation', () => {
    render(<CircularProgress variant="indeterminate" data-testid="progress" />);
    const svg = screen.getByTestId('progress').querySelector('svg');
    expect(svg).toHaveClass('animate-spin');
  });

  it('renders determinate variant without spin animation', () => {
    render(<CircularProgress variant="determinate" value={50} data-testid="progress" />);
    const svg = screen.getByTestId('progress').querySelector('svg');
    expect(svg).not.toHaveClass('animate-spin');
  });

  it('applies role="progressbar"', () => {
    render(<CircularProgress data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveAttribute('role', 'progressbar');
  });

  it('applies aria-valuenow for determinate variant', () => {
    render(<CircularProgress variant="determinate" value={75} data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveAttribute('aria-valuenow', '75');
  });

  it('does not apply aria-valuenow for indeterminate variant', () => {
    render(<CircularProgress variant="indeterminate" data-testid="progress" />);
    expect(screen.getByTestId('progress')).not.toHaveAttribute('aria-valuenow');
  });

  it('renders SVG with correct viewBox', () => {
    render(<CircularProgress size="medium" data-testid="progress" />);
    const svg = screen.getByTestId('progress').querySelector('svg');
    expect(svg).toHaveAttribute('viewBox', '0 0 32 32');
  });

  it('renders background circle', () => {
    render(<CircularProgress data-testid="progress" />);
    const circles = screen.getByTestId('progress').querySelectorAll('circle');
    expect(circles).toHaveLength(2);
    expect(circles[0]).toHaveClass('opacity-25');
  });

  it('renders progress circle for determinate', () => {
    render(<CircularProgress variant="determinate" value={50} data-testid="progress" />);
    const circles = screen.getByTestId('progress').querySelectorAll('circle');
    expect(circles).toHaveLength(2);
    expect(circles[1]).toHaveClass('transition-all', 'duration-300', 'ease-out');
  });

  it('renders indeterminate circle', () => {
    render(<CircularProgress variant="indeterminate" data-testid="progress" />);
    const circles = screen.getByTestId('progress').querySelectorAll('circle');
    expect(circles).toHaveLength(2);
    expect(circles[1]).toHaveClass('opacity-75');
  });

  it('applies custom className', () => {
    render(<CircularProgress className="custom-progress" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('custom-progress');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLSpanElement | null };
    render(<CircularProgress ref={ref} data-testid="progress" />);
    expect(ref.current).toBeInTheDocument();
  });

  it('passes through additional props', () => {
    render(<CircularProgress id="test-id" aria-label="Loading" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveAttribute('id', 'test-id');
    expect(screen.getByTestId('progress')).toHaveAttribute('aria-label', 'Loading');
  });
});
