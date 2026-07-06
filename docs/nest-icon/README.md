# nest-icon (legacy)

**Do not use in new desktop apps.** Icons belong in the React `ui/` layer.

## Replacement

| Legacy | Use instead |
|--------|-------------|
| `nest-icon` / `IconModule` | [Lucide React](https://lucide.dev) or [Font Awesome React](https://fontawesome.com/docs/web/use-with/react) |
| `IconButton`, `Icon::rich_text` | React `<button>` + icon component + Tailwind |

Size and color via Tailwind + [`nest-react-theme`](../nest-react-theme/README.md) tokens (e.g. `size-4 text-nest-muted-foreground`).

Full migration plan: [nest-react-ui v1](../plan/nest-react-ui-v1.md).

Nest desktop apps use **Tauri + React + TypeScript + Tailwind**. See [nest-tauri](../nest-tauri/README.md).

The `nest-icon` crate may remain in the workspace temporarily for Kiwi migration only.
