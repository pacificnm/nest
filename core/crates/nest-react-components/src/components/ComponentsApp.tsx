import { useState, useMemo } from 'react';
import { Button, TextField, Alert, Dialog, Snackbar, IconButton } from '@nest/components';
import {
  Box,
  Type,
  Square,
  MessageSquare,
  PanelLeftClose,
  PanelLeftOpen,
  CheckCircle,
  AlertCircle,
  AlertTriangle,
  Info,
} from 'lucide-react';

type ComponentCategory = 'inputs' | 'feedback' | 'navigation' | 'surface' | 'data-display' | 'layout';

type ComponentDef = {
  id: string;
  name: string;
  category: ComponentCategory;
  icon: React.ComponentType<{ className?: string }>;
  description: string;
};

const COMPONENTS: ComponentDef[] = [
  // Inputs
  { id: 'button', name: 'Button', category: 'inputs', icon: Square, description: 'Action buttons with variants' },
  { id: 'icon-button', name: 'IconButton', category: 'inputs', icon: Square, description: 'Icon-only buttons' },
  { id: 'text-field', name: 'TextField', category: 'inputs', icon: Type, description: 'Text input with label' },
  // Feedback
  { id: 'dialog', name: 'Dialog', category: 'feedback', icon: MessageSquare, description: 'Modal overlays' },
  { id: 'alert', name: 'Alert', category: 'feedback', icon: AlertCircle, description: 'Inline messages' },
  { id: 'snackbar', name: 'Snackbar', category: 'feedback', icon: MessageSquare, description: 'Toast notifications' },
];

const CATEGORIES: { id: ComponentCategory; label: string }[] = [
  { id: 'inputs', label: 'Inputs' },
  { id: 'feedback', label: 'Feedback' },
  { id: 'navigation', label: 'Navigation' },
  { id: 'surface', label: 'Surface' },
  { id: 'data-display', label: 'Data Display' },
  { id: 'layout', label: 'Layout' },
];

export function ComponentsApp() {
  const [selectedComponent, setSelectedComponent] = useState<string | null>('button');
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const selectedDef = useMemo(
    () => COMPONENTS.find((c) => c.id === selectedComponent),
    [selectedComponent]
  );

  const filteredComponents = useMemo(() => {
    if (!selectedDef) return COMPONENTS;
    return COMPONENTS.filter((c) => c.category === selectedDef.category);
  }, [selectedDef]);

  return (
    <div className="flex h-screen min-h-0 bg-nest-background">
      {/* Sidebar */}
      <aside
        className={[
          'flex flex-col border-r border-nest-border bg-nest-surface transition-all duration-200',
          sidebarOpen ? 'w-64' : 'w-0 overflow-hidden',
        ].join(' ')}
      >
        <div className="flex items-center justify-between border-b border-nest-border p-3">
          <h2 className="text-sm font-semibold text-nest-foreground">Components</h2>
          <IconButton
            aria-label="Toggle sidebar"
            size="small"
            onClick={() => setSidebarOpen(!sidebarOpen)}
          >
            <PanelLeftClose className="size-4" />
          </IconButton>
        </div>
        <nav className="flex-1 overflow-auto p-2">
          {CATEGORIES.map((cat) => (
            <div key={cat.id} className="mb-2">
              <h3 className="mb-1 px-2 text-xs font-medium uppercase tracking-wider text-nest-muted">
                {cat.label}
              </h3>
              {COMPONENTS.filter((c) => c.category === cat.id).map((comp) => (
                <button
                  key={comp.id}
                  type="button"
                  className={[
                    'flex w-full items-center gap-2 rounded-nest-md px-2 py-1.5 text-left text-sm transition-colors',
                    selectedComponent === comp.id
                      ? 'bg-nest-primary/10 font-medium text-nest-primary'
                      : 'text-nest-foreground hover:bg-nest-muted/10',
                  ].join(' ')}
                  onClick={() => setSelectedComponent(comp.id)}
                >
                  <comp.icon className="size-4 shrink-0" />
                  <span className="truncate">{comp.name}</span>
                </button>
              ))}
            </div>
          ))}
        </nav>
      </aside>

      {/* Toggle button when sidebar closed */}
      {!sidebarOpen && (
        <div className="flex flex-col border-r border-nest-border bg-nest-surface">
          <IconButton
            aria-label="Open sidebar"
            size="small"
            onClick={() => setSidebarOpen(true)}
            className="m-2"
          >
            <PanelLeftOpen className="size-4" />
          </IconButton>
        </div>
      )}

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        {selectedDef ? (
          <ComponentViewer component={selectedDef} />
        ) : (
          <EmptyState />
        )}
      </main>
    </div>
  );
}

