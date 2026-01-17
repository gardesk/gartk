# gartk Documentation

gartk is a shared X11/Cairo toolkit for the gardesk suite. It provides common functionality used by garlaunch, garbar, gardm-greeter, and other gar components.

## Crates

| Crate | Purpose |
|-------|---------|
| `gartk-core` | Fundamental types: Color, Rect, Theme, InputEvent |
| `gartk-x11` | X11 connection, window management, monitors, event loop |
| `gartk-render` | Cairo/Pango rendering: surfaces, shapes, text |

## Quick Start

```toml
[dependencies]
gartk-core = { path = "../gartk/gartk-core" }
gartk-x11 = { path = "../gartk/gartk-x11" }
gartk-render = { path = "../gartk/gartk-render" }
```

### Minimal Window

```rust
use gartk_core::Theme;
use gartk_x11::{Connection, Window, WindowConfig, EventLoop};
use gartk_render::Renderer;

fn main() -> anyhow::Result<()> {
    let conn = Connection::connect(None)?;
    let window = Window::create(
        conn.clone(),
        WindowConfig::default()
            .title("Example")
            .size(400, 300),
    )?;

    window.show()?;
    window.focus()?;

    let theme = Theme::dark();
    let renderer = Renderer::with_theme(400, 300, theme)?;
    renderer.clear()?;
    renderer.flush();

    let event_loop = EventLoop::new(conn, Default::default());
    event_loop.run(|event| {
        // Handle events
        Ok(true) // Return false to exit
    })?;

    Ok(())
}
```

## Documentation

- [gartk-core](core.md) - Types and theming
- [gartk-x11](x11.md) - X11 integration
- [gartk-render](render.md) - Cairo/Pango rendering

## Usage in gardesk

gartk is used by:

- **garlaunch**: Popup window, text rendering, keyboard input
- **garbar**: Status bar rendering (shares patterns, not yet migrated)
- **gardm-greeter**: Login UI (shares patterns, not yet migrated)

The toolkit extracts common patterns from these components. Future gar components can use gartk directly.
