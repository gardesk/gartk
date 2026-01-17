# gartk-core

Fundamental types used throughout gartk.

## Color

RGBA color representation with parsing utilities.

```rust
use gartk_core::Color;

// Constructors
let c = Color::new(255, 128, 64, 255);
let c = Color::from_rgb(255, 128, 64);
let c = Color::from_hex("#ff8040").unwrap();
let c = Color::from_hex("#ff804080").unwrap(); // with alpha

// Named colors
let c = Color::WHITE;
let c = Color::BLACK;
let c = Color::TRANSPARENT;
let c = Color::RED;
let c = Color::GRAY;

// Methods
let (r, g, b, a) = c.to_rgba();
let (r, g, b, a) = c.to_rgba_f64(); // 0.0-1.0 range
let hex = c.to_hex();
let lighter = c.lighten(0.1);
let darker = c.darken(0.1);
let faded = c.with_alpha(128);
```

## Geometry

### Point

```rust
use gartk_core::Point;

let p = Point::new(10, 20);
let p = Point::origin(); // (0, 0)
```

### Size

```rust
use gartk_core::Size;

let s = Size::new(800, 600);
let area = s.area();
let is_empty = s.is_empty();
```

### Rect

```rust
use gartk_core::Rect;

let r = Rect::new(10, 20, 100, 50); // x, y, width, height
let r = Rect::from_points(Point::new(10, 20), Point::new(110, 70));

// Properties
r.x; r.y; r.width; r.height;
r.left(); r.right(); r.top(); r.bottom();
r.center();
r.size();
r.origin();

// Operations
r.contains(Point::new(50, 40));
r.intersects(&other);
r.intersection(&other);
r.union(&other);
r.inset(5); // shrink by 5 on all sides
r.offset(10, 20); // move
```

### Edges

Padding/margin representation.

```rust
use gartk_core::Edges;

let e = Edges::all(10);
let e = Edges::symmetric(10, 5); // horizontal, vertical
let e = Edges::new(5, 10, 5, 10); // top, right, bottom, left
```

## InputEvent

Abstraction over X11 input events.

```rust
use gartk_core::{InputEvent, Key, Modifiers};

match event {
    InputEvent::Key(key_event) => {
        if key_event.pressed {
            match key_event.key {
                Key::Return => { /* enter pressed */ }
                Key::Escape => { /* escape pressed */ }
                Key::Char(c) => { /* character input */ }
                _ => {}
            }
        }
        // Check modifiers
        if key_event.modifiers.ctrl { /* ctrl held */ }
        if key_event.modifiers.shift { /* shift held */ }
    }
    InputEvent::Mouse(mouse_event) => {
        // mouse_event.x, mouse_event.y, mouse_event.button
    }
    InputEvent::Scroll(scroll_event) => {
        // scroll_event.delta_x, scroll_event.delta_y
    }
    InputEvent::FocusIn | InputEvent::FocusOut => {}
    InputEvent::Close => { /* window close requested */ }
}
```

### Key Enum

Common keys: `Escape`, `Return`, `Tab`, `Backspace`, `Delete`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`, `PageDown`, `Space`, `Char(char)`, `F1`-`F12`, `Unknown(u8)`.

## Theme

UI theming with presets.

```rust
use gartk_core::Theme;

// Presets
let theme = Theme::dark();
let theme = Theme::light();
let theme = Theme::high_contrast();

// Access properties
theme.background;
theme.foreground;
theme.border;
theme.border_width;
theme.border_radius;
theme.font_family;
theme.font_size;
theme.padding;
theme.item_padding;
theme.item_spacing;

// Input field colors
theme.input_background;
theme.input_foreground;
theme.input_border;
theme.input_placeholder;
theme.input_cursor;

// Item/list colors
theme.item_background;
theme.item_foreground;
theme.item_selected_background;
theme.item_selected_foreground;
theme.item_hover_background;
theme.item_hover_foreground;
theme.item_description;
```

### ThemeBuilder

```rust
use gartk_core::{Theme, ThemeBuilder, Color};

let theme = Theme::builder()
    .background(Color::from_hex("#1a1a1a").unwrap())
    .foreground(Color::WHITE)
    .font_size(16.0)
    .padding(16)
    .build();

// Or modify existing
let theme = ThemeBuilder::from(Theme::dark())
    .font_family("JetBrains Mono")
    .build();
```
