import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Paper } from './Paper';

describe('Paper', () => {
  it('renders children', () => {
    render(<Paper>Test content</Paper>);
    expect(screen.getByText('Test content')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Paper>Content</Paper>);
    const element = screen.getByText('Content');
    expect(element.tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Paper component="section">Section</Paper>);
    const element = screen.getByText('Section');
    expect(element.tagName).toBe('SECTION');
  });

  it('renders as article', () => {
    render(<Paper component="article">Article</Paper>);
    expect(screen.getByText('Article').tagName).toBe('ARTICLE');
  });

  it('applies base styles', () => {
    render(<Paper>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('bg-nest-surface', 'text-nest-foreground');
  });

  it('applies default elevation (1)', () => {
    render(<Paper>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('shadow-sm');
  });

  it('applies elevation 0', () => {
    render(<Paper elevation={0}>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('shadow-none');
  });

  it('applies elevation 1', () => {
    render(<Paper elevation={1}>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('shadow-sm');
  });

  it('applies elevation 2', () => {
    render(<Paper elevation={2}>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('shadow');
  });

  it('applies elevation 3', () => {
    render(<Paper elevation={3}>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('shadow-md');
  });

  it('applies elevation 4', () => {
    render(<Paper elevation={4}>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('shadow-lg');
  });

  it('applies outlined variant', () => {
    render(<Paper variant="outlined">Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('border', 'border-nest-border');
  });

  it('does not apply shadow when outlined', () => {
    render(<Paper variant="outlined" elevation={4}>Content</Paper>);
    expect(screen.getByText('Content')).not.toHaveClass('shadow-sm', 'shadow', 'shadow-md', 'shadow-lg');
  });

  it('applies default radius (rounded-nest-md)', () => {
    render(<Paper>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('rounded-nest-md');
  });

  it('applies square corners when square=true', () => {
    render(<Paper square>Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('rounded-none');
  });

  it('does not apply rounded-nest-md when square=true', () => {
    render(<Paper square>Content</Paper>);
    expect(screen.getByText('Content')).not.toHaveClass('rounded-nest-md');
  });

  it('applies custom className', () => {
    render(<Paper className="custom-class">Content</Paper>);
    expect(screen.getByText('Content')).toHaveClass('custom-class');
  });

  it('combines all props correctly', () => {
    render(<Paper elevation={3} variant="elevation" square className="custom">Content</Paper>);
    const element = screen.getByText('Content');
    expect(element).toHaveClass('bg-nest-surface', 'text-nest-foreground', 'shadow-md', 'rounded-none', 'custom');
  });

  it('combines outlined with square', () => {
    render(<Paper variant="outlined" square>Content</Paper>);
    const element = screen.getByText('Content');
    expect(element).toHaveClass('bg-nest-surface', 'text-nest-foreground', 'border', 'border-nest-border', 'rounded-none');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Paper ref={ref}>Content</Paper>);
    expect(ref.current).toBeInTheDocument();
  });

  it('forwards ref with custom component', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Paper ref={ref} component="main">Main</Paper>);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('MAIN');
  });

  it('passes through additional props', () => {
    render(<Paper data-testid="paper" id="test-id">Content</Paper>);
    const element = screen.getByTestId('paper');
    expect(element).toHaveAttribute('id', 'test-id');
  });

  it('passes through onClick', () => {
    const handleClick = vi.fn();
    render(<Paper onClick={handleClick}>Clickable</Paper>);
    screen.getByText('Clickable').click();
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('renders multiple children', () => {
    render(
      <Paper>
        <span data-testid="child-1">Child 1</span>
        <span data-testid="child-2">Child 2</span>
      </Paper>
    );
    expect(screen.getByTestId('child-1')).toBeInTheDocument();
    expect(screen.getByTestId('child-2')).toBeInTheDocument();
  });
});
