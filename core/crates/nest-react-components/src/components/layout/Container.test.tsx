import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Container } from './Container';

describe('Container', () => {
  it('renders children', () => {
    render(<Container>Test content</Container>);
    expect(screen.getByText('Test content')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Container>Content</Container>);
    const element = screen.getByText('Content');
    expect(element.tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Container component="main">Main content</Container>);
    const element = screen.getByText('Main content');
    expect(element.tagName).toBe('MAIN');
  });

  it('renders as section', () => {
    render(<Container component="section">Section</Container>);
    expect(screen.getByText('Section').tagName).toBe('SECTION');
  });

  it('applies default maxWidth (lg)', () => {
    render(<Container>Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('max-w-screen-lg');
  });

  it('applies maxWidth sm', () => {
    render(<Container maxWidth="sm">Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('max-w-screen-sm');
  });

  it('applies maxWidth md', () => {
    render(<Container maxWidth="md">Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('max-w-screen-md');
  });

  it('applies maxWidth lg', () => {
    render(<Container maxWidth="lg">Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('max-w-screen-lg');
  });

  it('applies maxWidth xl', () => {
    render(<Container maxWidth="xl">Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('max-w-screen-xl');
  });

  it('applies maxWidth xxl', () => {
    render(<Container maxWidth="xxl">Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('max-w-screen-2xl');
  });

  it('applies maxWidth false (full width)', () => {
    render(<Container maxWidth={false}>Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('max-w-full');
  });

  it('applies default gutters (px-4)', () => {
    render(<Container>Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('px-4');
  });

  it('removes gutters when disableGutters=true', () => {
    render(<Container disableGutters>Content</Container>);
    expect(screen.getByText('Content')).not.toHaveClass('px-4');
  });

  it('applies fixed width when fixed=true', () => {
    render(<Container maxWidth="md" fixed>Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('w-screen-md');
  });

  it('does not apply fixed width when fixed=false', () => {
    render(<Container maxWidth="md" fixed={false}>Content</Container>);
    expect(screen.getByText('Content')).not.toHaveClass('w-screen-md');
  });

  it('applies base styles (mx-auto w-full)', () => {
    render(<Container>Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('mx-auto', 'w-full');
  });

  it('applies custom className', () => {
    render(<Container className="custom-class">Content</Container>);
    expect(screen.getByText('Content')).toHaveClass('custom-class');
  });

  it('combines all classes correctly', () => {
    render(<Container maxWidth="xl" disableGutters className="custom">Content</Container>);
    const element = screen.getByText('Content');
    expect(element).toHaveClass('mx-auto', 'w-full', 'max-w-screen-xl', 'custom');
    expect(element).not.toHaveClass('px-4');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Container ref={ref}>Content</Container>);
    expect(ref.current).toBeInTheDocument();
  });

  it('forwards ref with custom component', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Container ref={ref} component="article">Article</Container>);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('ARTICLE');
  });

  it('renders multiple children', () => {
    render(
      <Container>
        <span data-testid="child-1">Child 1</span>
        <span data-testid="child-2">Child 2</span>
      </Container>
    );
    expect(screen.getByTestId('child-1')).toBeInTheDocument();
    expect(screen.getByTestId('child-2')).toBeInTheDocument();
  });

  it('passes through additional props', () => {
    render(<Container data-testid="container" id="test-id">Content</Container>);
    const element = screen.getByTestId('container');
    expect(element).toHaveAttribute('id', 'test-id');
  });

  it('passes through aria-label', () => {
    render(<Container aria-label="main content">Content</Container>);
    expect(screen.getByText('Content')).toHaveAttribute('aria-label', 'main content');
  });
});
