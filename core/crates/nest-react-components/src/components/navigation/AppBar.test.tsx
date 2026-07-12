import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AppBar, Toolbar } from './AppBar';

describe('AppBar', () => {
  it('renders a banner header with children', () => {
    render(<AppBar>Title bar</AppBar>);
    expect(screen.getByRole('banner')).toHaveTextContent('Title bar');
  });

  it('applies position styles', () => {
    render(<AppBar position="sticky">x</AppBar>);
    expect(screen.getByRole('banner')).toHaveClass('sticky');
  });

  it('applies color styles', () => {
    render(<AppBar color="primary">x</AppBar>);
    expect(screen.getByRole('banner')).toHaveClass('bg-nest-primary');
  });

  it('adds elevation by default and omits it when disabled', () => {
    const { rerender } = render(<AppBar>x</AppBar>);
    expect(screen.getByRole('banner')).toHaveClass('border-b');

    rerender(<AppBar elevation={false}>x</AppBar>);
    expect(screen.getByRole('banner')).not.toHaveClass('border-b');
  });
});

describe('Toolbar', () => {
  it('renders children', () => {
    render(<Toolbar>Actions</Toolbar>);
    expect(screen.getByText('Actions')).toBeInTheDocument();
  });

  it('applies variant height', () => {
    const { rerender } = render(<Toolbar>x</Toolbar>);
    expect(screen.getByText('x')).toHaveClass('h-12');

    rerender(<Toolbar variant="dense">x</Toolbar>);
    expect(screen.getByText('x')).toHaveClass('h-8');
  });
});
