import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Backdrop } from './Backdrop';

describe('Backdrop', () => {
  it('renders when open is true', () => {
    render(<Backdrop open data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toBeInTheDocument();
  });

  it('does not render when open is false', () => {
    render(<Backdrop open={false} data-testid="backdrop" />);
    expect(screen.queryByTestId('backdrop')).not.toBeInTheDocument();
  });

  it('renders when invisible is true even if open is false', () => {
    render(<Backdrop open={false} invisible data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toBeInTheDocument();
  });

  it('applies invisible styles', () => {
    render(<Backdrop open invisible data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveClass('bg-transparent');
  });

  it('applies visible backdrop styles', () => {
    render(<Backdrop open data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveClass('bg-black/50');
  });

  it('applies opacity-100 when open', () => {
    render(<Backdrop open data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveClass('opacity-100');
  });

  it('applies opacity-0 when not open', () => {
    render(<Backdrop open={false} invisible data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveClass('opacity-0');
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<Backdrop open onClick={handleClick} data-testid="backdrop" />);
    fireEvent.click(screen.getByTestId('backdrop'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies custom className', () => {
    render(<Backdrop open className="custom-class" data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveClass('custom-class');
  });

  it('applies aria-hidden attribute', () => {
    render(<Backdrop open aria-hidden={true} data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveAttribute('aria-hidden', 'true');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Backdrop open ref={ref} data-testid="backdrop" />);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('DIV');
  });

  it('has fixed positioning', () => {
    render(<Backdrop open data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveClass('fixed');
  });

  it('has z-index 50', () => {
    render(<Backdrop open data-testid="backdrop" />);
    expect(screen.getByTestId('backdrop')).toHaveClass('z-50');
  });

  it('renders children', () => {
    render(
      <Backdrop open data-testid="backdrop">
        <span data-testid="child">Child content</span>
      </Backdrop>
    );
    expect(screen.getByTestId('child')).toBeInTheDocument();
  });
});