function ComponentViewer({ component }: { component: ComponentDef }) {
  const [tab, setTab] = useState<'preview' | 'docs' | 'code'>('preview');

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <header className="border-b border-nest-border bg-nest-surface px-6 py-4">
        <div className="flex items-center gap-3">
          <component.icon className="size-6 text-nest-primary" />
          <div>
            <h1 className="text-lg font-semibold text-nest-foreground">{component.name}</h1>
            <p className="text-sm text-nest-muted">{component.description}</p>
          </div>
        </div>
        {/* Tabs */}
        <div className="mt-4 flex gap-1">
          <TabButton active={tab === 'preview'} onClick={() => setTab('preview')}>
            Preview
          </TabButton>
          <TabButton active={tab === 'docs'} onClick={() => setTab('docs')}>
            Documentation
          </TabButton>
          <TabButton active={tab === 'code'} onClick={() => setTab('code')}>
            Code
          </TabButton>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        {tab === 'preview' && <PreviewTab component={component} />}
        {tab === 'docs' && <DocsTab component={component} />}
        {tab === 'code' && <CodeTab component={component} />}
      </div>
    </div>
  );
}

function TabButton({
  active,
  children,
  onClick,
}: {
  active: boolean;
  children: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      className={[
        'rounded-t-nest-md px-4 py-2 text-sm transition-colors',
        active
          ? 'bg-nest-background font-medium text-nest-primary'
          : 'text-nest-muted hover:text-nest-foreground',
      ].join(' ')}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

function PreviewTab({ component }: { component: ComponentDef }) {
  return (
    <div className="space-y-6">
      <ComponentPreview componentId={component.id} />
    </div>
  );
}

function ComponentPreview({ componentId }: { componentId: string }) {
  // Live previews for each component
  switch (componentId) {
    case 'button':
      return (
        <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
          <h3 className="mb-4 text-sm font-medium text-nest-foreground">Live Preview</h3>
          <div className="flex flex-wrap gap-4">
            <Button variant="contained">Contained</Button>
            <Button variant="outlined">Outlined</Button>
            <Button variant="text">Text</Button>
            <Button variant="contained" color="secondary">
              Secondary
            </Button>
            <Button variant="contained" color="error">
              Error
            </Button>
            <Button startIcon={<CheckCircle className="size-4" />}>With Icon</Button>
            <Button loading>Loading</Button>
            <Button disabled>Disabled</Button>
          </div>
        </div>
      );

    case 'text-field':
      return (
        <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
          <h3 className="mb-4 text-sm font-medium text-nest-foreground">Live Preview</h3>
          <div className="space-y-4 max-w-md">
            <TextField label="Basic Input" placeholder="Enter text..." />
            <TextField label="With Helper" helperText="This is helper text" />
            <TextField label="Error State" error="This field is required" />
          </div>
        </div>
      );

    case 'dialog':
      return <DialogPreview />;

    case 'alert':
      return (
        <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
          <h3 className="mb-4 text-sm font-medium text-nest-foreground">Live Preview</h3>
          <div className="space-y-4">
            <Alert severity="success">Success! Your changes have been saved.</Alert>
            <Alert severity="error">Error! Something went wrong.</Alert>
            <Alert severity="warning">Warning! Please review before continuing.</Alert>
            <Alert severity="info">Info: A new update is available.</Alert>
          </div>
        </div>
      );

    case 'snackbar':
      return <SnackbarPreview />;

    case 'icon-button':
      return (
        <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
          <h3 className="mb-4 text-sm font-medium text-nest-foreground">Live Preview</h3>
          <div className="flex gap-2">
            <IconButton aria-label="Settings">
              <Box className="size-5" />
            </IconButton>
            <IconButton aria-label="Delete" color="error">
              <AlertCircle className="size-5" />
            </IconButton>
            <IconButton aria-label="Info" color="primary">
              <Info className="size-5" />
            </IconButton>
            <IconButton aria-label="Warning" color="warning">
              <AlertTriangle className="size-5" />
            </IconButton>
          </div>
        </div>
      );

    default:
      return (
        <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-6 text-center text-nest-muted">
          Preview coming soon for {componentId}
        </div>
      );
  }
}

function DialogPreview() {
  const [open, setOpen] = useState(false);

  return (
    <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
      <h3 className="mb-4 text-sm font-medium text-nest-foreground">Live Preview</h3>
      <Button variant="contained" onClick={() => setOpen(true)}>
        Open Dialog
      </Button>

      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Sample Dialog"
        actions={
          <>
            <Button variant="text" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button variant="contained" onClick={() => setOpen(false)}>
              Confirm
            </Button>
          </>
        }
      >
        <p className="text-nest-foreground">
          This is a sample dialog demonstrating the Dialog component from @nest/components.
        </p>
      </Dialog>
    </div>
  );
}

function SnackbarPreview() {
  const [open, setOpen] = useState(false);

  return (
    <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
      <h3 className="mb-4 text-sm font-medium text-nest-foreground">Live Preview</h3>
      <Button variant="contained" onClick={() => setOpen(true)}>
        Show Snackbar
      </Button>

      <Snackbar open={open} onClose={() => setOpen(false)} severity="success">
        Action completed successfully!
      </Snackbar>
    </div>
  );
}

function DocsTab({ component }: { component: ComponentDef }) {
  return (
    <div className="prose prose-nest max-w-none">
      <h2>Documentation</h2>
      <p className="text-nest-muted">
        Full documentation for <strong>{component.name}</strong> is available in the component source.
      </p>

      <div className="mt-6 rounded-nest-md border border-nest-border bg-nest-surface p-4">
        <h3 className="text-sm font-medium text-nest-foreground">When to Use</h3>
        <p className="mt-2 text-sm text-nest-muted">
          See the component's `.docs.md` file for detailed usage guidelines, props reference,
          and best practices.
        </p>
      </div>

      <div className="mt-6 rounded-nest-md border border-nest-border bg-nest-surface p-4">
        <h3 className="text-sm font-medium text-nest-foreground">Props</h3>
        <p className="mt-2 text-sm text-nest-muted">
          Component props are fully typed. Import the component and hover over it in your IDE
          to see the complete props interface.
        </p>
      </div>
    </div>
  );
}

function CodeTab({ component }: { component: ComponentDef }) {
  const codeExamples: Record<string, string> = {
    button: `<Button variant="contained" color="primary">
  Click me
</Button>

<Button variant="outlined" startIcon={<Save />}>
  Save
</Button>

<Button loading onClick={handleSubmit}>
  Submit
</Button>`,
    'text-field': `<TextField
  label="Email"
  value={email}
  onChange={(e) => setEmail(e.target.value)}
  helperText="We'll never share your email"
/>

<TextField
  label="Password"
  type="password"
  error={passwordError}
  startAdornment={<Lock />}
/>`,
    dialog: `<Dialog
  open={open}
  onClose={() => setOpen(false)}
  title="Confirm Delete"
  actions={
    <>
      <Button variant="text" onClick={() => setOpen(false)}>
        Cancel
      </Button>
      <Button variant="contained" color="error" onClick={handleDelete}>
        Delete
      </Button>
    </>
  }
>
  <p>Are you sure you want to delete this item?</p>
</Dialog>`,
    alert: `<Alert severity="success" onClose={() => setOpen(false)}>
  Operation completed successfully!
</Alert>

<Alert
  severity="error"
  action={<Button size="small">Retry</Button>}
>
  Connection failed
</Alert>`,
    snackbar: `<Snackbar
  open={open}
  onClose={() => setOpen(false)}
  severity="success"
  action={<Button size="small" onClick={handleUndo}>Undo</Button>}
>
  Item deleted
</Snackbar>`,
    'icon-button': `<IconButton aria-label="delete" color="error">
  <TrashIcon />
</IconButton>

<IconButton aria-label="settings" size="large">
  <SettingsIcon />
</IconButton>`,
  };

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-medium text-nest-foreground">Usage Example</h3>
      <pre className="overflow-auto rounded-nest-md bg-nest-background p-4 text-sm text-nest-foreground">
        <code>{codeExamples[component.id] || '// Example coming soon'}</code>
      </pre>
      <p className="text-sm text-nest-muted">
        Import from <code className="rounded bg-nest-surface px-1 py-0.5">@nest/components</code>
      </p>
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex h-full items-center justify-center text-nest-muted">
      <div className="text-center">
        <Box className="mx-auto mb-4 size-12 opacity-50" />
        <h2 className="text-lg font-medium text-nest-foreground">Select a Component</h2>
        <p className="mt-2 text-sm">Choose a component from the sidebar to view its preview and documentation.</p>
      </div>
    </div>
  );
}
