# gartk-render

Cairo/Pango rendering with surface management.

## Surface

Cairo ImageSurface wrapper.

```rust
use gartk_render::Surface;
use gartk_core::Color;

let mut surface = Surface::new(800, 600)?;

// Clear
surface.clear(Color::BLACK)?;

// Access Cairo context
let ctx = surface.context()?;
// ... custom Cairo drawing ...

// Resize
surface.resize(1024, 768)?;

// Get raw pixel data (ARGB32 format)
let data: &[u8] = surface.data()?;

// Dimensions
surface.size(); // Size
surface.width();
surface.height();

// Flush pending operations
surface.flush();

// Access underlying Cairo surface
surface.cairo_surface();
```

### DoubleBufferedSurface

For flicker-free rendering (swap buffers pattern).

```rust
use gartk_render::DoubleBufferedSurface;

let mut db = DoubleBufferedSurface::new(800, 600)?;

// Draw to back buffer
let ctx = db.back_context()?;
// ... draw ...

// Swap buffers
db.swap();

// Access front buffer data
let data = db.front_data()?;
```

## Renderer

High-level rendering API combining shapes and text.

```rust
use gartk_render::Renderer;
use gartk_core::{Theme, Color, Rect, Point};

// Create with theme
let theme = Theme::dark();
let renderer = Renderer::with_theme(800, 600, theme)?;

// Or without theme
let renderer = Renderer::new(800, 600)?;

// Clear
renderer.clear()?;  // Uses theme background
renderer.clear_color(Color::RED)?;

// Resize
renderer.resize(1024, 768)?;

// After drawing, flush
renderer.flush();

// Access internals
renderer.surface();
renderer.theme();
renderer.size();
renderer.context()?;  // Cairo context for custom drawing
```

### Shapes

```rust
use gartk_core::{Rect, Color};

let rect = Rect::new(10, 10, 100, 50);

// Rectangles
renderer.fill_rect(rect, Color::BLUE)?;
renderer.stroke_rect(rect, Color::WHITE, 2.0)?;

// Rounded rectangles
renderer.fill_rounded_rect(rect, 8.0, Color::BLUE)?;
renderer.stroke_rounded_rect(rect, 8.0, Color::WHITE, 2.0)?;

// Circles
renderer.fill_circle(50.0, 50.0, 20.0, Color::RED)?;

// Lines
renderer.line(0.0, 0.0, 100.0, 100.0, Color::WHITE, 1.0)?;
```

### Text

```rust
use gartk_render::TextStyle;
use gartk_core::Color;

let style = TextStyle::new()
    .font_family("sans-serif")
    .font_size(14.0)
    .color(Color::WHITE);

// Draw text at position
renderer.text("Hello", 10.0, 20.0, &style)?;

// Draw with theme defaults
renderer.text_default("Hello", 10.0, 20.0, Color::WHITE)?;

// Draw in rectangle (auto-centers vertically)
renderer.text_in_rect("Hello", rect, &style)?;

// Draw centered at point
renderer.text_centered("Hello", Point::new(100, 100), &style)?;

// Measure text
let size = renderer.measure_text("Hello", &style)?;
```

## TextStyle

Text rendering configuration.

```rust
use gartk_render::{TextStyle, TextAlign};
use gartk_core::Color;

let style = TextStyle::new()
    .font_family("JetBrains Mono")
    .font_size(12.0)
    .color(Color::WHITE)
    .align(TextAlign::Left)    // Left, Center, Right
    .ellipsize(true)           // Add "..." when truncated
    .wrap(true)                // Word wrap
    .max_width(200);           // Width constraint (required for ellipsize/wrap)
```

## TextRenderer

Lower-level Pango text rendering (used internally by Renderer).

```rust
use gartk_render::TextRenderer;

let text_renderer = TextRenderer::new();

// Or with default style
let text_renderer = TextRenderer::with_style(style);

// Measure
let size = text_renderer.measure(&ctx, "Hello", &style);

// Draw
text_renderer.draw(&ctx, "Hello", 10.0, 20.0, &style);
text_renderer.draw_in_rect(&ctx, "Hello", rect, &style);
text_renderer.draw_centered(&ctx, "Hello", center_point, &style);
```

## Low-Level Shape Functions

Direct Cairo drawing (requires Cairo context).

```rust
use gartk_render::{fill_rect, stroke_rect, fill_rounded_rect, rounded_rect_path};
use gartk_render::{fill_circle, stroke_circle, circle_path};
use gartk_render::{line, hline, vline, set_color};

let ctx = surface.context()?;

// Color
set_color(&ctx, Color::RED);

// Paths (for custom operations)
rect_path(&ctx, rect);
rounded_rect_path(&ctx, rect, radius);
circle_path(&ctx, cx, cy, radius);

// Fill/stroke
fill_rect(&ctx, rect, color);
stroke_rect(&ctx, rect, color, line_width);
fill_rounded_rect(&ctx, rect, radius, color);
stroke_rounded_rect(&ctx, rect, radius, color, line_width);
fill_circle(&ctx, cx, cy, radius, color);
stroke_circle(&ctx, cx, cy, radius, color, line_width);

// Lines
line(&ctx, x1, y1, x2, y2, color, line_width);
hline(&ctx, x1, x2, y, color, line_width);
vline(&ctx, x, y1, y2, color, line_width);
```

## Font Parsing

Parse font specification strings.

```rust
use gartk_render::parse_font_spec;

// Pango-style
let (family, size) = parse_font_spec("Sans 12")?;
let (family, size) = parse_font_spec("JetBrains Mono 11")?;

// Polybar-style
let (family, size) = parse_font_spec("Monospace:size=14")?;
```

## Re-exports

For direct access to underlying libraries:

```rust
use gartk_render::cairo;
use gartk_render::pango;
use gartk_render::pangocairo;
```
