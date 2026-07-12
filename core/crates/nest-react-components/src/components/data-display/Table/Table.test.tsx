import { render, screen } from '@testing-library/react';
import { Table, TableHead, TableBody, TableFooter, TableRow, TableCell } from './Table';
import { describe, it, expect } from 'vitest';

describe('Table', () => {
  it('renders a basic table', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell>Cell 1</TableCell>
            <TableCell>Cell 2</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    expect(screen.getByText('Cell 1')).toBeInTheDocument();
    expect(screen.getByText('Cell 2')).toBeInTheDocument();
  });

  it('applies fullWidth by default', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell>Content</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const tableContainer = screen.getByRole('table').parentElement;
    expect(tableContainer).toHaveClass('w-full');
  });

  it('does not apply fullWidth when false', () => {
    render(
      <Table fullWidth={false}>
        <TableBody>
          <TableRow>
            <TableCell>Content</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const tableContainer = screen.getByRole('table').parentElement;
    expect(tableContainer).not.toHaveClass('w-full');
  });

  it('applies sticky header styles', () => {
    render(
      <Table stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell component="th">Header</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          <TableRow>
            <TableCell>Content</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    // stickyHeader applies sticky positioning to the thead via the table class.
    const table = screen.getByRole('table');
    expect(table).toHaveClass('[&_thead]:sticky');
    expect(table.querySelector('thead')).toBeInTheDocument();
  });

  it('renders TableHead', () => {
    render(
      <Table>
        <TableHead>
          <TableRow>
            <TableCell component="th">Header</TableCell>
          </TableRow>
        </TableHead>
      </Table>
    );
    expect(screen.getByText('Header')).toBeInTheDocument();
  });

  it('renders TableBody', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell>Body Content</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    expect(screen.getByText('Body Content')).toBeInTheDocument();
  });

  it('renders TableFooter', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell>Body</TableCell>
          </TableRow>
        </TableBody>
        <TableFooter>
          <TableRow>
            <TableCell component="th">Footer</TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    );
    expect(screen.getByText('Footer')).toBeInTheDocument();
  });

  it('renders TableRow with hover effect', () => {
    render(
      <Table>
        <TableBody>
          <TableRow hover>
            <TableCell>Hoverable</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const row = screen.getByText('Hoverable').closest('tr');
    expect(row).toHaveClass('hover:bg-nest-surface');
  });

  it('renders TableRow without border when border=false', () => {
    render(
      <Table>
        <TableBody>
          <TableRow border={false}>
            <TableCell>No Border</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const row = screen.getByText('No Border').closest('tr');
    expect(row).not.toHaveClass('border-b');
  });

  it('renders TableCell with numeric alignment', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell numeric>123</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const cell = screen.getByText('123');
    expect(cell).toHaveClass('text-right');
  });

  it('renders TableCell with center alignment', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell center>Center</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const cell = screen.getByText('Center');
    expect(cell).toHaveClass('text-center');
  });

  it('renders TableCell with right alignment', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell right>Right</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const cell = screen.getByText('Right');
    expect(cell).toHaveClass('text-right');
  });

  it('renders TableCell as th component', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell component="th">Header Cell</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const cell = screen.getByText('Header Cell');
    expect(cell.tagName).toBe('TH');
    expect(cell).toHaveClass('font-medium');
  });

  it('renders TableCell with colSpan', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell colSpan={2}>Span 2</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const cell = screen.getByText('Span 2');
    expect(cell).toHaveAttribute('colspan', '2');
  });

  it('renders TableCell with rowSpan', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell rowSpan={2}>Span Rows</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );
    const cell = screen.getByText('Span Rows');
    expect(cell).toHaveAttribute('rowspan', '2');
  });
});
