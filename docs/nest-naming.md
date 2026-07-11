# Nest Project Naming Conventions

## Official Product Names

| Name | Description | Location |
|------|-------------|----------|
| **Nest Desktop** | The main desktop shell application (Tauri + React) - the "operating system" window manager | `ui/` (formerly `nest-desktop`) |
| **Nest UI Components** | The reusable React component library (`@nest/components`) | `core/crates/nest-react-components/` |
| **Nest UI Browser** | The component viewer/gallery app inside Nest Desktop (shows Nest UI Components) | Built into Nest UI Components, launched as "Component Library" app |

## Application Names in Nest Desktop

These are the apps that appear in the Nest Desktop shell:

| App Name | Category | Description |
|----------|----------|-------------|
| **Help** | System | Browse Nest framework documentation |
| **Component Library** | Development | Launches the Nest UI Browser to view Nest UI Components |
| **Kiwi** | (varies) | Media library manager (external app) |
| **Loon** | (varies) | Media player (external app) |

## Terminology Guide

| Term | Meaning |
|------|---------|
| **Nest** | The overall framework and monorepo |
| **Nest Desktop** | The desktop shell/window manager (the "OS") |
| **Nest Shell** | Alternative name for Nest Desktop (the window manager environment) |
| **Nest UI Components** | The component library (code in `@nest/components`) |
| **Nest UI Browser** | The viewer app that displays Nest UI Components with demos/docs |
| **Component Library** | The app icon in Nest Desktop that launches Nest UI Browser |

## Quick Reference

- Working on the **desktop shell/window manager**? → **Nest Desktop** (`ui/`)
- Working on **buttons, text fields, dialogs**? → **Nest UI Components** (`core/crates/nest-react-components/`)
- Working on the **component viewer/gallery**? → **Nest UI Browser** (inside Nest UI Components)
- User clicks **Component Library** icon → Opens **Nest UI Browser**

## Related Documentation

- [Nest Architecture](./architecture.md)
- [Nest UI Components Plan](./plan/nest-react-components-v1.md)
