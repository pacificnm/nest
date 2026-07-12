import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Chip } from './Chip';

describe('Chip', () => {
  it('renders label', () => {
    render(<Chip label="Test Label" />);
    expect(screen.getByText('Test Label')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<Chip label="Content" data-testid="chip" />);
    expect(screen.getByTestId('chip').tagName).toBe('DIV');
  });

  it('renders as button when clickable', () => {
    render(<Chip label="Clickable" clickable data-testid="chip" />);
    expect(screen.getByTestId('chip').tagName).toBe('BUTTON');
  });

  it('renders as button when onClick provided', () => {
    render(<Chip label="Clickable" onClick={() => {}} data-testid="chip" />);
    expect(screen.getByTestId('chip').tagName).toBe('BUTTON');
  });

  it('renders as custom component', () => {
    render(<Chip label="Content" component="span" data-testid="chip" />);
    expect(screen.getByTestId('chip').tagName).toBe('SPAN');
  });

  it('applies default variant (filled) and color (default)', () => {
    render(<Chip label="Default" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('bg-nest-surface', 'text-nest-foreground');
  });

  it('applies filled variant with primary color', () => {
    render(<Chip label="Primary" variant="filled" color="primary" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('bg-nest-primary', 'text-white');
  });

  it('applies filled variant with error color', () => {
    render(<Chip label="Error" variant="filled" color="error" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('bg-nest-error', 'text-white');
  });

  it('applies outlined variant with primary color', () => {
    render(<Chip label="Primary" variant="outlined" color="primary" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('border', 'border-nest-primary', 'text-nest-primary');
  });

  it('applies outlined variant with default color', () => {
    render(<Chip label="Default" variant="outlined" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('border', 'border-nest-border', 'text-nest-foreground');
  });

  it('applies default size (medium)', () => {
    render(<Chip label="Medium" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('h-8', 'text-sm', 'px-3');
  });

  it('applies small size', () => {
    render(<Chip label="Small" size="small" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('h-6', 'text-xs', 'px-2');
  });

  it('renders icon before label', () => {
    render(<Chip label="With Icon" icon={<span data-testid="icon">★</span>} data-testid="chip" />);
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('renders delete icon when onDelete provided', () => {
    render(<Chip label="Deletable" onDelete={vi.fn()} data-testid="chip" />);
    const deleteButton = screen.getByRole('button', { name: 'Delete' });
    expect(deleteButton).toBeInTheDocument();
  });

  it('calls onDelete when delete icon is clicked', () => {
    const handleDelete = vi.fn();
    render(<Chip label="Deletable" onDelete={handleDelete} />);
    const deleteButton = screen.getByRole('button', { name: 'Delete' });
    fireEvent.click(deleteButton);
    expect(handleDelete).toHaveBeenCalledTimes(1);
  });

  it('does not call onClick when delete icon is clicked', () => {
    const handleClick = vi.fn();
    const handleDelete = vi.fn();
    render(
      <Chip label="Deletable" onClick={handleClick} onDelete={handleDelete} />
    );
    const deleteButton = screen.getByRole('button', { name: 'Delete' });
    fireEvent.click(deleteButton);
    expect(handleClick).not.toHaveBeenCalled();
    expect(handleDelete).toHaveBeenCalledTimes(1);
  });

  it('applies clickable styles when clickable=true', () => {
    render(<Chip label="Clickable" clickable data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('cursor-pointer', 'hover:opacity-80');
  });

  it('applies clickable styles when onClick provided', () => {
    render(<Chip label="Clickable" onClick={() => {}} data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('cursor-pointer', 'hover:opacity-80');
  });

  it('calls onClick when chip is clicked', () => {
    const handleClick = vi.fn();
    render(<Chip label="Clickable" onClick={handleClick} clickable />);
    fireEvent.click(screen.getByText('Clickable'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies disabled styles when disabled=true', () => {
    render(<Chip label="Disabled" disabled data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('opacity-50', 'cursor-not-allowed', 'pointer-events-none');
  });

  it('does not call onClick when disabled', () => {
    const handleClick = vi.fn();
    render(<Chip label="Disabled" disabled onClick={handleClick} />);
    fireEvent.click(screen.getByText('Disabled'));
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('does not call onDelete when disabled', () => {
    const handleDelete = vi.fn();
    render(<Chip label="Disabled" disabled onDelete={handleDelete} />);
    const deleteButton = screen.getByRole('button', { name: 'Delete' });
    fireEvent.click(deleteButton);
    expect(handleDelete).not.toHaveBeenCalled();
  });

  it('applies custom className', () => {
    render(<Chip label="Custom" className="custom-class" data-testid="chip" />);
    expect(screen.getByTestId('chip')).toHaveClass('custom-class');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Chip ref={ref} label="Ref test" />);
    expect(ref.current).toBeInTheDocument();
  });

  it('uses custom deleteLabel', () => {
    render(<Chip label="Remove me" onDelete={vi.fn()} deleteLabel="Remove item" />);
    expect(screen.getByRole('button', { name: 'Remove item' })).toBeInTheDocument();
  });

  it('truncates long label', () => {
    render(<Chip label="This is a very long label that should truncate" data-testid="chip" />);
    expect(screen.getByText('This is a very long label that should truncate')).toHaveClass('truncate');
  });

  it('renders all colors correctly (filled)', () => {
    render(<Chip label="default" color="default" data-testid="default" />);
    expect(screen.getByTestId('default')).toHaveClass('bg-nest-surface');

    render(<Chip label="primary" color="primary" data-testid="primary" />);
    expect(screen.getByTestId('primary')).toHaveClass('bg-nest-primary');

    render(<Chip label="error" color="error" data-testid="error" />);
    expect(screen.getByTestId('error')).toHaveClass('bg-nest-error');
  });

  it('renders all colors correctly (outlined)', () => {
    render(<Chip label="default" variant="outlined" color="default" data-testid="default" />);
    expect(screen.getByTestId('default')).toHaveClass('border-nest-border');

    render(<Chip label="primary" variant="outlined" color="primary" data-testid="primary" />);
    expect(screen.getByTestId('primary')).toHaveClass('border-nest-primary');

    render(<Chip label="error" variant="outlined" color="error" data-testid="error" />);
    expect(screen.getByTestId('error')).toHaveClass('border-nest-error');
  });
});
