import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { FormControl, FormLabel, FormHelperText } from './FormControl';

describe('FormControl', () => {
  it('renders children', () => {
    render(
      <FormControl>
        <div data-testid="child">Content</div>
      </FormControl>
    );
    expect(screen.getByTestId('child')).toBeInTheDocument();
  });

  it('applies error state styles', () => {
    render(
      <FormControl error data-testid="control">
        <div>Content</div>
      </FormControl>
    );
    const control = screen.getByTestId('control');
    expect(control).toHaveClass('flex', 'flex-col', 'gap-1');
  });

  it('applies disabled state', () => {
    render(
      <FormControl disabled data-testid="control">
        <div>Content</div>
      </FormControl>
    );
    const control = screen.getByTestId('control');
    expect(control).toHaveClass('opacity-60', 'pointer-events-none');
  });

  it('applies fullWidth', () => {
    render(
      <FormControl fullWidth data-testid="control">
        <div>Content</div>
      </FormControl>
    );
    const control = screen.getByTestId('control');
    expect(control).toHaveClass('w-full');
  });

  it('renders custom component', () => {
    render(
      <FormControl component="section" data-testid="control">
        <div>Content</div>
      </FormControl>
    );
    const control = screen.getByTestId('control');
    expect(control.tagName).toBe('SECTION');
  });

  it('applies custom className', () => {
    render(
      <FormControl className="custom-class" data-testid="control">
        <div>Content</div>
      </FormControl>
    );
    const control = screen.getByTestId('control');
    expect(control).toHaveClass('custom-class');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <FormControl ref={ref} data-testid="control">
        <div>Content</div>
      </FormControl>
    );
    expect(ref.current).toBeInTheDocument();
  });
});

describe('FormLabel', () => {
  it('renders children', () => {
    render(<FormLabel>Label Text</FormLabel>);
    expect(screen.getByText('Label Text')).toBeInTheDocument();
  });

  it('applies base styles', () => {
    render(<FormLabel data-testid="label">Label</FormLabel>);
    const label = screen.getByTestId('label');
    expect(label).toHaveClass('text-sm', 'font-medium', 'text-nest-foreground');
  });

  it('applies error state', () => {
    render(<FormLabel error data-testid="label">Label</FormLabel>);
    const label = screen.getByTestId('label');
    expect(label).toHaveClass('text-nest-error');
  });

  it('applies disabled state', () => {
    render(<FormLabel disabled data-testid="label">Label</FormLabel>);
    const label = screen.getByTestId('label');
    expect(label).toHaveClass('text-nest-muted', 'cursor-not-allowed');
  });

  it('applies focused state', () => {
    render(<FormLabel focused data-testid="label">Label</FormLabel>);
    const label = screen.getByTestId('label');
    expect(label).toHaveClass('text-nest-primary');
  });

  it('renders required asterisk', () => {
    render(<FormLabel required data-testid="label">Label</FormLabel>);
    const label = screen.getByTestId('label');
    expect(label).toHaveTextContent('*');
  });

  it('applies htmlFor attribute', () => {
    render(<FormLabel htmlFor="test-input" data-testid="label">Label</FormLabel>);
    const label = screen.getByTestId('label');
    expect(label).toHaveAttribute('for', 'test-input');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLLabelElement | null };
    render(<FormLabel ref={ref}>Label</FormLabel>);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('LABEL');
  });
});

describe('FormHelperText', () => {
  it('renders children', () => {
    render(<FormHelperText>Helper text</FormHelperText>);
    expect(screen.getByText('Helper text')).toBeInTheDocument();
  });

  it('applies base styles', () => {
    render(<FormHelperText data-testid="helper">Helper</FormHelperText>);
    const helper = screen.getByTestId('helper');
    expect(helper).toHaveClass('text-xs', 'text-nest-muted', 'mt-0.5');
  });

  it('applies error state', () => {
    render(<FormHelperText error data-testid="helper">Error text</FormHelperText>);
    const helper = screen.getByTestId('helper');
    expect(helper).toHaveClass('text-nest-error');
  });

  it('applies disabled state', () => {
    render(<FormHelperText disabled data-testid="helper">Disabled</FormHelperText>);
    const helper = screen.getByTestId('helper');
    expect(helper).toHaveClass('opacity-60');
  });

  it('applies visually hidden', () => {
    render(<FormHelperText visuallyHidden data-testid="helper">Hidden</FormHelperText>);
    const helper = screen.getByTestId('helper');
    expect(helper).toHaveClass('sr-only');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLParagraphElement | null };
    render(<FormHelperText ref={ref}>Helper</FormHelperText>);
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('P');
  });
});

describe('FormControl composition', () => {
  it('renders complete form control with label, input, and helper text', () => {
    render(
      <FormControl>
        <FormLabel htmlFor="email">Email</FormLabel>
        <input id="email" data-testid="input" />
        <FormHelperText>We'll never share your email</FormHelperText>
      </FormControl>
    );
    expect(screen.getByText('Email')).toBeInTheDocument();
    expect(screen.getByTestId('input')).toBeInTheDocument();
    expect(screen.getByText("We'll never share your email")).toBeInTheDocument();
  });

  it('renders error state throughout', () => {
    render(
      <FormControl error>
        <FormLabel error>Email</FormLabel>
        <input id="email" data-testid="input" />
        <FormHelperText error>Invalid email</FormHelperText>
      </FormControl>
    );
    expect(screen.getByText('Email')).toHaveClass('text-nest-error');
    expect(screen.getByText('Invalid email')).toHaveClass('text-nest-error');
  });
});
