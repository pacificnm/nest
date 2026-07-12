import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Box } from './Box';

describe('Box', () => {
  it('renders children', () => {
    render(<Box>Test content</Box>);
    expect(screen.getByText('Test content')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Box>Default element</Box>);
    const element = screen.getByText('Default element');
    expect(element.tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Box component="section">Section content</Box>);
    const element = screen.getByText('Section content');
    expect(element.tagName).toBe('SECTION');
  });

  it('renders as span when specified', () => {
    render(<Box component="span">Inline content</Box>);
    const element = screen.getByText('Inline content');
    expect(element.tagName).toBe('SPAN');
  });

  it('renders as article when specified', () => {
    render(<Box component="article">Article content</Box>);
    const element = screen.getByText('Article content');
    expect(element.tagName).toBe('ARTICLE');
  });

  it('applies custom className', () => {
    render(<Box className="custom-class">Custom class</Box>);
    expect(screen.getByText('Custom class')).toHaveClass('custom-class');
  });

  it('applies multiple custom classes', () => {
    render(<Box className="class-a class-b class-c">Multiple classes</Box>);
    expect(screen.getByText('Multiple classes')).toHaveClass('class-a', 'class-b', 'class-c');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Box ref={ref}>Ref test</Box>);
    expect(ref.current).toBeInTheDocument();
  });

  it('forwards ref with custom component', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Box ref={ref} component="main">Main ref test</Box>);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('MAIN');
  });

  it('passes through additional props', () => {
    render(<Box data-testid="box" id="test-id">With props</Box>);
    const element = screen.getByTestId('box');
    expect(element).toHaveAttribute('id', 'test-id');
  });

  it('passes through additional props with custom component', () => {
    render(
      <Box component="nav" aria-label="navigation" data-testid="nav">
        Navigation
      </Box>
    );
    const element = screen.getByTestId('nav');
    expect(element).toHaveAttribute('aria-label', 'navigation');
    expect(element.tagName).toBe('NAV');
  });

  it('supports children with nested elements', () => {
    render(
      <Box>
        <span>Nested 1</span>
        <span>Nested 2</span>
      </Box>
    );
    expect(screen.getByText('Nested 1')).toBeInTheDocument();
    expect(screen.getByText('Nested 2')).toBeInTheDocument();
  });

  it('supports React fragments as children', () => {
    render(
      <Box>
        <>Fragment content</>
      </Box>
    );
    expect(screen.getByText('Fragment content')).toBeInTheDocument();
  });

  it('supports null children', () => {
    const { container } = render(<Box>{null}</Box>);
    // Should not throw, just render empty div
    expect(container.firstChild).toBeInTheDocument();
  });
});
