# nest-gui

Desktop GUI host for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-gui`](../../core/crates/nest-gui)

`nest-gui` owns eframe startup, the egui frame loop, and window options. It does not own CSV, HTTP, data, validation, or logging logic beyond host initialization.

## Quick start

```rust
use nest_core::AppContext;
use nest_error::NestResult;
use nest_gui::{GuiApp, GuiView};
use nest_theme::ThemeModule;

struct MainView;

impl GuiView for MainView {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &AppContext) -> NestResult<()> {
        ui.heading("Hello, Kiwi");
        Ok(())
    }
}

fn main() {
    GuiApp::new("kiwi")
        .module(ThemeModule::default())
        .view(MainView)
        .run();
}
```

## Startup options

| Flag | Effect |
|------|--------|
| `--config <path>` | Load explicit config file |
| `--log-level <level>` | Override log level |
| `--log-file <path>` | Log file directory/path |
| `--no-color` | Plain error output on stderr |
| `--title <title>` | Window title |
| `--width` / `--height` | Window size in pixels |
| `--debug` | Debug logging |

## `[gui]` config section

```toml
[gui]
title = "Kiwi"
width = 1280
height = 800
vsync = true
```

CLI flags override config; config overrides defaults.

## Logging

GUI hosts default to **file-only** logging via `LoggingConfig::for_gui` (no stdout while the window is active).

## Theme

Register `ThemeModule` explicitly. v1 applies a stub light/dark mapping to `egui::Visuals`; full token mapping is deferred to `nest-egui-theme`.

## Related

- [Implementation plan](../plan/nest-gui-v1.md)
- [nest-tui](../nest-tui/README.md) — terminal host
- [nest-theme](../nest-theme/README.md) — theme service
