/**
 * Nest UI Browser - Component viewer for Nest UI Components
 *
 * Displays the @nest/components library with:
 * - Left sidebar menu of all components
 * - Main area with component documentation and live examples
 */

import { useState, useMemo } from "react";
import {
  Button,
  TextField,
  IconButton,
  Alert,
  Dialog,
  Snackbar,
  AppBar,
  Toolbar,
  Menu,
  MenuItem,
  MenuDivider,
  MenuBar,
  MenuBarItem,
} from "@nest/components";
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
  Layers,
  Eye,
  Code,
  BookOpen,
  PanelTop,
  ListTree,
  Save,
  FolderOpen,
  Settings,
} from "lucide-react";

type ComponentCategory = "inputs" | "feedback" | "navigation" | "surface" | "data-display" | "layout";

type ComponentDef = {
  id: string;
  name: string;
  category: ComponentCategory;
  icon: React.ComponentType<{ className?: string }>;
  description: string;
};

const COMPONENTS: ComponentDef[] = [
  // Inputs
  { id: "button", name: "Button", category: "inputs", icon: Square, description: "Action buttons with variants" },
  { id: "icon-button", name: "IconButton", category: "inputs", icon: Square, description: "Icon-only buttons" },
  { id: "text-field", name: "TextField", category: "inputs", icon: Type, description: "Text input with label" },
  // Feedback
  { id: "dialog", name: "Dialog", category: "feedback", icon: MessageSquare, description: "Modal overlays" },
  { id: "alert", name: "Alert", category: "feedback", icon: AlertCircle, description: "Inline messages" },
  { id: "snackbar", name: "Snackbar", category: "feedback", icon: MessageSquare, description: "Toast notifications" },
  // Navigation
  { id: "app-bar", name: "AppBar", category: "navigation", icon: PanelTop, description: "Top application bar + toolbar" },
  { id: "menu", name: "Menu", category: "navigation", icon: ListTree, description: "Dropdown menus and File-menu bars" },
];

const CATEGORIES: { id: ComponentCategory; label: string }[] = [
  { id: "inputs", label: "Inputs" },
  { id: "feedback", label: "Feedback" },
  { id: "navigation", label: "Navigation" },
  { id: "surface", label: "Surface" },
  { id: "data-display", label: "Data Display" },
  { id: "layout", label: "Layout" },
];

