import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Rating } from './Rating';

describe('Rating', () => {
  it('renders rating with correct number of stars', () => {
    render(<Rating defaultValue={3} max={5} />);
    const stars = screen.getAllByRole('radio');
    expect(stars).toHaveLength(5);
  });

  it('displays correct filled stars', () => {
    render(<Rating defaultValue={3} max={5} />);
    const filledStars = screen.getAllByRole('radio').filter((star) =>
      star.querySelector('svg')?.classList.contains('fill-current')
    );
    expect(filledStars).toHaveLength(3);
  });

  it('calls onChange when star is clicked', () => {
    const handleChange = vi.fn();
    render(<Rating onChange={handleChange} />);
    const stars = screen.getAllByRole('radio');
    fireEvent.click(stars[2]);
    expect(handleChange).toHaveBeenCalledWith(expect.anything(), 3);
  });

  it('respects readOnly prop', () => {
    const handleChange = vi.fn();
    render(<Rating defaultValue={3} readOnly onChange={handleChange} />);
    const stars = screen.getAllByRole('radio');
    fireEvent.click(stars[4]);
    expect(handleChange).not.toHaveBeenCalled();
  });

  it('respects disabled prop', () => {
    const handleChange = vi.fn();
    render(<Rating defaultValue={3} disabled onChange={handleChange} />);
    const stars = screen.getAllByRole('radio');
    expect(stars[0]).toBeDisabled();
    fireEvent.click(stars[4]);
    expect(handleChange).not.toHaveBeenCalled();
  });

  it('applies small size', () => {
    render(<Rating defaultValue={3} size="small" />);
    const stars = screen.getAllByRole('radio');
    expect(stars[0].querySelector('svg')).toHaveClass('size-4');
  });

  it('applies medium size (default)', () => {
    render(<Rating defaultValue={3} />);
    const stars = screen.getAllByRole('radio');
    expect(stars[0].querySelector('svg')).toHaveClass('size-6');
  });

  it('applies large size', () => {
    render(<Rating defaultValue={3} size="large" />);
    const stars = screen.getAllByRole('radio');
    expect(stars[0].querySelector('svg')).toHaveClass('size-8');
  });

  it('applies color styles', () => {
    render(<Rating defaultValue={3} color="primary" />);
    const filledStar = screen.getAllByRole('radio')[0];
    expect(filledStar.querySelector('svg')).toHaveClass('text-nest-primary');
  });

  it('applies warning color (default)', () => {
    render(<Rating defaultValue={3} />);
    const filledStar = screen.getAllByRole('radio')[0];
    expect(filledStar.querySelector('svg')).toHaveClass('text-nest-warning');
  });

  it('supports custom max value', () => {
    render(<Rating defaultValue={7} max={10} />);
    const stars = screen.getAllByRole('radio');
    expect(stars).toHaveLength(10);
  });

  it('supports precision (half stars)', () => {
    render(<Rating defaultValue={3.5} precision={0.5} />);
    // Should show 3 full stars and 1 half star
    const halfStars = screen.getAllByLabelText('4 stars');
    expect(halfStars[0]?.querySelector('svg')).toHaveClass('lucide-star-half');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Rating ref={ref} defaultValue={3} />);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.role).toBe('radiogroup');
  });

  it('has radiogroup role', () => {
    render(<Rating defaultValue={3} />);
    expect(screen.getByRole('radiogroup')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    render(<Rating defaultValue={3} className="custom-class" />);
    expect(screen.getByRole('radiogroup')).toHaveClass('custom-class');
  });
});
