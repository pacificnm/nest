import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Menu, MenuItem, MenuDivider, MenuBar, MenuBarItem } from './Menu';

describe('Menu', () => {
  it('renders nothing when closed', () => {
    render(
      <Menu open={false} onClose={vi.fn()}>
        <MenuItem>Save</MenuItem>
      </Menu>
    );
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
  });

  it('renders a menu with items when open', () => {
    render(
      <Menu open onClose={vi.fn()}>
        <MenuItem>Save</MenuItem>
      </Menu>
    );
    expect(screen.getByRole('menu')).toBeInTheDocument();
    expect(screen.getByRole('menuitem', { name: 'Save' })).toBeInTheDocument();
  });

  it('fires an item onClick', () => {
    const onClick = vi.fn();
    render(
      <Menu open onClose={vi.fn()}>
        <MenuItem onClick={onClick}>Save</MenuItem>
      </Menu>
    );
    fireEvent.click(screen.getByRole('menuitem', { name: 'Save' }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('styles a danger item and renders a divider', () => {
    render(
      <Menu open onClose={vi.fn()}>
        <MenuItem>Save</MenuItem>
        <MenuDivider />
        <MenuItem danger>Delete</MenuItem>
      </Menu>
    );
    expect(screen.getByRole('menuitem', { name: 'Delete' })).toHaveClass('text-nest-error');
    expect(screen.getByRole('separator')).toBeInTheDocument();
  });

  it('closes on outside click', () => {
    const onClose = vi.fn();
    render(
      <Menu open onClose={onClose}>
        <MenuItem>Save</MenuItem>
      </Menu>
    );
    fireEvent.mouseDown(document.body);
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});

describe('MenuBar', () => {
  it('renders a menubar and opens a dropdown on click', () => {
    render(
      <MenuBar>
        <MenuBarItem id="file" label="File">
          <MenuItem>Open</MenuItem>
        </MenuBarItem>
      </MenuBar>
    );
    expect(screen.getByRole('menubar')).toBeInTheDocument();
    expect(screen.queryByText('Open')).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('menuitem', { name: 'File' }));
    expect(screen.getByText('Open')).toBeInTheDocument();
  });
});
