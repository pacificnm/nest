import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Accordion, AccordionItem, AccordionSummary, AccordionDetails } from './Accordion';

describe('Accordion', () => {
  it('renders accordion items', () => {
    render(
      <Accordion>
        <AccordionItem value="panel1" summary="Panel 1">Content 1</AccordionItem>
        <AccordionItem value="panel2" summary="Panel 2">Content 2</AccordionItem>
      </Accordion>
    );
    expect(screen.getByText('Panel 1')).toBeInTheDocument();
    expect(screen.getByText('Panel 2')).toBeInTheDocument();
  });

  it('expands panel when clicked (uncontrolled)', () => {
    render(
      <Accordion defaultExpanded="panel1">
        <AccordionItem value="panel1" summary="Panel 1" data-testid="item1">Content 1</AccordionItem>
        <AccordionItem value="panel2" summary="Panel 2" data-testid="item2">Content 2</AccordionItem>
      </Accordion>
    );
    const item1Summary = screen.getByText('Panel 1').closest('button');
    const item2Summary = screen.getByText('Panel 2').closest('button');
    expect(item1Summary).toHaveAttribute('aria-expanded', 'true');
    expect(item2Summary).toHaveAttribute('aria-expanded', 'false');
  });

  it('expands panel when clicked (controlled)', () => {
    const handleChange = vi.fn();
    render(
      <Accordion expanded="panel1" onChange={handleChange} exclusive>
        <AccordionItem value="panel1" summary="Panel 1" data-testid="item1">Content 1</AccordionItem>
        <AccordionItem value="panel2" summary="Panel 2" data-testid="item2">Content 2</AccordionItem>
      </Accordion>
    );
    const item2Summary = screen.getByText('Panel 2').closest('button');
    fireEvent.click(item2Summary!);
    expect(handleChange).toHaveBeenCalledWith('panel2');
  });

  it('collapses expanded panel when clicked (exclusive)', () => {
    const handleChange = vi.fn();
    render(
      <Accordion expanded="panel1" onChange={handleChange} exclusive>
        <AccordionItem value="panel1" summary="Panel 1">Content 1</AccordionItem>
        <AccordionItem value="panel2" summary="Panel 2">Content 2</AccordionItem>
      </Accordion>
    );
    const item1Summary = screen.getByText('Panel 1').closest('button');
    fireEvent.click(item1Summary!);
    expect(handleChange).toHaveBeenCalledWith('');
  });

  it('allows multiple expanded panels (non-exclusive)', () => {
    const handleChange = vi.fn();
    render(
      <Accordion expanded={['panel1']} onChange={handleChange} exclusive={false}>
        <AccordionItem value="panel1" summary="Panel 1">Content 1</AccordionItem>
        <AccordionItem value="panel2" summary="Panel 2">Content 2</AccordionItem>
      </Accordion>
    );
    const item2Summary = screen.getByText('Panel 2').closest('button');
    fireEvent.click(item2Summary!);
    expect(handleChange).toHaveBeenCalledWith(['panel1', 'panel2']);
  });

  it('applies disabled state', () => {
    render(
      <Accordion disabled>
        <AccordionItem value="panel1" summary="Panel 1" data-testid="item1">Content 1</AccordionItem>
      </Accordion>
    );
    const item1Summary = screen.getByText('Panel 1').closest('button');
    expect(item1Summary).toHaveClass('opacity-50', 'cursor-not-allowed');
    expect(item1Summary).toBeDisabled();
  });

  it('applies custom className', () => {
    render(
      <Accordion className="custom-class">
        <AccordionItem value="panel1" summary="Panel 1">Content 1</AccordionItem>
      </Accordion>
    );
    expect(screen.getByText('Panel 1').closest('button')?.parentElement?.parentElement).toHaveClass('custom-class');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Accordion ref={ref}>
        <AccordionItem value="panel1" summary="Panel 1">Content 1</AccordionItem>
      </Accordion>
    );
    expect(ref.current).toBeInTheDocument();
  });
});

describe('AccordionItem', () => {
  it('renders summary and details when expanded', () => {
    render(
      <Accordion defaultExpanded="panel1">
        <AccordionItem value="panel1" summary="Summary">Details content</AccordionItem>
      </Accordion>
    );
    expect(screen.getByText('Summary')).toBeInTheDocument();
    expect(screen.getByText('Details content')).toBeInTheDocument();
  });

  it('toggles on summary click', () => {
    render(
      <Accordion defaultExpanded="">
        <AccordionItem value="panel1" summary="Summary" data-testid="item">Details</AccordionItem>
      </Accordion>
    );
    const summary = screen.getByText('Summary').closest('button');
    expect(summary).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(summary!);
    expect(summary).toHaveAttribute('aria-expanded', 'true');
  });

  it('applies disabled state to summary', () => {
    render(
      <Accordion>
        <AccordionItem value="panel1" summary="Summary" disabled>Details</AccordionItem>
      </Accordion>
    );
    const summary = screen.getByText('Summary').closest('button');
    expect(summary).toBeDisabled();
  });
});

describe('AccordionSummary', () => {
  it('renders children', () => {
    render(<AccordionSummary>Summary Text</AccordionSummary>);
    expect(screen.getByText('Summary Text')).toBeInTheDocument();
  });

  it('shows chevron rotation when expanded', () => {
    const { container, rerender } = render(<AccordionSummary expanded={false}>Summary</AccordionSummary>);
    const chevron = container.querySelector('svg');
    expect(chevron).not.toHaveClass('rotate-180');
    rerender(<AccordionSummary expanded>Summary</AccordionSummary>);
    expect(container.querySelector('svg')).toHaveClass('rotate-180');
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<AccordionSummary onClick={handleClick}>Summary</AccordionSummary>);
    fireEvent.click(screen.getByText('Summary'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies disabled state', () => {
    render(<AccordionSummary disabled>Summary</AccordionSummary>);
    const button = screen.getByText('Summary').closest('button');
    expect(button).toHaveClass('opacity-50', 'cursor-not-allowed');
    expect(button).toBeDisabled();
  });
});

describe('AccordionDetails', () => {
  it('renders children when expanded', () => {
    render(<AccordionDetails expanded>Details Content</AccordionDetails>);
    expect(screen.getByText('Details Content')).toBeInTheDocument();
  });

  it('hides children when not expanded', () => {
    render(<AccordionDetails expanded={false}>Details Content</AccordionDetails>);
    expect(screen.queryByText('Details Content')).not.toBeInTheDocument();
  });

  it('applies transition classes', () => {
    const { container, rerender } = render(<AccordionDetails expanded={false}>Details</AccordionDetails>);
    expect(container.firstChild).toHaveClass('py-0', 'opacity-0');
    rerender(<AccordionDetails expanded>Details</AccordionDetails>);
    expect(container.firstChild).toHaveClass('py-3', 'opacity-100');
  });
});
