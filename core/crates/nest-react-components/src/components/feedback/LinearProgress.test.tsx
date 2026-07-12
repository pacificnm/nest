import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LinearProgress } from './LinearProgress';

describe('LinearProgress', () => {
  it('renders without crashing', () => {
    render(<LinearProgress data-testid="progress" />);
    expect(screen.getByTestId('progress')).toBeInTheDocument();
  });

  it('renders as span by default', () => {
    render(<LinearProgress data-testid="progress" />);
    expect(screen.getByTestId('progress').tagName).toBe('SPAN');
  });

  it('renders as custom component', () => {
    render(<LinearProgress component="div" data-testid="progress" />);
    expect(screen.getByTestId('progress').tagName).toBe('DIV');
  });

  it('applies base styles', () => {
    render(<LinearProgress data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass(
      'relative',
      'h-1',
      'w-full',
      'rounded-nest-full',
      'bg-nest-surface',
      'overflow-hidden'
    );
  });

  it('applies default color (primary)', () => {
    render(<LinearProgress data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('bg-nest-primary');
  });

  it('applies primary color', () => {
    render(<LinearProgress color="primary" data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('bg-nest-primary');
  });

  it('applies secondary color', () => {
    render(<LinearProgress color="secondary" data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('bg-nest-secondary');
  });

  it('applies error color', () => {
    render(<LinearProgress color="error" data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('bg-nest-error');
  });

  it('applies success color', () => {
    render(<LinearProgress color="success" data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('bg-nest-success');
  });

  it('applies warning color', () => {
    render(<LinearProgress color="warning" data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('bg-nest-warning');
  });

  it('applies inherit color', () => {
    render(<LinearProgress color="inherit" data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('bg-inherit');
  });

  it('renders indeterminate variant with two bars', () => {
    render(<LinearProgress variant="indeterminate" data-testid="progress" />);
    const bars = screen.getByTestId('progress').querySelectorAll('div');
    expect(bars).toHaveLength(2);
  });

  it('renders determinate variant with one bar', () => {
    render(<LinearProgress variant="determinate" value={50} data-testid="progress" />);
    const bars = screen.getByTestId('progress').querySelectorAll('div');
    expect(bars).toHaveLength(1);
  });

  it('renders buffer variant with two bars', () => {
    render(<LinearProgress variant="buffer" value={50} bufferValue={75} data-testid="progress" />);
    const bars = screen.getByTestId('progress').querySelectorAll('div');
    expect(bars).toHaveLength(2);
  });

  it('applies correct width for determinate', () => {
    render(<LinearProgress variant="determinate" value={75} data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveStyle('width: 75%');
  });

  it('applies correct width for buffer', () => {
    render(<LinearProgress variant="buffer" value={50} bufferValue={80} data-testid="progress" />);
    const bars = screen.getByTestId('progress').querySelectorAll('div');
    expect(bars[0]).toHaveStyle('width: 80%');
    expect(bars[1]).toHaveStyle('width: 50%');
  });

  it('applies transition styles for determinate', () => {
    render(<LinearProgress variant="determinate" value={50} data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveClass('transition-all', 'duration-300', 'ease-out');
  });

  it('applies animation for indeterminate', () => {
    render(<LinearProgress variant="indeterminate" data-testid="progress" />);
    const bars = screen.getByTestId('progress').querySelectorAll('div');
    expect(bars[0]).toHaveClass('animate-[linear-progress-indeterminate1_2.1s_cubic-bezier(0.65,0,0.35,1)_infinite]');
  });

  it('applies role="progressbar"', () => {
    render(<LinearProgress data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveAttribute('role', 'progressbar');
  });

  it('applies aria-valuenow for determinate variant', () => {
    render(<LinearProgress variant="determinate" value={75} data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveAttribute('aria-valuenow', '75');
  });

  it('applies aria-valuenow for buffer variant', () => {
    render(<LinearProgress variant="buffer" value={50} bufferValue={75} data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveAttribute('aria-valuenow', '50');
  });

  it('does not apply aria-valuenow for indeterminate variant', () => {
    render(<LinearProgress variant="indeterminate" data-testid="progress" />);
    expect(screen.getByTestId('progress')).not.toHaveAttribute('aria-valuenow');
  });

  it('applies custom className', () => {
    render(<LinearProgress className="custom-progress" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveClass('custom-progress');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLSpanElement | null };
    render(<LinearProgress ref={ref} data-testid="progress" />);
    expect(ref.current).toBeInTheDocument();
  });

  it('passes through additional props', () => {
    render(<LinearProgress id="test-id" aria-label="Loading" data-testid="progress" />);
    expect(screen.getByTestId('progress')).toHaveAttribute('id', 'test-id');
    expect(screen.getByTestId('progress')).toHaveAttribute('aria-label', 'Loading');
  });

  it('renders with 0% value', () => {
    render(<LinearProgress variant="determinate" value={0} data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveStyle('width: 0%');
  });

  it('renders with 100% value', () => {
    render(<LinearProgress variant="determinate" value={100} data-testid="progress" />);
    const bar = screen.getByTestId('progress').querySelector('div');
    expect(bar).toHaveStyle('width: 100%');
  });
});
