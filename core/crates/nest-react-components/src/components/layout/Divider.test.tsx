import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Divider } from './Divider';

describe('Divider', () => {
  it('renders without children', () => {
    render(<Divider data-testid="divider" />);
    expect(screen.getByTestId('divider')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Divider data-testid="divider">Content</Divider>);
    expect(screen.getByTestId('divider').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Divider component="hr" data-testid="divider" />);
    expect(screen.getByTestId('divider').tagName).toBe('HR');
  });

  it('applies horizontal orientation by default', () => {
    render(<Divider data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('border-t');
  });

  it('applies horizontal orientation explicitly', () => {
    render(<Divider orientation="horizontal" data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('border-t');
  });

  it('applies vertical orientation', () => {
    render(<Divider orientation="vertical" data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('border-l', 'h-full');
  });

  it('applies fullWidth by default (horizontal)', () => {
    render(<Divider data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('w-full');
  });

  it('applies fullWidth when true', () => {
    render(<Divider fullWidth data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('w-full');
  });

  it('does not apply fullWidth when false', () => {
    render(<Divider fullWidth={false} data-testid="divider" />);
    expect(screen.getByTestId('divider')).not.toHaveClass('w-full');
  });

  it('applies flexItem when true', () => {
    render(<Divider flexItem data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('flex-shrink-0');
  });

  it('does not apply flexItem when false', () => {
    render(<Divider flexItem={false} data-testid="divider" />);
    expect(screen.getByTestId('divider')).not.toHaveClass('flex-shrink-0');
  });

  it('renders children (text divider)', () => {
    render(<Divider data-testid="divider">OR</Divider>);
    expect(screen.getByText('OR')).toBeInTheDocument();
  });

  it('renders children with flex container', () => {
    render(<Divider data-testid="divider">Continue</Divider>);
    const divider = screen.getByTestId('divider');
    expect(divider).toHaveClass('flex', 'items-center');
  });

  it('renders text divider with two border lines', () => {
    render(<Divider data-testid="divider">Text</Divider>);
    const divider = screen.getByTestId('divider');
    const spans = divider.querySelectorAll('span');
    // 3 spans: left border, text content, right border
    expect(spans).toHaveLength(3);
    expect(spans[0]).toHaveClass('border-t', 'border-nest-border', 'flex-1');
    expect(spans[1]).toHaveClass('px-4', 'text-sm', 'text-nest-muted');
    expect(spans[2]).toHaveClass('border-t', 'border-nest-border', 'flex-1');
  });

  it('renders text divider with centered text', () => {
    render(<Divider data-testid="divider">Section</Divider>);
    expect(screen.getByText('Section')).toHaveClass('px-4', 'text-sm', 'text-nest-muted');
  });

  it('renders with icon children', () => {
    render(
      <Divider data-testid="divider">
        <span data-testid="icon">★</span>
      </Divider>
    );
    expect(screen.getByTestId('icon')).toBeInTheDocument();
    // The icon is wrapped in a span with the text styling classes
    const iconWrapper = screen.getByTestId('icon').parentElement;
    expect(iconWrapper).toHaveClass('px-4', 'text-sm', 'text-nest-muted');
  });

  it('applies base border class', () => {
    render(<Divider data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('border-nest-border');
  });

  it('applies custom className', () => {
    render(<Divider className="custom-class" data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveClass('custom-class');
  });

  it('applies role="separator"', () => {
    render(<Divider data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveAttribute('role', 'separator');
  });

  it('applies aria-orientation="vertical" for vertical divider', () => {
    render(<Divider orientation="vertical" data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveAttribute('aria-orientation', 'vertical');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Divider ref={ref} data-testid="divider" />);
    expect(ref.current).toBeInTheDocument();
  });

  it('passes through additional props', () => {
    render(<Divider id="test-id" data-testid="divider" />);
    expect(screen.getByTestId('divider')).toHaveAttribute('id', 'test-id');
  });

  it('combines all props correctly (horizontal with children)', () => {
    render(<Divider className="custom" flexItem data-testid="divider">Text</Divider>);
    const element = screen.getByTestId('divider') as HTMLElement;
    expect(element).toHaveClass('border-nest-border', 'flex', 'items-center', 'w-full', 'flex-shrink-0', 'custom');
  });

  it('combines all props correctly (vertical)', () => {
    render(<Divider orientation="vertical" className="custom" flexItem data-testid="divider" />);
    const element = screen.getByTestId('divider');
    expect(element).toHaveClass('border-nest-border', 'border-l', 'h-full', 'flex-shrink-0', 'custom');
  });
});