export function NestUIBrowser() {
  const [selectedComponent, setSelectedComponent] = useState<string | null>("button");
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [activeTab, setActiveTab] = useState<"preview" | "docs" | "code">("preview");

  const selectedDef = useMemo(
    () => COMPONENTS.find((c) => c.id === selectedComponent),
    [selectedComponent]
  );

  return (
    <div className="nest-ui-browser">
      {/* Sidebar */}
      <aside className={`nest-ui-browser-sidebar ${sidebarOpen ? "open" : "closed"}`}>
        <div className="nest-ui-browser-sidebar-header">
          <div className="flex items-center gap-2">
            <Layers className="size-5 text-nest-primary" />
            <span className="font-semibold text-nest-foreground">Nest UI Components</span>
          </div>
          <IconButton
            aria-label="Toggle sidebar"
            size="small"
            onClick={() => setSidebarOpen(!sidebarOpen)}
          >
            {sidebarOpen ? <PanelLeftClose className="size-4" /> : <PanelLeftOpen className="size-4" />}
          </IconButton>
        </div>
        <nav className="nest-ui-browser-toc">
          {CATEGORIES.map((cat) => (
            <div key={cat.id} className="nest-ui-browser-toc-category">
              <h3 className="nest-ui-browser-toc-category-label">{cat.label}</h3>
              {COMPONENTS.filter((c) => c.category === cat.id).map((comp) => (
                <button
                  key={comp.id}
                  type="button"
                  className={`nest-ui-browser-toc-item ${selectedComponent === comp.id ? "active" : ""}`}
                  onClick={() => {
                    setSelectedComponent(comp.id);
                    setActiveTab("preview");
                  }}
                >
                  <comp.icon className="size-4 shrink-0" />
                  <span className="truncate">{comp.name}</span>
                </button>
              ))}
            </div>
          ))}
        </nav>
      </aside>

      {/* Main content */}
      <main className="nest-ui-browser-content">
        {selectedDef ? (
          <div className="nest-ui-browser-component-view">
            {/* Header */}
            <header className="nest-ui-browser-component-header">
              <div className="flex items-center gap-3">
                <selectedDef.icon className="size-6 text-nest-primary" />
                <div>
                  <h1 className="nest-ui-browser-component-title">{selectedDef.name}</h1>
                  <p className="nest-ui-browser-component-description">{selectedDef.description}</p>
                </div>
              </div>
              {/* Tabs */}
              <div className="nest-ui-browser-tabs">
                <button
                  type="button"
                  className={`nest-ui-browser-tab ${activeTab === "preview" ? "active" : ""}`}
                  onClick={() => setActiveTab("preview")}
                >
                  <Eye className="mr-1.5 size-3.5" />
                  Preview
                </button>
                <button
                  type="button"
                  className={`nest-ui-browser-tab ${activeTab === "docs" ? "active" : ""}`}
                  onClick={() => setActiveTab("docs")}
                >
                  <BookOpen className="mr-1.5 size-3.5" />
                  Documentation
                </button>
                <button
                  type="button"
                  className={`nest-ui-browser-tab ${activeTab === "code" ? "active" : ""}`}
                  onClick={() => setActiveTab("code")}
                >
                  <Code className="mr-1.5 size-3.5" />
                  Code
                </button>
              </div>
            </header>

            {/* Content Area */}
            <div className="nest-ui-browser-component-body">
              {activeTab === "preview" && <ComponentPreview componentId={selectedDef.id} />}
              {activeTab === "docs" && <ComponentDocs componentId={selectedDef.id} />}
              {activeTab === "code" && <ComponentCode componentId={selectedDef.id} />}
            </div>
          </div>
        ) : (
          <div className="nest-ui-browser-empty">
            <Layers className="nest-ui-browser-empty-icon" />
            <h2 className="nest-ui-browser-empty-title">Select a Component</h2>
            <p className="nest-ui-browser-empty-text">
              Choose a component from the sidebar to view its preview and documentation.
            </p>
          </div>
        )}
      </main>
    </div>
  );
}

function ComponentPreview({ componentId }: { componentId: string }) {
  switch (componentId) {
    case "button":
      return <ButtonPreview />;
    case "text-field":
      return <TextFieldPreview />;
    case "dialog":
      return <DialogPreview />;
    case "alert":
      return <AlertPreview />;
    case "snackbar":
      return <SnackbarPreview />;
    case "icon-button":
      return <IconButtonPreview />;
    case "app-bar":
      return <AppBarPreview />;
    case "menu":
      return <MenuPreview />;
    default:
      return (
        <div className="nest-ui-browser-preview-placeholder">
          <p>Preview coming soon for {componentId}</p>
        </div>
      );
  }
}

function ButtonPreview() {
  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>
      <div className="nest-ui-browser-preview-grid">
        <div className="nest-ui-browser-preview-card">
          <span className="nest-ui-browser-preview-label">Variants</span>
          <div className="flex flex-wrap gap-3">
            <Button variant="contained">Contained</Button>
            <Button variant="outlined">Outlined</Button>
            <Button variant="text">Text</Button>
          </div>
        </div>
        <div className="nest-ui-browser-preview-card">
          <span className="nest-ui-browser-preview-label">Colors</span>
          <div className="flex flex-wrap gap-3">
            <Button variant="contained" color="primary">Primary</Button>
            <Button variant="contained" color="secondary">Secondary</Button>
            <Button variant="contained" color="error">Error</Button>
          </div>
        </div>
        <div className="nest-ui-browser-preview-card">
          <span className="nest-ui-browser-preview-label">With Icons</span>
          <div className="flex flex-wrap gap-3">
            <Button startIcon={<CheckCircle className="size-4" />}>With Icon</Button>
            <Button loading>Loading</Button>
            <Button disabled>Disabled</Button>
          </div>
        </div>
      </div>
    </div>
  );
}

function TextFieldPreview() {
  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>
      <div className="nest-ui-browser-preview-grid-vertical">
        <div className="nest-ui-browser-preview-card">
          <span className="nest-ui-browser-preview-label">Basic Input</span>
          <TextField label="First Name" placeholder="Enter your name" />
        </div>
        <div className="nest-ui-browser-preview-card">
          <span className="nest-ui-browser-preview-label">With Helper Text</span>
          <TextField label="Email" helperText="We'll never share your email" />
        </div>
        <div className="nest-ui-browser-preview-card">
          <span className="nest-ui-browser-preview-label">Error State</span>
          <TextField label="Username" error="This username is taken" defaultValue="taken" />
        </div>
      </div>
    </div>
  );
}

