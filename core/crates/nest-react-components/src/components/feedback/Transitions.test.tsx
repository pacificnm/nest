import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Fade, Grow, Collapse } from './Transitions';

describe('Fade', () => {
  it('renders children when in is true', () => {
    render(<Fade in><div data-testid="content">Hello</div></Fade>);
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('does not render children when in is false and unmountOnExit is true', () => {
    render(<Fade in={false} unmountOnExit><div data-testid="content">Hello</div></Fade>);
    expect(screen.queryByTestId('content')).not.toBeInTheDocument();
  });

  it('renders children when in is false (before exit transition)', () => {
    render(<Fade in={false}><div data-testid="content">Hello</div></Fade>);
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('applies opacity transition classes when in is true', () => {
    const { container } = render(<Fade in><div>Content</div></Fade>);
    expect(container.firstChild).toHaveClass('opacity-100');
  });

  it('applies opacity transition classes when in is false', () => {
    const { container } = render(<Fade in={false}><div>Content</div></Fade>);
    expect(container.firstChild).toHaveClass('opacity-0');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Fade ref={ref} in><div>Content</div></Fade>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('Grow', () => {
  it('renders children when in is true', () => {
    render(<Grow in><div data-testid="content">Hello</div></Grow>);
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('does not render children when in is false and unmountOnExit is true', () => {
    render(<Grow in={false} unmountOnExit><div data-testid="content">Hello</div></Grow>);
    expect(screen.queryByTestId('content')).not.toBeInTheDocument();
  });

  it('applies scale and opacity transition classes when in is true', () => {
    const { container } = render(<Grow in><div>Content</div></Grow>);
    expect(container.firstChild).toHaveClass('opacity-100', 'scale-100');
  });

  it('applies scale and opacity transition classes when in is false', () => {
    const { container } = render(<Grow in={false}><div>Content</div></Grow>);
    expect(container.firstChild).toHaveClass('opacity-0', 'scale-95');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Grow ref={ref} in><div>Content</div></Grow>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('Collapse', () => {
  it('renders children when in is true', () => {
    render(<Collapse in><div data-testid="content">Hello</div></Collapse>);
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });

  it('does not render children when in is false and unmountOnExit is true', () => {
    render(<Collapse in={false} unmountOnExit><div data-testid="content">Hello</div></Collapse>);
    expect(screen.queryByTestId('content')).not.toBeInTheDocument();
  });

  it('applies vertical collapse classes when in is true', () => {
    const { container } = render(<Collapse in><div>Content</div></Collapse>);
    expect(container.firstChild).toHaveClass('grid-rows-[1fr]', 'opacity-100');
  });

  it('applies vertical collapse classes when in is false', () => {
    const { container } = render(<Collapse in={false}><div>Content</div></Collapse>);
    expect(container.firstChild).toHaveClass('grid-rows-[0fr]', 'opacity-0');
  });

  it('applies horizontal collapse classes', () => {
    const { container } = render(<Collapse in={false} orientation="horizontal"><div>Content</div></Collapse>);
    expect(container.firstChild).toHaveClass('w-0', 'opacity-0');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Collapse ref={ref} in><div>Content</div></Collapse>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('Transitions integration', () => {
  it('can chain transitions', () => {
    render(
      <Fade in>
        <Grow in>
          <div data-testid="nested">Nested</div>
        </Grow>
      </Fade>
    );
    expect(screen.getByTestId('nested')).toBeInTheDocument();
  });

  it('works with Collapse inside Fade', () => {
    render(
      <Fade in>
        <Collapse in>
          <div data-testid="content">Content</div>
        </Collapse>
      </Fade>
    );
    expect(screen.getByTestId('content')).toBeInTheDocument();
  });
});
