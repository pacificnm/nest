import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Breadcrumbs } from './Breadcrumbs';

const defaultItems = [
  { label: 'Home', href: '/' },
  { label: 'Products', href: '/products' },
  { label: 'Electronics', current: true },
];

describe('Breadcrumbs', () => {
  it('renders breadcrumb items', () => {
    render(<Breadcrumbs items={defaultItems} />);
    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Products')).toBeInTheDocument();
    expect(screen.getByText('Electronics')).toBeInTheDocument();
  });

  it('renders as nav by default', () => {
    render(<Breadcrumbs items={defaultItems} data-testid="breadcrumbs" />);
    expect(screen.getByTestId('breadcrumbs').tagName).toBe('NAV');
  });

  it('renders as custom component', () => {
    render(<Breadcrumbs items={defaultItems} component="div" data-testid="breadcrumbs" />);
    expect(screen.getByTestId('breadcrumbs').tagName).toBe('DIV');
  });

  it('applies default aria-label', () => {
    render(<Breadcrumbs items={defaultItems} data-testid="breadcrumbs" />);
    expect(screen.getByTestId('breadcrumbs')).toHaveAttribute('aria-label', 'breadcrumb');
  });

  it('applies custom aria-label', () => {
    render(<Breadcrumbs items={defaultItems} ariaLabel="custom-breadcrumb" data-testid="breadcrumbs" />);
    expect(screen.getByTestId('breadcrumbs')).toHaveAttribute('aria-label', 'custom-breadcrumb');
  });

  it('renders links for items with href', () => {
    render(<Breadcrumbs items={defaultItems} />);
    const homeLink = screen.getByText('Home');
    expect(homeLink.tagName).toBe('A');
    expect(homeLink).toHaveAttribute('href', '/');

    const productsLink = screen.getByText('Products');
    expect(productsLink.tagName).toBe('A');
    expect(productsLink).toHaveAttribute('href', '/products');
  });

  it('renders current item as span (not clickable)', () => {
    render(<Breadcrumbs items={defaultItems} />);
    const electronics = screen.getByText('Electronics');
    expect(electronics.tagName).toBe('SPAN');
    expect(electronics).not.toHaveAttribute('href');
  });

  it('applies aria-current="page" to current item', () => {
    render(<Breadcrumbs items={defaultItems} />);
    const electronics = screen.getByText('Electronics');
    expect(electronics).toHaveAttribute('aria-current', 'page');
  });

  it('renders separator between items', () => {
    render(<Breadcrumbs items={defaultItems} />);
    // Should have 2 separators for 3 items (SVG icons)
    const separators = screen.getAllByTestId('separator-icon');
    expect(separators).toHaveLength(2);
  });

  it('renders custom separator', () => {
    render(<Breadcrumbs items={defaultItems} separator="/" data-testid="breadcrumbs" />);
    // Check that slashes appear between items
    const text = screen.getByTestId('breadcrumbs').textContent;
    expect(text).toContain('/');
  });

  it('applies base styles', () => {
    render(<Breadcrumbs items={defaultItems} data-testid="breadcrumbs" />);
    expect(screen.getByTestId('breadcrumbs')).toHaveClass('flex', 'items-center', 'gap-1', 'text-sm');
  });

  it('applies custom className', () => {
    render(<Breadcrumbs items={defaultItems} className="custom-breadcrumbs" data-testid="breadcrumbs" />);
    expect(screen.getByTestId('breadcrumbs')).toHaveClass('custom-breadcrumbs');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLElement | null };
    render(<Breadcrumbs items={defaultItems} ref={ref} />);
    expect(ref.current).toBeInTheDocument();
  });

  it('renders items without href as non-clickable', () => {
    const itemsWithoutHref = [
      { label: 'Home' },
      { label: 'Current', current: true },
    ];
    render(<Breadcrumbs items={itemsWithoutHref} />);
    const home = screen.getByText('Home');
    const current = screen.getByText('Current');
    expect(home.tagName).toBe('SPAN');
    expect(current.tagName).toBe('SPAN');
    expect(home).not.toHaveAttribute('href');
  });

  it('applies link styles to clickable items', () => {
    render(<Breadcrumbs items={defaultItems} />);
    const homeLink = screen.getByText('Home');
    expect(homeLink).toHaveClass('text-nest-primary', 'hover:text-nest-primary/80', 'hover:underline');
  });

  it('applies current item styles', () => {
    render(<Breadcrumbs items={defaultItems} />);
    const electronics = screen.getByText('Electronics');
    expect(electronics).toHaveClass('font-medium', 'text-nest-foreground');
  });

  it('applies non-current item styles', () => {
    const items = [
      { label: 'Home', href: '/' },
      { label: 'Section' },
    ];
    render(<Breadcrumbs items={items} />);
    const section = screen.getByText('Section');
    expect(section).toHaveClass('text-nest-muted');
  });

  it('handles maxItems collapsing', () => {
    const manyItems = [
      { label: 'Home', href: '/' },
      { label: 'Level 1', href: '/1' },
      { label: 'Level 2', href: '/2' },
      { label: 'Level 3', href: '/3' },
      { label: 'Current', current: true },
    ];
    render(<Breadcrumbs items={manyItems} maxItems={3} />);
    // Should show +3 collapsed indicator and last 2 items
    expect(screen.getByText('+3')).toBeInTheDocument();
    expect(screen.getByText('Level 3')).toBeInTheDocument();
    expect(screen.getByText('Current')).toBeInTheDocument();
  });

  it('does not collapse when maxItems is 0', () => {
    render(<Breadcrumbs items={defaultItems} maxItems={0} />);
    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Products')).toBeInTheDocument();
    expect(screen.getByText('Electronics')).toBeInTheDocument();
    expect(screen.queryByText('+2')).not.toBeInTheDocument();
  });

  it('renders single item correctly', () => {
    const singleItem = [{ label: 'Current', current: true }];
    render(<Breadcrumbs items={singleItem} />);
    expect(screen.getByText('Current')).toBeInTheDocument();
    expect(screen.getByText('Current')).toHaveAttribute('aria-current', 'page');
  });

  it('renders with React node labels', () => {
    const itemsWithIcons = [
      { label: <span data-testid="home-icon">🏠</span>, href: '/' },
      { label: <span data-testid="current-icon">📄</span>, current: true },
    ];
    render(<Breadcrumbs items={itemsWithIcons} />);
    expect(screen.getByTestId('home-icon')).toBeInTheDocument();
    expect(screen.getByTestId('current-icon')).toBeInTheDocument();
  });

  it('passes through additional props', () => {
    render(<Breadcrumbs items={defaultItems} id="test-id" data-testid="breadcrumbs" />);
    expect(screen.getByTestId('breadcrumbs')).toHaveAttribute('id', 'test-id');
  });

  it('renders ordered list structure', () => {
    render(<Breadcrumbs items={defaultItems} />);
    const ol = screen.getByRole('list');
    expect(ol.tagName).toBe('OL');
    const listItems = ol.querySelectorAll('li');
    expect(listItems).toHaveLength(3);
  });
});
