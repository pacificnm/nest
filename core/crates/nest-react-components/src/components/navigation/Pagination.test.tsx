import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Pagination } from './Pagination';

describe('Pagination', () => {
  it('renders pagination with page numbers', () => {
    render(<Pagination count={5} defaultPage={1} />);
    // With count=5, all pages should be visible (no ellipsis needed)
    expect(screen.getByLabelText('Page 1')).toBeInTheDocument();
    expect(screen.getByLabelText('Page 2')).toBeInTheDocument();
    expect(screen.getByLabelText('Page 5')).toBeInTheDocument();
  });

  it('highlights current page', () => {
    render(<Pagination count={5} defaultPage={3} />);
    const page3 = screen.getByLabelText('Page 3');
    expect(page3).toHaveAttribute('aria-current', 'page');
  });

  it('calls onChange when page is clicked', () => {
    const handleChange = vi.fn();
    render(<Pagination count={10} onChange={handleChange} defaultPage={1} />);
    // Page 2 should be visible as a sibling of page 1
    fireEvent.click(screen.getByLabelText('Page 2'));
    expect(handleChange).toHaveBeenCalledWith(expect.anything(), 2);
  });

  it('navigates to previous page', () => {
    const handleChange = vi.fn();
    render(<Pagination count={10} page={5} onChange={handleChange} />);
    fireEvent.click(screen.getByLabelText('Go to previous page'));
    expect(handleChange).toHaveBeenCalledWith(expect.anything(), 4);
  });

  it('navigates to next page', () => {
    const handleChange = vi.fn();
    render(<Pagination count={10} page={5} onChange={handleChange} />);
    fireEvent.click(screen.getByLabelText('Go to next page'));
    expect(handleChange).toHaveBeenCalledWith(expect.anything(), 6);
  });

  it('navigates to first page', () => {
    const handleChange = vi.fn();
    render(<Pagination count={10} page={5} onChange={handleChange} />);
    fireEvent.click(screen.getByLabelText('Go to first page'));
    expect(handleChange).toHaveBeenCalledWith(expect.anything(), 1);
  });

  it('navigates to last page', () => {
    const handleChange = vi.fn();
    render(<Pagination count={10} page={5} onChange={handleChange} />);
    fireEvent.click(screen.getByLabelText('Go to last page'));
    expect(handleChange).toHaveBeenCalledWith(expect.anything(), 10);
  });

  it('disables previous button on first page', () => {
    render(<Pagination count={10} defaultPage={1} />);
    const prevButton = screen.getByLabelText('Go to previous page');
    expect(prevButton).toBeDisabled();
  });

  it('disables next button on last page', () => {
    render(<Pagination count={10} defaultPage={10} />);
    const nextButton = screen.getByLabelText('Go to next page');
    expect(nextButton).toBeDisabled();
  });

  it('applies disabled state', () => {
    render(<Pagination count={10} disabled data-testid="pagination" />);
    const pagination = screen.getByTestId('pagination');
    expect(pagination).toHaveClass('opacity-50', 'pointer-events-none');
  });

  it('applies small size', () => {
    render(<Pagination count={5} size="small" data-testid="pagination" />);
    const pagination = screen.getByTestId('pagination');
    expect(pagination).toHaveClass('text-xs');
  });

  it('applies medium size (default)', () => {
    render(<Pagination count={5} data-testid="pagination" />);
    const pagination = screen.getByTestId('pagination');
    expect(pagination).toHaveClass('text-sm');
  });

  it('applies large size', () => {
    render(<Pagination count={5} size="large" data-testid="pagination" />);
    const pagination = screen.getByTestId('pagination');
    expect(pagination).toHaveClass('text-base');
  });

  it('applies color styles to active page', () => {
    render(<Pagination count={5} defaultPage={1} color="error" />);
    const page1 = screen.getByLabelText('Page 1');
    expect(page1.className).toContain('bg-nest-error');
    expect(page1.className).toContain('text-white');
  });

  it('hides first/last buttons when hideFirstLast=true', () => {
    render(<Pagination count={10} hideFirstLast defaultPage={5} />);
    expect(screen.queryByLabelText('Go to first page')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Go to last page')).not.toBeInTheDocument();
  });

  it('hides prev/next buttons when hidePrevNext=true', () => {
    render(<Pagination count={10} hidePrevNext defaultPage={5} />);
    expect(screen.queryByLabelText('Go to previous page')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Go to next page')).not.toBeInTheDocument();
  });

  it('shows ellipsis for skipped pages', () => {
    render(<Pagination count={20} defaultPage={10} siblingCount={1} boundaryCount={1} />);
    // Should show ellipsis between boundary pages and sibling pages
    const ellipsis = screen.getAllByText('…');
    expect(ellipsis.length).toBeGreaterThanOrEqual(1);
  });

  it('has navigation role', () => {
    render(<Pagination count={5} defaultPage={1} data-testid="pagination" />);
    expect(screen.getByTestId('pagination')).toHaveAttribute('role', 'navigation');
  });

  it('has aria-label', () => {
    render(<Pagination count={5} defaultPage={1} data-testid="pagination" />);
    expect(screen.getByTestId('pagination')).toHaveAttribute('aria-label', 'Pagination');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<Pagination count={5} ref={ref} defaultPage={1} />);
    expect(ref.current).toBeInTheDocument();
  });
});