function DialogPreview() {
  const [open, setOpen] = useState(false);

  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>
      <div className="nest-ui-browser-preview-card">
        <Button variant="contained" onClick={() => setOpen(true)}>Open Dialog</Button>
      </div>

      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Sample Dialog"
        actions={
          <>
            <Button variant="text" onClick={() => setOpen(false)}>Cancel</Button>
            <Button variant="contained" onClick={() => setOpen(false)}>Confirm</Button>
          </>
        }
      >
        <p className="text-nest-foreground">
          This is a sample dialog demonstrating the Dialog component from Nest UI Components.
        </p>
      </Dialog>
    </div>
  );
}

function AlertPreview() {
  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>
      <div className="nest-ui-browser-preview-grid-vertical">
        <Alert severity="success"><strong>Success!</strong> Your changes have been saved.</Alert>
        <Alert severity="error"><strong>Error!</strong> Something went wrong.</Alert>
        <Alert severity="warning"><strong>Warning!</strong> Please review before continuing.</Alert>
        <Alert severity="info"><strong>Info:</strong> A new update is available.</Alert>
      </div>
    </div>
  );
}

function SnackbarPreview() {
  const [open, setOpen] = useState(false);

  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>
      <div className="nest-ui-browser-preview-card">
        <Button variant="contained" onClick={() => setOpen(true)}>Show Snackbar</Button>
      </div>

      <Snackbar open={open} onClose={() => setOpen(false)} severity="success">
        Action completed successfully!
      </Snackbar>
    </div>
  );
}

function IconButtonPreview() {
  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>
      <div className="nest-ui-browser-preview-card">
        <div className="flex gap-3">
          <IconButton aria-label="settings">
            <Box className="size-5" />
          </IconButton>
          <IconButton aria-label="delete" color="error">
            <AlertCircle className="size-5" />
          </IconButton>
          <IconButton aria-label="info" color="primary">
            <Info className="size-5" />
          </IconButton>
          <IconButton aria-label="warning" color="warning">
            <AlertTriangle className="size-5" />
          </IconButton>
        </div>
      </div>
    </div>
  );
}

function AppBarPreview() {
  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>

      <div className="nest-ui-browser-preview-card">
        <span className="nest-ui-browser-preview-label">Basic Toolbar</span>
        <div className="overflow-hidden rounded-nest-md border border-nest-border">
          <AppBar>
            <Toolbar>
              <span className="font-semibold text-nest-foreground">My App</span>
              <span className="flex-1" />
              <IconButton aria-label="Save" size="small">
                <Save className="size-4" />
              </IconButton>
              <IconButton aria-label="Settings" size="small">
                <Settings className="size-4" />
              </IconButton>
            </Toolbar>
          </AppBar>
        </div>
      </div>

      <div className="nest-ui-browser-preview-card">
        <span className="nest-ui-browser-preview-label">File Menu (Kiwi / Swift style)</span>
        <p className="mb-2 text-xs text-nest-muted">
          AppBar composed with MenuBar — the pattern used by Kiwi and Swift's top chrome.
        </p>
        <div className="overflow-hidden rounded-nest-md border border-nest-border">
          <AppBar elevation={false}>
            <Toolbar variant="dense">
              <MenuBar>
                <MenuBarItem id="file" label="File">
                  <MenuItem endAdornment="Ctrl/Cmd+O" onClick={() => {}}>
                    <FolderOpen className="size-3.5" />
                    Open…
                  </MenuItem>
                  <MenuItem endAdornment="Ctrl/Cmd+S" onClick={() => {}}>
                    <Save className="size-3.5" />
                    Save
                  </MenuItem>
                  <MenuDivider />
                  <MenuItem disabled>Open Recent</MenuItem>
                </MenuBarItem>
                <MenuBarItem id="edit" label="Edit">
                  <MenuItem endAdornment="Ctrl/Cmd+Z" onClick={() => {}}>
                    Undo
                  </MenuItem>
                  <MenuItem endAdornment="Ctrl/Cmd+Shift+Z" onClick={() => {}}>
                    Redo
                  </MenuItem>
                </MenuBarItem>
                <MenuBarItem id="help" label="Help">
                  <MenuItem onClick={() => {}}>About</MenuItem>
                </MenuBarItem>
              </MenuBar>
              <span className="flex-1" />
              <span className="self-center truncate text-[11px] text-nest-muted">Project Title</span>
            </Toolbar>
          </AppBar>
        </div>
      </div>
    </div>
  );
}

