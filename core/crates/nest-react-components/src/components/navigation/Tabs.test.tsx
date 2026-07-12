import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Tabs, Tab, TabPanel } from './Tabs';

describe('Tabs', () => {
  it('renders tabs', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" label="Tab One" data-testid="tab-one" />
        <Tab value="two" label="Tab Two" data-testid="tab-two" />
      </Tabs>
    );
    expect(screen.getByText('Tab One')).toBeInTheDocument();
    expect(screen.getByText('Tab Two')).toBeInTheDocument();
  });

  it('selects tab when clicked (uncontrolled)', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" label="One" data-testid="tab-one" />
        <Tab value="two" label="Two" data-testid="tab-two" />
      </Tabs>
    );
    const tabOne = screen.getByTestId('tab-one');
    const tabTwo = screen.getByTestId('tab-two');
    expect(tabOne).toHaveAttribute('aria-selected', 'true');
    expect(tabTwo).toHaveAttribute('aria-selected', 'false');
    fireEvent.click(tabTwo);
    expect(tabOne).toHaveAttribute('aria-selected', 'false');
    expect(tabTwo).toHaveAttribute('aria-selected', 'true');
  });

  it('calls onChange when tab is clicked (controlled)', () => {
    const handleChange = vi.fn();
    render(
      <Tabs value="one" onChange={handleChange}>
        <Tab value="one" label="One" data-testid="tab-one" />
        <Tab value="two" label="Two" data-testid="tab-two" />
      </Tabs>
    );
    fireEvent.click(screen.getByTestId('tab-two'));
    expect(handleChange).toHaveBeenCalledWith('two');
  });

  it('applies fullWidth variant', () => {
    render(
      <Tabs variant="fullWidth" defaultValue="one">
        <Tab value="one" label="One" data-testid="tab" />
      </Tabs>
    );
    const tabsContainer = screen.getByTestId('tab').parentElement;
    expect(tabsContainer).toHaveClass('w-full');
  });

  it('applies vertical orientation', () => {
    render(
      <Tabs orientation="vertical" defaultValue="one">
        <Tab value="one" label="One" data-testid="tab" />
      </Tabs>
    );
    const tabsContainer = screen.getByTestId('tab').parentElement;
    expect(tabsContainer).toHaveClass('flex-col', 'border-r', 'border-nest-border');
    expect(tabsContainer).not.toHaveClass('border-b');
  });

  it('applies horizontal orientation (default)', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" label="One" data-testid="tab" />
      </Tabs>
    );
    const tabsContainer = screen.getByTestId('tab').parentElement;
    expect(tabsContainer).toHaveClass('flex-row', 'border-b', 'border-nest-border');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Tabs ref={ref} defaultValue="one">
        <Tab value="one" label="One" />
      </Tabs>
    );
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.role).toBe('tablist');
  });
});

describe('Tab', () => {
  it('renders label', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" label="Test Label" data-testid="tab" />
      </Tabs>
    );
    expect(screen.getByText('Test Label')).toBeInTheDocument();
  });

  it('renders icon', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" icon={<span data-testid="icon">★</span>} label="Test" data-testid="tab" />
      </Tabs>
    );
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('applies selected styles', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" label="Selected" data-testid="tab" />
      </Tabs>
    );
    const tab = screen.getByTestId('tab');
    expect(tab).toHaveClass('text-nest-primary', 'border-nest-primary');
  });

  it('applies disabled state', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" label="Disabled" disabled data-testid="tab" />
      </Tabs>
    );
    const tab = screen.getByTestId('tab');
    expect(tab).toHaveClass('disabled:opacity-50', 'disabled:cursor-not-allowed');
    expect(tab).toBeDisabled();
  });

  it('does not call onChange when disabled', () => {
    const handleChange = vi.fn();
    render(
      <Tabs value="one" onChange={handleChange}>
        <Tab value="two" label="Disabled" disabled data-testid="tab" />
      </Tabs>
    );
    fireEvent.click(screen.getByTestId('tab'));
    expect(handleChange).not.toHaveBeenCalled();
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLButtonElement | null };
    render(
      <Tabs defaultValue="one">
        <Tab value="one" ref={ref} label="Test" />
      </Tabs>
    );
    expect(ref.current).toBeInTheDocument();
    expect(ref.current?.tagName).toBe('BUTTON');
    expect(ref.current?.role).toBe('tab');
  });
});

describe('TabPanel', () => {
  it('renders content when selected', () => {
    render(
      <Tabs defaultValue="one">
        <TabPanel value="one">Panel Content</TabPanel>
      </Tabs>
    );
    expect(screen.getByText('Panel Content')).toBeInTheDocument();
  });

  it('hides content when not selected', () => {
    render(
      <Tabs defaultValue="one">
        <TabPanel value="two">Hidden Content</TabPanel>
      </Tabs>
    );
    expect(screen.queryByText('Hidden Content')).not.toBeInTheDocument();
  });

  it('applies hidden attribute when not selected', () => {
    render(
      <Tabs defaultValue="one">
        <TabPanel value="two" data-testid="panel">Content</TabPanel>
      </Tabs>
    );
    const panel = screen.getByTestId('panel');
    expect(panel).toHaveAttribute('hidden');
  });

  it('has role="tabpanel"', () => {
    render(
      <Tabs defaultValue="one">
        <TabPanel value="one" data-testid="panel">Content</TabPanel>
      </Tabs>
    );
    expect(screen.getByTestId('panel')).toHaveAttribute('role', 'tabpanel');
  });

  it('forwards ref correctly', () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <Tabs defaultValue="one">
        <TabPanel value="one" ref={ref}>Content</TabPanel>
      </Tabs>
    );
    expect(ref.current).toBeInTheDocument();
  });
});

describe('Tabs integration', () => {
  it('shows correct panel for selected tab', () => {
    render(
      <Tabs defaultValue="one">
        <Tab value="one" label="Tab 1" data-testid="tab-1" />
        <Tab value="two" label="Tab 2" data-testid="tab-2" />
        <TabPanel value="one" data-testid="panel-1">Panel 1</TabPanel>
        <TabPanel value="two" data-testid="panel-2">Panel 2</TabPanel>
      </Tabs>
    );
    expect(screen.getByTestId('panel-1')).toBeInTheDocument();
    expect(screen.getByText('Panel 1')).toBeInTheDocument();
    expect(screen.queryByText('Panel 2')).not.toBeInTheDocument();
    fireEvent.click(screen.getByTestId('tab-2'));
    expect(screen.getByText('Panel 2')).toBeInTheDocument();
    expect(screen.queryByText('Panel 1')).not.toBeInTheDocument();
  });
});
