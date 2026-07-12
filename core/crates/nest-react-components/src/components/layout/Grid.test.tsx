import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Grid } from './Grid';

describe('Grid', () => {
  it('renders children', () => {
    render(<Grid>Test content</Grid>);
    expect(screen.getByText('Test content')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Grid>Content</Grid>);
    const element = screen.getByText('Content');
    expect(element.tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Grid component="section">Section</Grid>);
    const element = screen.getByText('Section');
    expect(element.tagName).toBe('SECTION');
  });

  // Container tests
  describe('container', () => {
    it('applies grid class when container=true', () => {
      render(<Grid container>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('grid');
    });

    it('applies default columns (12) when container=true', () => {
      render(<Grid container>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('grid-cols-12');
    });

    it('applies custom columns', () => {
      render(<Grid container columns={6}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('grid-cols-6');
    });

    it('applies spacing 0', () => {
      render(<Grid container spacing={0}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('gap-0');
    });

    it('applies spacing 2', () => {
      render(<Grid container spacing={2}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('gap-2');
    });

    it('applies spacing 4', () => {
      render(<Grid container spacing={4}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('gap-4');
    });

    it('applies spacing 8', () => {
      render(<Grid container spacing={8}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('gap-8');
    });

    it('combines container classes', () => {
      render(<Grid container spacing={3} columns={6}>Content</Grid>);
      const element = screen.getByText('Content');
      expect(element).toHaveClass('grid', 'grid-cols-6', 'gap-3');
    });
  });

  // Size tests
  describe('size', () => {
    it('applies size as number', () => {
      render(<Grid size={6}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-span-6');
    });

    it('applies size 1', () => {
      render(<Grid size={1}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-span-1');
    });

    it('applies size 12', () => {
      render(<Grid size={12}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-span-12');
    });

    it('applies size auto', () => {
      render(<Grid size="auto">Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-auto');
    });

    it('applies responsive size with xs only', () => {
      render(<Grid size={{ xs: 6 }}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-span-6');
    });

    it('applies responsive size with xs and md', () => {
      render(<Grid size={{ xs: 12, md: 6 }}>Content</Grid>);
      const element = screen.getByText('Content');
      expect(element).toHaveClass('col-span-12', 'md:col-span-6');
    });

    it('applies responsive size with all breakpoints', () => {
      render(<Grid size={{ xs: 12, sm: 8, md: 6, lg: 4 }}>Content</Grid>);
      const element = screen.getByText('Content');
      expect(element).toHaveClass('col-span-12', 'sm:col-span-8', 'md:col-span-6', 'lg:col-span-4');
    });

    it('applies responsive size with auto', () => {
      render(<Grid size={{ xs: 'auto', md: 6 }}>Content</Grid>);
      const element = screen.getByText('Content');
      expect(element).toHaveClass('col-auto', 'md:col-span-6');
    });
  });

  // Offset tests
  describe('offset', () => {
    it('applies offset as number', () => {
      render(<Grid offset={3}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-start-4');
    });

    it('applies offset 0', () => {
      render(<Grid offset={0}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-start-1');
    });

    it('applies offset 6', () => {
      render(<Grid offset={6}>Content</Grid>);
      expect(screen.getByText('Content')).toHaveClass('col-start-7');
    });

    it('applies responsive offset', () => {
      render(<Grid offset={{ xs: 0, md: 3 }}>Content</Grid>);
      const element = screen.getByText('Content');
      expect(element).toHaveClass('col-start-1', 'md:col-start-4');
    });
  });

  // Combined tests
  it('combines container and size correctly', () => {
    render(
      <Grid container spacing={2}>
        <Grid size={6} data-testid="item">Item</Grid>
      </Grid>
    );
    const item = screen.getByTestId('item');
    expect(item).toHaveClass('col-span-6');
    expect(item).not.toHaveClass('grid'); // Item itself is not a grid
  });

  it('combines size and offset', () => {
    render(<Grid size={6} offset={3}>Content</Grid>);
    const element = screen.getByText('Content');
    expect(element).toHaveClass('col-span-6', 'col-start-4');
  });

  // Custom className
  it('applies custom className', () => {
    render(<Grid className="custom-class">Content</Grid>);
    expect(screen.getByText('Content')).toHaveClass('custom-class');
  });

  it('applies custom className with container', () => {
    render(<Grid container className="custom-grid">Content</Grid>);
    expect(screen.getByText('Content')).toHaveClass('custom-grid');
  });

  // Ref forwarding
  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Grid ref={ref}>Content</Grid>);
    expect(ref.current).toBeInTheDocument();
  });

  // Multiple children in container
  it('renders multiple children in container', () => {
    render(
      <Grid container spacing={2}>
        <span data-testid="child-1">Child 1</span>
        <span data-testid="child-2">Child 2</span>
      </Grid>
    );
    expect(screen.getByTestId('child-1')).toBeInTheDocument();
    expect(screen.getByTestId('child-2')).toBeInTheDocument();
  });
});
