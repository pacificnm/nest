import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Stack } from './Stack';

describe('Stack', () => {
  it('renders children', () => {
    render(<Stack>Test content</Stack>);
    expect(screen.getByText('Test content')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Stack>Default element</Stack>);
    const element = screen.getByText('Default element');
    expect(element.tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Stack component="section">Section</Stack>);
    const element = screen.getByText('Section');
    expect(element.tagName).toBe('SECTION');
  });

  it('applies default direction (column)', () => {
    render(<Stack>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('flex-col');
  });

  it('applies row direction', () => {
    render(<Stack direction="row">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('flex-row');
  });

  it('applies column direction explicitly', () => {
    render(<Stack direction="column">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('flex-col');
  });

  it('applies default spacing (2)', () => {
    render(<Stack>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-2');
  });

  it('applies spacing 0', () => {
    render(<Stack spacing={0}>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-0');
  });

  it('applies spacing 1', () => {
    render(<Stack spacing={1}>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-1');
  });

  it('applies spacing 3', () => {
    render(<Stack spacing={3}>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-3');
  });

  it('applies spacing 4', () => {
    render(<Stack spacing={4}>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-4');
  });

  it('applies spacing 5', () => {
    render(<Stack spacing={5}>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-5');
  });

  it('applies spacing 6', () => {
    render(<Stack spacing={6}>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-6');
  });

  it('applies spacing 8', () => {
    render(<Stack spacing={8}>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('gap-8');
  });

  it('applies default align (stretch)', () => {
    render(<Stack>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('items-stretch');
  });

  it('applies align start', () => {
    render(<Stack align="start">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('items-start');
  });

  it('applies align center', () => {
    render(<Stack align="center">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('items-center');
  });

  it('applies align end', () => {
    render(<Stack align="end">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('items-end');
  });

  it('applies default justify (start)', () => {
    render(<Stack>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('justify-start');
  });

  it('applies justify center', () => {
    render(<Stack justify="center">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('justify-center');
  });

  it('applies justify end', () => {
    render(<Stack justify="end">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('justify-end');
  });

  it('applies justify between', () => {
    render(<Stack justify="between">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('justify-between');
  });

  it('applies justify around', () => {
    render(<Stack justify="around">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('justify-around');
  });

  it('applies wrap when true', () => {
    render(<Stack wrap>Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('flex-wrap');
  });

  it('does not apply wrap when false', () => {
    render(<Stack wrap={false}>Content</Stack>);
    expect(screen.getByText('Content')).not.toHaveClass('flex-wrap');
  });

  it('applies custom className', () => {
    render(<Stack className="custom-class">Content</Stack>);
    expect(screen.getByText('Content')).toHaveClass('custom-class');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Stack ref={ref}>Content</Stack>);
    expect(ref.current).toBeInTheDocument();
  });

  it('renders multiple children with spacing', () => {
    render(
      <Stack spacing={4}>
        <span data-testid="child-1">Child 1</span>
        <span data-testid="child-2">Child 2</span>
        <span data-testid="child-3">Child 3</span>
      </Stack>
    );
    expect(screen.getByTestId('child-1')).toBeInTheDocument();
    expect(screen.getByTestId('child-2')).toBeInTheDocument();
    expect(screen.getByTestId('child-3')).toBeInTheDocument();
    expect(screen.getByTestId('child-1').parentElement).toHaveClass('gap-4');
  });

  it('combines all props correctly', () => {
    render(
      <Stack direction="row" spacing={3} align="center" justify="between" wrap>
        Content
      </Stack>
    );
    const element = screen.getByText('Content');
    expect(element).toHaveClass('flex', 'flex-row', 'gap-3', 'items-center', 'justify-between', 'flex-wrap');
  });
});
