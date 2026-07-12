import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Avatar } from './Avatar';

describe('Avatar', () => {
  it('renders children as fallback when no src', () => {
    render(<Avatar>JD</Avatar>);
    expect(screen.getByText('JD')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Avatar data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<Avatar component="span" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar').tagName).toBe('SPAN');
  });

  it('renders image when src provided', () => {
    render(<Avatar src="/test.jpg" alt="Test" data-testid="avatar" />);
    const img = screen.getByTestId('avatar').querySelector('img');
    expect(img).toHaveAttribute('src', '/test.jpg');
    expect(img).toHaveAttribute('alt', 'Test');
  });

  it('applies object-cover to image', () => {
    render(<Avatar src="/test.jpg" data-testid="avatar" />);
    const img = screen.getByTestId('avatar').querySelector('img');
    expect(img).toHaveClass('object-cover');
  });

  it('applies default size (medium)', () => {
    render(<Avatar data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('h-10', 'w-10');
  });

  it('applies small size', () => {
    render(<Avatar size="small" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('h-8', 'w-8');
  });

  it('applies medium size', () => {
    render(<Avatar size="medium" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('h-10', 'w-10');
  });

  it('applies large size', () => {
    render(<Avatar size="large" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('h-12', 'w-12');
  });

  it('applies default variant (circular)', () => {
    render(<Avatar data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('rounded-full');
  });

  it('applies circular variant', () => {
    render(<Avatar variant="circular" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('rounded-full');
  });

  it('applies rounded variant', () => {
    render(<Avatar variant="rounded" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('rounded-nest-md');
  });

  it('applies square variant', () => {
    render(<Avatar variant="square" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('rounded-none');
  });

  it('applies base styles', () => {
    render(<Avatar data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass(
      'flex',
      'shrink-0',
      'items-center',
      'justify-center',
      'overflow-hidden',
      'bg-nest-muted',
      'text-nest-foreground'
    );
  });

  it('shows fallback when image fails to load', () => {
    render(<Avatar src="/invalid.jpg" data-testid="avatar">Fallback</Avatar>);
    // Initially renders img
    expect(screen.getByTestId('avatar').querySelector('img')).toBeInTheDocument();
    // Simulate error
    fireEvent.error(screen.getByTestId('avatar').querySelector('img')!);
    // Should show fallback
    expect(screen.getByText('Fallback')).toBeInTheDocument();
  });

  it('calls imgProps onError when image fails', () => {
    const handleError = vi.fn();
    render(
      <Avatar src="/invalid.jpg" imgProps={{ onError: handleError }} data-testid="avatar">
        Fallback
      </Avatar>
    );
    fireEvent.error(screen.getByTestId('avatar').querySelector('img')!);
    expect(handleError).toHaveBeenCalledTimes(1);
  });

  it('applies custom className', () => {
    render(<Avatar className="custom-avatar" data-testid="avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('custom-avatar');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Avatar ref={ref} data-testid="avatar">Content</Avatar>);
    expect(ref.current).toBeInTheDocument();
  });

  it('passes through additional props', () => {
    render(<Avatar data-testid="avatar" id="test-id" aria-label="User avatar">Content</Avatar>);
    expect(screen.getByTestId('avatar')).toHaveAttribute('id', 'test-id');
    expect(screen.getByTestId('avatar')).toHaveAttribute('aria-label', 'User avatar');
  });

  it('passes imgProps to image', () => {
    render(
      <Avatar
        src="/test.jpg"
        imgProps={{ loading: 'lazy', decoding: 'async' }}
        data-testid="avatar"
      />
    );
    const img = screen.getByTestId('avatar').querySelector('img');
    expect(img).toHaveAttribute('loading', 'lazy');
    expect(img).toHaveAttribute('decoding', 'async');
  });

  it('renders icon as children', () => {
    render(
      <Avatar data-testid="avatar">
        <span data-testid="icon">★</span>
      </Avatar>
    );
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('renders initials as children', () => {
    render(<Avatar data-testid="avatar">JD</Avatar>);
    expect(screen.getByText('JD')).toBeInTheDocument();
    expect(screen.getByText('JD')).toHaveClass('font-medium');
  });
});
