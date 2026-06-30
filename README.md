# Nest Framework

## Vision & Architecture (Draft)

### Overview

Nest is a modular application framework built on top of **Rust** and **egui** for developing modern desktop applications.

Rather than being a GUI library itself, Nest provides the application architecture that sits above egui. It supplies the infrastructure commonly found in mature frameworks such as WPF, Qt, ASP.NET Core, Flutter, and Spring while embracing Rust's philosophy of composability and zero-cost abstractions.

The primary design goal is:

> **Only compile and load the functionality your application actually needs.**

Nest should be equally suitable for a small utility, a business application, or a full desktop IDE such as Kiwi.

---

# Core Principles

## Modular First

Every major feature lives in its own crate.

Applications should opt into functionality instead of inheriting a large framework.

```text
nest-core
nest-app
nest-ui
nest-forms
nest-validation
nest-data
nest-routing
nest-theme
nest-commands
nest-events
nest-state
nest-settings
nest-plugins
nest-docking
nest-notifications
nest-tasks
nest-auth
nest-files
nest-git
nest-ai
```

---

## Lightweight Core

The framework core should remain intentionally small.

Its responsibility is to define contracts—not implement features.

Responsibilities include:

* Module system
* Application lifecycle
* Service registry / dependency injection
* Application context
* Event definitions
* Error handling
* Trait definitions
* Version information
* Feature registration

Example:

```rust
trait Module {
    fn configure(&self, app: &mut AppBuilder);
}

trait Service {}

struct AppContext;
```

The core should have very few dependencies.

---

## Batteries Included, But Optional

Every capability beyond the core is implemented as a module.

Examples include:

* Docking
* Forms
* Validation
* Database support
* Plugin loading
* Themes
* Notifications
* Background tasks
* Git integration
* AI integration
* Terminal integration

Applications simply include the modules they require.

---

# Framework Layout

```text
Nest Workspace
│
├── nest-core
├── nest-app
├── nest-ui
├── nest-forms
├── nest-validation
├── nest-data
├── nest-routing
├── nest-theme
├── nest-commands
├── nest-events
├── nest-state
├── nest-settings
├── nest-plugins
├── nest-docking
├── nest-notifications
├── nest-tasks
├── nest-auth
├── nest-files
├── nest-git
├── nest-ai
└── ...
```

---

# Module Responsibilities

## nest-app

Application startup.

```rust
App::new()
    .module(UiModule)
    .module(ThemeModule)
    .run();
```

---

## nest-ui

Reusable interface components.

Examples:

* Buttons
* Toolbars
* Tables
* Tree Views
* Tabs
* Splitters
* Property Grid
* Dialogs
* Status Bar
* Menus
* Navigation Controls

---

## nest-forms

Provides a complete form framework.

Features:

* Form layouts
* Labels
* Automatic field generation
* Dirty tracking
* Save / Cancel workflow
* Keyboard navigation
* Validation integration

---

## nest-validation

Independent validation library.

Validation should be reusable across:

* Desktop UI
* REST APIs
* CLI tools
* Background services

Example:

```rust
#[derive(Validate)]
struct Person {
    #[required]
    name: String,

    #[email]
    email: String
}
```

---

## nest-data

Database abstraction.

Support multiple providers through adapters.

Potential providers:

* SQLite
* PostgreSQL
* MySQL
* SQL Server
* SeaORM
* Diesel
* sqlx

Applications only compile the providers they use.

---

## nest-docking

Dockable interface framework.

Supports:

* Panels
* Floating windows
* Tab groups
* Layout persistence
* Split views
* Workspace management

Ideal for applications such as:

* IDEs
* Database tools
* Log viewers
* Monitoring applications

---

## nest-commands

Global command system.

Supports:

* Command palette
* Keyboard shortcuts
* Context menus
* Toolbar actions
* Searchable commands

---

## nest-settings

Application configuration.

Examples:

* User preferences
* Theme
* Window layouts
* Workspace settings
* Recent files

---

## nest-notifications

Central notification system.

Supports:

* Toast notifications
* Progress
* Alerts
* Background job updates

---

## nest-tasks

Background work manager.

Supports:

* Async tasks
* Progress reporting
* Cancellation
* Scheduling

---

## nest-plugins

Plugin architecture.

Plugins register themselves with the application.

Example:

```rust
pub struct GitPlugin;

impl Plugin for GitPlugin {
    fn register(&self, app: &mut AppBuilder) {
        app.register_panel(GitPanel);
        app.register_commands();
        app.register_services();
    }
}
```

---

# Unified Registration Model

Everything in Nest should register through the same builder.

Examples:

```rust
app.register_service::<GitService>();

app.register_panel::<GitPanel>();

app.register_command::<OpenRepository>();

app.register_theme::<DarkTheme>();

app.register_validator::<EmailValidator>();

app.register_database::<SqliteProvider>();
```

This keeps the framework predictable and extensible.

---

# Application Builder

Small applications:

```rust
App::new()
    .module(UiModule)
    .run();
```

Larger applications:

```rust
App::new()
    .module(UiModule)
    .module(ThemeModule)
    .module(DockingModule)
    .module(DataModule)
    .module(GitModule)
    .module(AiModule)
    .module(TerminalModule)
    .module(PluginModule)
    .run();
```

Applications should only include the modules they require.

---

# Automatic Forms (Future)

Nest should eventually support automatic form generation.

Example:

```rust
#[derive(NestForm)]
struct Customer {

    #[required]
    name: String,

    #[email]
    email: String,

    #[range(18,120)]
    age: u32,
}
```

Generating a form should be as simple as:

```rust
ui.form(&mut customer);
```

Automatically providing:

* Labels
* Controls
* Validation
* Error messages
* Dirty tracking
* Keyboard navigation
* Save state
* Responsive layout

---

# Long-Term Vision

Nest is intended to become the missing application framework for the Rust desktop ecosystem.

It should provide the architectural foundation that egui intentionally leaves to application developers.

Rather than replacing egui, Nest complements it by supplying:

* Application architecture
* State management
* Forms
* Validation
* Navigation
* Plugins
* Data access
* Theming
* Docking
* Background services
* Notifications
* Developer productivity

---

# Relationship to Kiwi

Kiwi is envisioned as the flagship application built on Nest.

This serves two purposes:

1. Nest is continuously validated by building a complex, real-world application.
2. Kiwi benefits from reusable, well-tested infrastructure rather than custom application-specific code.

Other applications—including Finch and future tools—can leverage the same framework, demonstrating that Nest is a general-purpose platform rather than one built solely for a single IDE.

---

# Philosophy

Nest should remain:

* Modular
* Lightweight
* Extensible
* Strongly typed
* Cross-platform
* High performance
* Keyboard-first
* Plugin-oriented
* Developer friendly

The goal is to make building sophisticated Rust desktop applications feel as productive as using mature frameworks in other ecosystems, while preserving Rust's focus on performance, safety, and composability.
