# nest-icon

Font Awesome icon fonts and egui widgets for [Nest](../../README.md) desktop apps.

**Crate path:** [`core/crates/nest-icon`](../../core/crates/nest-icon)

## Quick start

```rust
use nest_gui::GuiApp;
use nest_icon::{Icon, IconButton, IconModule};

GuiApp::new("my-app")
    .module(IconModule::new())
    .view(MyView)
    .run();
```

```rust
use nest_icon::{Icon, IconButton, icons};

// Named shortcuts (Font Awesome solid icons)
ui.add(IconButton::new(Icon::PLAY).size(18.0).tooltip("Play"));
ui.add(IconButton::new(Icon::EYE).tooltip("View"));
ui.add(IconButton::new(Icon::PEN_TO_SQUARE).tooltip("Edit"));
ui.add(IconButton::new(Icon::TRASH).tooltip("Delete"));

// Any solid icon by codepoint constant
ui.label(Icon::solid(icons::solid::GEAR).rich_text(20.0));
```

## Icon styles

| Style | Module | Font file |
|-------|--------|-----------|
| Solid | `icons::solid` | `fa-solid-900.ttf` |
| Regular | `icons::regular` | `fa-regular-400.ttf` |
| Brands | `icons::brands` | `fa-brands-400.ttf` |

```rust
Icon::solid(icons::solid::PLAY)
Icon::regular(icons::regular::STAR)
Icon::brands(icons::brands::GITHUB)
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `solid` | yes | Solid icon font (~426 KB) |
| `regular` | yes | Regular icon font (~68 KB) |
| `brands` | yes | Brand icon font (~211 KB) |

Disable unused styles to trim binary size:

```toml
nest-icon = { workspace = true, default-features = false, features = ["solid"] }
```

## License

Nest crate: MIT OR Apache-2.0. Bundled Font Awesome Free 6 fonts: SIL OFL 1.1; icons CC BY 4.0. See [`assets/fonts/LICENSE.txt`](../../core/crates/nest-icon/assets/fonts/LICENSE.txt).

## Related

- [nest-gui](../nest-gui/README.md) — desktop host
- [Font Awesome icons](https://fontawesome.com/icons)
