import { render, screen, fireEvent } from '@testing-library/react';
import { Slider } from './Slider';
import { describe, it, expect, vi } from 'vitest';

describe('Slider', () => {
  it('renders a basic slider', () => {
    render(<Slider defaultValue={50} min={0} max={100} />);
    expect(screen.getByRole('slider')).toBeInTheDocument();
  });

  it('renders with default value', () => {
    render(<Slider defaultValue={30} min={0} max={100} />);
    const slider = screen.getByRole('slider');
    expect(slider).toHaveValue('30');
  });

  it('calls onChange when value changes', () => {
    const onChange = vi.fn();
    render(<Slider defaultValue={50} min={0} max={100} onChange={onChange} />);
    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '75' } });
    expect(onChange).toHaveBeenCalledWith(75);
  });

  it('supports range values', () => {
    render(<Slider defaultValue={[20, 80]} min={0} max={100} />);
    const sliders = screen.getAllByRole('slider');
    expect(sliders).toHaveLength(2);
  });

  it('supports vertical orientation', () => {
    render(<Slider defaultValue={50} min={0} max={100} orientation="vertical" />);
    const container = screen.getByRole('slider').closest('.relative');
    expect(container).toHaveClass('h-48');
  });

  it('supports horizontal orientation (default)', () => {
    render(<Slider defaultValue={50} min={0} max={100} />);
    const container = screen.getByRole('slider').closest('.relative');
    expect(container).toHaveClass('w-full');
  });

  it('applies disabled styles', () => {
    render(<Slider defaultValue={50} min={0} max={100} disabled />);
    const container = screen.getByRole('slider').closest('.relative');
    expect(container).toHaveClass('opacity-50');
    expect(container).toHaveClass('cursor-not-allowed');
  });

  it('shows tick marks when showTicks is true', () => {
    render(<Slider defaultValue={50} min={0} max={100} step={10} showTicks />);
    const ticks = screen.getAllByRole('slider')[0].closest('.relative')?.querySelectorAll('.w-1.h-1');
    expect(ticks).toHaveLength(11);
  });

  it('applies custom className', () => {
    render(<Slider defaultValue={50} min={0} max={100} className="custom-slider" />);
    const container = screen.getByRole('slider').closest('.relative');
    expect(container).toHaveClass('custom-slider');
  });

  it('applies custom track className', () => {
    render(<Slider defaultValue={50} min={0} max={100} trackClassName="custom-track" />);
    // The track is a sibling subtree of the thumb/input, so find it within the container.
    const container = screen.getByRole('slider').closest('.relative');
    const track = container?.querySelector('.bg-nest-border');
    expect(track).toHaveClass('custom-track');
  });

  it('applies custom thumb className', () => {
    render(<Slider defaultValue={50} min={0} max={100} thumbClassName="custom-thumb" />);
    const thumb = screen.getByRole('slider').nextElementSibling;
    expect(thumb).toHaveClass('custom-thumb');
  });

  it('renders with label', () => {
    render(<Slider defaultValue={50} min={0} max={100} label="Volume" />);
    expect(screen.getByText('Volume')).toBeInTheDocument();
  });

  it('renders thumb with custom label', () => {
    render(<Slider defaultValue={[20, 80]} min={0} max={100} thumbLabels={['Min', 'Max']} />);
    const sliders = screen.getAllByRole('slider');
    expect(sliders[0]).toHaveAttribute('aria-label', 'Min');
    expect(sliders[1]).toHaveAttribute('aria-label', 'Max');
  });
});
