import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Typography } from './Typography';

describe('Typography', () => {
  it('renders children', () => {
    render(<Typography>Test text</Typography>);
    expect(screen.getByText('Test text')).toBeInTheDocument();
  });

  it('renders with default variant (body1) as a paragraph', () => {
    render(<Typography>Body text</Typography>);
    expect(screen.getByText('Body text')).toBeInTheDocument();
  });

  it('renders h1 variant as heading', () => {
    render(<Typography variant="h1">Heading 1</Typography>);
    const element = screen.getByText('Heading 1');
    expect(element.tagName).toBe('H1');
    expect(element).toHaveClass('text-4xl', 'font-bold');
  });

  it('renders h2 variant with correct styles', () => {
    render(<Typography variant="h2">Heading 2</Typography>);
    const element = screen.getByText('Heading 2');
    expect(element.tagName).toBe('H2');
    expect(element).toHaveClass('text-3xl', 'font-bold');
  });

  it('renders h3 variant with correct styles', () => {
    render(<Typography variant="h3">Heading 3</Typography>);
    const element = screen.getByText('Heading 3');
    expect(element.tagName).toBe('H3');
    expect(element).toHaveClass('text-2xl', 'font-semibold');
  });

  it('renders h4 variant with correct styles', () => {
    render(<Typography variant="h4">Heading 4</Typography>);
    const element = screen.getByText('Heading 4');
    expect(element.tagName).toBe('H4');
    expect(element).toHaveClass('text-xl', 'font-semibold');
  });

  it('renders h5 variant with correct styles', () => {
    render(<Typography variant="h5">Heading 5</Typography>);
    const element = screen.getByText('Heading 5');
    expect(element.tagName).toBe('H5');
    expect(element).toHaveClass('text-lg', 'font-medium');
  });

  it('renders h6 variant with correct styles', () => {
    render(<Typography variant="h6">Heading 6</Typography>);
    const element = screen.getByText('Heading 6');
    expect(element.tagName).toBe('H6');
    expect(element).toHaveClass('text-base', 'font-medium');
  });

  it('renders subtitle1 with correct styles', () => {
    render(<Typography variant="subtitle1">Subtitle 1</Typography>);
    const element = screen.getByText('Subtitle 1');
    expect(element).toHaveClass('text-lg', 'text-nest-muted');
  });

  it('renders subtitle2 with correct styles', () => {
    render(<Typography variant="subtitle2">Subtitle 2</Typography>);
    const element = screen.getByText('Subtitle 2');
    expect(element).toHaveClass('text-base', 'text-nest-muted');
  });

  it('renders body1 with correct styles', () => {
    render(<Typography variant="body1">Body 1</Typography>);
    const element = screen.getByText('Body 1');
    expect(element).toHaveClass('text-sm');
  });

  it('renders body2 with correct styles', () => {
    render(<Typography variant="body2">Body 2</Typography>);
    const element = screen.getByText('Body 2');
    expect(element).toHaveClass('text-xs');
  });

  it('renders caption with correct styles', () => {
    render(<Typography variant="caption">Caption</Typography>);
    const element = screen.getByText('Caption');
    expect(element).toHaveClass('text-xs', 'text-nest-muted');
  });

  it('renders overline with correct styles', () => {
    render(<Typography variant="overline">Overline</Typography>);
    const element = screen.getByText('Overline');
    expect(element).toHaveClass('text-xs', 'uppercase', 'tracking-wide');
  });

  it('applies color styles', () => {
    render(
      <>
        <Typography color="primary" data-testid="primary">Primary</Typography>
        <Typography color="secondary" data-testid="secondary">Secondary</Typography>
        <Typography color="error" data-testid="error">Error</Typography>
        <Typography color="success" data-testid="success">Success</Typography>
        <Typography color="warning" data-testid="warning">Warning</Typography>
        <Typography color="muted" data-testid="muted">Muted</Typography>
      </>
    );
    expect(screen.getByTestId('primary')).toHaveClass('text-nest-primary');
    expect(screen.getByTestId('secondary')).toHaveClass('text-nest-secondary');
    expect(screen.getByTestId('error')).toHaveClass('text-nest-error');
    expect(screen.getByTestId('success')).toHaveClass('text-nest-success');
    expect(screen.getByTestId('warning')).toHaveClass('text-nest-warning');
    expect(screen.getByTestId('muted')).toHaveClass('text-nest-muted');
  });

  it('applies align styles', () => {
    render(
      <>
        <Typography align="left" data-testid="left">Left</Typography>
        <Typography align="center" data-testid="center">Center</Typography>
        <Typography align="right" data-testid="right">Right</Typography>
        <Typography align="justify" data-testid="justify">Justify</Typography>
      </>
    );
    expect(screen.getByTestId('left')).toHaveClass('text-left');
    expect(screen.getByTestId('center')).toHaveClass('text-center');
    expect(screen.getByTestId('right')).toHaveClass('text-right');
    expect(screen.getByTestId('justify')).toHaveClass('text-justify');
  });

  it('applies gutterBottom class', () => {
    render(<Typography gutterBottom>With margin</Typography>);
    expect(screen.getByText('With margin')).toHaveClass('mb-2');
  });

  it('applies noWrap class (truncate)', () => {
    render(<Typography noWrap>Truncated text</Typography>);
    expect(screen.getByText('Truncated text')).toHaveClass('truncate');
  });

  it('renders with custom component', () => {
    render(<Typography component="span">Custom element</Typography>);
    const element = screen.getByText('Custom element');
    expect(element.tagName).toBe('SPAN');
  });

  it('renders h1 as span when component override provided', () => {
    render(<Typography variant="h1" component="span">Override</Typography>);
    const element = screen.getByText('Override');
    expect(element.tagName).toBe('SPAN');
    expect(element).toHaveClass('text-4xl', 'font-bold');
  });

  it('applies custom className', () => {
    render(<Typography className="custom-class">Custom class</Typography>);
    expect(screen.getByText('Custom class')).toHaveClass('custom-class');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Typography ref={ref}>Ref test</Typography>);
    expect(ref.current).toBeInTheDocument();
  });
});
