import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { List, ListItem, ListItemButton, ListItemText, ListItemIcon, ListItemAvatar } from './List';

describe('List', () => {
  it('renders list items', () => {
    render(
      <List>
        <ListItem>Item 1</ListItem>
        <ListItem>Item 2</ListItem>
      </List>
    );
    expect(screen.getByText('Item 1')).toBeInTheDocument();
    expect(screen.getByText('Item 2')).toBeInTheDocument();
  });

  it('renders as ul by default', () => {
    render(<List data-testid="list"><ListItem>Item</ListItem></List>);
    expect(screen.getByTestId('list').tagName).toBe('UL');
  });

  it('renders as custom component', () => {
    render(<List component="div" data-testid="list"><ListItem>Item</ListItem></List>);
    expect(screen.getByTestId('list').tagName).toBe('DIV');
  });

  it('applies role="list"', () => {
    render(<List data-testid="list"><ListItem>Item</ListItem></List>);
    expect(screen.getByTestId('list')).toHaveAttribute('role', 'list');
  });

  it('applies dense padding', () => {
    render(<List dense data-testid="list"><ListItem>Item</ListItem></List>);
    expect(screen.getByTestId('list')).toHaveClass('py-1');
  });

  it('applies normal padding when not dense', () => {
    render(<List data-testid="list"><ListItem>Item</ListItem></List>);
    expect(screen.getByTestId('list')).toHaveClass('py-2');
  });

  it('applies base styles', () => {
    render(<List data-testid="list"><ListItem>Item</ListItem></List>);
    expect(screen.getByTestId('list')).toHaveClass('flex', 'flex-col');
  });

  it('applies custom className', () => {
    render(<List className="custom-list" data-testid="list"><ListItem>Item</ListItem></List>);
    expect(screen.getByTestId('list')).toHaveClass('custom-list');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLUListElement | null };
    render(<List ref={ref} data-testid="list"><ListItem>Item</ListItem></List>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('ListItem', () => {
  it('renders as li by default', () => {
    render(<ListItem data-testid="item">Item</ListItem>);
    expect(screen.getByTestId('item').tagName).toBe('LI');
  });

  it('renders as custom component', () => {
    render(<ListItem component="div" data-testid="item">Item</ListItem>);
    expect(screen.getByTestId('item').tagName).toBe('DIV');
  });

  it('applies role="listitem"', () => {
    render(<ListItem data-testid="item">Item</ListItem>);
    expect(screen.getByTestId('item')).toHaveAttribute('role', 'listitem');
  });

  it('applies base styles', () => {
    render(<ListItem data-testid="item">Item</ListItem>);
    expect(screen.getByTestId('item')).toHaveClass('flex', 'items-center', 'gap-3', 'px-3');
  });

  it('applies custom className', () => {
    render(<ListItem className="custom-item" data-testid="item">Item</ListItem>);
    expect(screen.getByTestId('item')).toHaveClass('custom-item');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLLIElement | null };
    render(<ListItem ref={ref} data-testid="item">Item</ListItem>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('ListItemButton', () => {
  it('renders as button by default', () => {
    render(<ListItemButton data-testid="button">Button</ListItemButton>);
    expect(screen.getByTestId('button').tagName).toBe('BUTTON');
  });

  it('renders as custom component', () => {
    render(<ListItemButton component="a" href="#" data-testid="button">Link</ListItemButton>);
    expect(screen.getByTestId('button').tagName).toBe('A');
  });

  it('applies selected styles', () => {
    render(<ListItemButton selected data-testid="button">Button</ListItemButton>);
    expect(screen.getByTestId('button')).toHaveClass('bg-nest-primary/10', 'text-nest-primary');
  });

  it('applies hover styles when not selected', () => {
    render(<ListItemButton data-testid="button">Button</ListItemButton>);
    expect(screen.getByTestId('button')).toHaveClass('hover:bg-nest-surface');
  });

  it('applies disabled styles', () => {
    render(<ListItemButton disabled data-testid="button">Button</ListItemButton>);
    expect(screen.getByTestId('button')).toHaveClass('opacity-50', 'cursor-not-allowed', 'pointer-events-none');
  });

  it('calls onClick handler', () => {
    const handleClick = vi.fn();
    render(<ListItemButton onClick={handleClick} data-testid="button">Button</ListItemButton>);
    fireEvent.click(screen.getByTestId('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies base styles', () => {
    render(<ListItemButton data-testid="button">Button</ListItemButton>);
    expect(screen.getByTestId('button')).toHaveClass(
      'flex',
      'items-center',
      'gap-3',
      'px-3',
      'w-full',
      'text-left',
      'rounded-nest-sm',
      'transition-colors',
      'duration-150',
      'focus:outline-none',
      'focus:ring-2',
      'focus:ring-nest-primary/50'
    );
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLButtonElement | null };
    render(<ListItemButton ref={ref} data-testid="button">Button</ListItemButton>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('ListItemText', () => {
  it('renders primary text', () => {
    render(<ListItemText primary="Primary" />);
    expect(screen.getByText('Primary')).toBeInTheDocument();
  });

  it('renders secondary text', () => {
    render(<ListItemText primary="Primary" secondary="Secondary" />);
    expect(screen.getByText('Secondary')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<ListItemText primary="Text" data-testid="text" />);
    expect(screen.getByTestId('text').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<ListItemText component="span" primary="Text" data-testid="text" />);
    expect(screen.getByTestId('text').tagName).toBe('SPAN');
  });

  it('applies primary text styles', () => {
    render(<ListItemText primary="Primary" />);
    expect(screen.getByText('Primary')).toHaveClass('text-nest-foreground', 'truncate');
  });

  it('applies secondary text styles', () => {
    render(<ListItemText primary="Primary" secondary="Secondary" />);
    expect(screen.getByText('Secondary')).toHaveClass('text-sm', 'text-nest-muted', 'truncate');
  });

  it('applies base styles', () => {
    render(<ListItemText primary="Text" data-testid="text" />);
    expect(screen.getByTestId('text')).toHaveClass('flex', 'flex-col', 'flex-1', 'min-w-0');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<ListItemText ref={ref} primary="Text" />);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('ListItemIcon', () => {
  it('renders children', () => {
    render(<ListItemIcon><span data-testid="icon">★</span></ListItemIcon>);
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<ListItemIcon data-testid="icon"><span>Icon</span></ListItemIcon>);
    expect(screen.getByTestId('icon').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<ListItemIcon component="span" data-testid="icon"><span>Icon</span></ListItemIcon>);
    expect(screen.getByTestId('icon').tagName).toBe('SPAN');
  });

  it('applies base styles', () => {
    render(<ListItemIcon data-testid="icon"><span>Icon</span></ListItemIcon>);
    expect(screen.getByTestId('icon')).toHaveClass('shrink-0', 'text-nest-muted');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<ListItemIcon ref={ref} data-testid="icon"><span>Icon</span></ListItemIcon>);
    expect(ref.current).toBeInTheDocument();
  });
});

describe('ListItemAvatar', () => {
  it('renders children', () => {
    render(<ListItemAvatar><span data-testid="avatar">👤</span></ListItemAvatar>);
    expect(screen.getByTestId('avatar')).toBeInTheDocument();
  });

  it('renders as div by default', () => {
    render(<ListItemAvatar data-testid="avatar"><span>Avatar</span></ListItemAvatar>);
    expect(screen.getByTestId('avatar').tagName).toBe('DIV');
  });

  it('renders as custom component', () => {
    render(<ListItemAvatar component="span" data-testid="avatar"><span>Avatar</span></ListItemAvatar>);
    expect(screen.getByTestId('avatar').tagName).toBe('SPAN');
  });

  it('applies base styles', () => {
    render(<ListItemAvatar data-testid="avatar"><span>Avatar</span></ListItemAvatar>);
    expect(screen.getByTestId('avatar')).toHaveClass('shrink-0');
  });

  it('forwards ref', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(<ListItemAvatar ref={ref} data-testid="avatar"><span>Avatar</span></ListItemAvatar>);
    expect(ref.current).toBeInTheDocument();
  });
});