function MenuPreview() {
  const [open, setOpen] = useState(false);

  return (
    <div className="nest-ui-browser-preview-section">
      <h3 className="nest-ui-browser-preview-title">Live Preview</h3>
      <div className="nest-ui-browser-preview-card">
        <span className="nest-ui-browser-preview-label">Standalone Dropdown</span>
        <div className="relative inline-block">
          <Button variant="outlined" onClick={() => setOpen((value) => !value)}>
            Options
          </Button>
          <Menu open={open} onClose={() => setOpen(false)}>
            <MenuItem onClick={() => setOpen(false)}>Rename</MenuItem>
            <MenuItem onClick={() => setOpen(false)}>Duplicate</MenuItem>
            <MenuDivider />
            <MenuItem danger onClick={() => setOpen(false)}>
              Delete
            </MenuItem>
          </Menu>
        </div>
      </div>
      <p className="text-xs text-nest-muted">
        See the <strong>AppBar</strong> component for the File/Edit/Help menu-bar pattern built on top of Menu.
      </p>
    </div>
  );
}

function ComponentDocs({ componentId }: { componentId: string }) {
  const docs: Record<string, { usage: string; props: string[] }> = {
    button: {
      usage: `<Button variant="contained" color="primary">\n  Click me\n</Button>`,
      props: ["variant: 'contained' | 'outlined' | 'text'", "color: 'primary' | 'secondary' | 'error'", "size: 'small' | 'medium' | 'large'", "startIcon?: ReactNode", "endIcon?: ReactNode", "loading?: boolean", "disabled?: boolean"],
    },
    "text-field": {
      usage: `<TextField\n  label="Email"\n  value={email}\n  onChange={(e) => setEmail(e.target.value)}\n  helperText="We'll never share your email"\n/>`,
      props: ["label?: string", "value?: string", "onChange?: (e) => void", "error?: string", "helperText?: ReactNode", "variant: 'outlined' | 'filled' | 'standard'", "multiline?: boolean", "rows?: number"],
    },
    dialog: {
      usage: `<Dialog\n  open={open}\n  onClose={() => setOpen(false)}\n  title="Confirm"\n  actions={<Button onClick={handleConfirm}>OK</Button>}\n>\n  <p>Are you sure?</p>\n</Dialog>`,
      props: ["open: boolean (required)", "onClose: () => void", "title?: ReactNode", "actions?: ReactNode", "disableBackdropClick?: boolean", "disableEscapeKeyDown?: boolean"],
    },
    alert: {
      usage: `<Alert severity="success" onClose={() => setOpen(false)}>\n  Operation completed!\n</Alert>`,
      props: ["severity: 'success' | 'error' | 'warning' | 'info'", "variant: 'filled' | 'outlined' | 'standard'", "icon?: ReactNode", "onClose?: () => void", "action?: ReactNode"],
    },
    snackbar: {
      usage: `<Snackbar\n  open={open}\n  onClose={() => setOpen(false)}\n  severity="success"\n>\n  Message here\n</Snackbar>`,
      props: ["open: boolean (required)", "onClose: () => void", "severity?: 'success' | 'error' | 'warning' | 'info'", "autoHideDuration?: number", "action?: ReactNode", "position?: ToastPosition"],
    },
    "icon-button": {
      usage: `<IconButton aria-label="delete" color="error">\n  <TrashIcon />\n</IconButton>`,
      props: ["aria-label: string (required)", "color: 'default' | 'primary' | 'error' | etc.", "size: 'small' | 'medium' | 'large'", "disabled?: boolean"],
    },
    "app-bar": {
      usage: `<AppBar>\n  <Toolbar>\n    <span>My App</span>\n  </Toolbar>\n</AppBar>`,
      props: [
        "position: 'static' | 'fixed' | 'sticky'",
        "color: 'surface' | 'primary' | 'transparent'",
        "elevation?: boolean",
        "— Toolbar —",
        "variant: 'regular' | 'dense'",
      ],
    },
    menu: {
      usage: `<div className="relative inline-block">\n  <Button onClick={() => setOpen(true)}>Options</Button>\n  <Menu open={open} onClose={() => setOpen(false)}>\n    <MenuItem onClick={() => { save(); setOpen(false); }}>Save</MenuItem>\n    <MenuDivider />\n    <MenuItem danger onClick={() => { remove(); setOpen(false); }}>Delete</MenuItem>\n  </Menu>\n</div>`,
      props: [
        "open: boolean (required)",
        "onClose: () => void (required) — call it yourself from each MenuItem's onClick",
        "— MenuItem —",
        "danger?: boolean",
        "endAdornment?: ReactNode",
        "disabled?: boolean",
        "— MenuBar / MenuBarItem —",
        "MenuBarItem id: string (required, unique per MenuBar)",
        "MenuBarItem label: string",
      ],
    },
  };

  const doc = docs[componentId];

  if (!doc) {
    return (
      <div className="nest-ui-browser-docs-placeholder">
        <p>Documentation coming soon</p>
      </div>
    );
  }

  return (
    <div className="nest-ui-browser-docs">
      <section className="nest-ui-browser-docs-section">
        <h3 className="nest-ui-browser-docs-title">Usage</h3>
        <pre className="nest-ui-browser-docs-code">{doc.usage}</pre>
      </section>
      <section className="nest-ui-browser-docs-section">
        <h3 className="nest-ui-browser-docs-title">Props</h3>
        <ul className="nest-ui-browser-docs-props">
          {doc.props.map((prop) => (
            <li key={prop} className="nest-ui-browser-docs-prop">
              <code className="nest-ui-browser-docs-prop-code">{prop.split(":")[0]}</code>
              <span className="nest-ui-browser-docs-prop-type">{prop.includes(":") ? prop.split(":")[1] : ""}</span>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

function ComponentCode({ componentId }: { componentId: string }) {
  const examples: Record<string, string> = {
    button: `<Button variant="contained" color="primary">
  Click me
</Button>

<Button variant="outlined" startIcon={<Save />}>
  Save
</Button>

<Button loading onClick={handleSubmit}>
  Submit
</Button>`,
    "text-field": `<TextField
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
    "icon-button": `<IconButton aria-label="delete" color="error">
  <TrashIcon />
</IconButton>

<IconButton aria-label="settings" size="large">
  <SettingsIcon />
</IconButton>`,
    "app-bar": `<AppBar>
  <Toolbar>
    <span className="font-semibold">My App</span>
    <span className="flex-1" />
    <IconButton aria-label="Save" size="small">
      <Save className="size-4" />
    </IconButton>
  </Toolbar>
</AppBar>

// File-menu bar (Kiwi / Swift style) — AppBar + MenuBar composed together
<AppBar elevation={false}>
  <Toolbar variant="dense">
    <MenuBar>
      <MenuBarItem id="file" label="File">
        <MenuItem endAdornment="Ctrl/Cmd+O" onClick={openFile}>
          Open…
        </MenuItem>
        <MenuDivider />
        <MenuItem endAdornment="Ctrl/Cmd+S" onClick={save}>
          Save
        </MenuItem>
      </MenuBarItem>
      <MenuBarItem id="help" label="Help">
        <MenuItem onClick={showAbout}>About</MenuItem>
      </MenuBarItem>
    </MenuBar>
  </Toolbar>
</AppBar>`,
    menu: `<div className="relative inline-block">
  <Button onClick={() => setOpen(true)}>Options</Button>
  <Menu open={open} onClose={() => setOpen(false)}>
    <MenuItem onClick={() => { rename(); setOpen(false); }}>
      Rename
    </MenuItem>
    <MenuDivider />
    <MenuItem danger onClick={() => { remove(); setOpen(false); }}>
      Delete
    </MenuItem>
  </Menu>
</div>`,
  };

  return (
    <div className="nest-ui-browser-code">
      <pre className="nest-ui-browser-code-block">
        <code>{examples[componentId] || "// Example coming soon"}</code>
      </pre>
      <p className="nest-ui-browser-code-hint">
        Import from <code>@nest/components</code>
      </p>
    </div>
  );
}
