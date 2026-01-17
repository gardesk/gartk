use crate::color::Color;
use serde::{Deserialize, Serialize};

/// Theme configuration for UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    // Window
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub border_width: u32,
    pub border_radius: f64,

    // Text
    pub font_family: String,
    pub font_size: f64,

    // Selection/highlighting
    pub selection_background: Color,
    pub selection_foreground: Color,

    // Input field
    pub input_background: Color,
    pub input_foreground: Color,
    pub input_border: Color,
    pub input_placeholder: Color,
    pub input_cursor: Color,

    // Items/list
    pub item_background: Color,
    pub item_foreground: Color,
    pub item_hover_background: Color,
    pub item_hover_foreground: Color,
    pub item_selected_background: Color,
    pub item_selected_foreground: Color,
    pub item_description: Color,

    // Scrollbar
    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_width: u32,

    // Spacing
    pub padding: u32,
    pub item_spacing: u32,
    pub item_padding: u32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            // Window
            background: Color::from_hex("#1e1e2e").unwrap(),
            foreground: Color::from_hex("#cdd6f4").unwrap(),
            border: Color::from_hex("#45475a").unwrap(),
            border_width: 2,
            border_radius: 8.0,

            // Text
            font_family: "sans-serif".to_string(),
            font_size: 14.0,

            // Selection
            selection_background: Color::from_hex("#45475a").unwrap(),
            selection_foreground: Color::from_hex("#cdd6f4").unwrap(),

            // Input
            input_background: Color::from_hex("#313244").unwrap(),
            input_foreground: Color::from_hex("#cdd6f4").unwrap(),
            input_border: Color::from_hex("#45475a").unwrap(),
            input_placeholder: Color::from_hex("#6c7086").unwrap(),
            input_cursor: Color::from_hex("#f5e0dc").unwrap(),

            // Items
            item_background: Color::TRANSPARENT,
            item_foreground: Color::from_hex("#cdd6f4").unwrap(),
            item_hover_background: Color::from_hex("#313244").unwrap(),
            item_hover_foreground: Color::from_hex("#cdd6f4").unwrap(),
            item_selected_background: Color::from_hex("#45475a").unwrap(),
            item_selected_foreground: Color::from_hex("#cdd6f4").unwrap(),
            item_description: Color::from_hex("#6c7086").unwrap(),

            // Scrollbar
            scrollbar_track: Color::from_hex("#1e1e2e").unwrap(),
            scrollbar_thumb: Color::from_hex("#45475a").unwrap(),
            scrollbar_width: 8,

            // Spacing
            padding: 12,
            item_spacing: 4,
            item_padding: 8,
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            // Window
            background: Color::from_hex("#eff1f5").unwrap(),
            foreground: Color::from_hex("#4c4f69").unwrap(),
            border: Color::from_hex("#bcc0cc").unwrap(),
            border_width: 2,
            border_radius: 8.0,

            // Text
            font_family: "sans-serif".to_string(),
            font_size: 14.0,

            // Selection
            selection_background: Color::from_hex("#bcc0cc").unwrap(),
            selection_foreground: Color::from_hex("#4c4f69").unwrap(),

            // Input
            input_background: Color::from_hex("#e6e9ef").unwrap(),
            input_foreground: Color::from_hex("#4c4f69").unwrap(),
            input_border: Color::from_hex("#bcc0cc").unwrap(),
            input_placeholder: Color::from_hex("#8c8fa1").unwrap(),
            input_cursor: Color::from_hex("#dc8a78").unwrap(),

            // Items
            item_background: Color::TRANSPARENT,
            item_foreground: Color::from_hex("#4c4f69").unwrap(),
            item_hover_background: Color::from_hex("#e6e9ef").unwrap(),
            item_hover_foreground: Color::from_hex("#4c4f69").unwrap(),
            item_selected_background: Color::from_hex("#bcc0cc").unwrap(),
            item_selected_foreground: Color::from_hex("#4c4f69").unwrap(),
            item_description: Color::from_hex("#8c8fa1").unwrap(),

            // Scrollbar
            scrollbar_track: Color::from_hex("#eff1f5").unwrap(),
            scrollbar_thumb: Color::from_hex("#bcc0cc").unwrap(),
            scrollbar_width: 8,

            // Spacing
            padding: 12,
            item_spacing: 4,
            item_padding: 8,
        }
    }

    /// High contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            // Window
            background: Color::BLACK,
            foreground: Color::WHITE,
            border: Color::WHITE,
            border_width: 2,
            border_radius: 0.0,

            // Text
            font_family: "sans-serif".to_string(),
            font_size: 16.0,

            // Selection
            selection_background: Color::WHITE,
            selection_foreground: Color::BLACK,

            // Input
            input_background: Color::BLACK,
            input_foreground: Color::WHITE,
            input_border: Color::WHITE,
            input_placeholder: Color::GRAY,
            input_cursor: Color::WHITE,

            // Items
            item_background: Color::TRANSPARENT,
            item_foreground: Color::WHITE,
            item_hover_background: Color::from_hex("#333333").unwrap(),
            item_hover_foreground: Color::WHITE,
            item_selected_background: Color::WHITE,
            item_selected_foreground: Color::BLACK,
            item_description: Color::LIGHT_GRAY,

            // Scrollbar
            scrollbar_track: Color::BLACK,
            scrollbar_thumb: Color::WHITE,
            scrollbar_width: 12,

            // Spacing
            padding: 16,
            item_spacing: 8,
            item_padding: 12,
        }
    }

    /// Builder for creating custom themes
    pub fn builder() -> ThemeBuilder {
        ThemeBuilder::new()
    }
}

/// Builder for creating custom themes
#[derive(Debug, Clone)]
pub struct ThemeBuilder {
    theme: Theme,
}

impl ThemeBuilder {
    pub fn new() -> Self {
        Self {
            theme: Theme::dark(),
        }
    }

    pub fn from(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.theme.background = color;
        self
    }

    pub fn foreground(mut self, color: Color) -> Self {
        self.theme.foreground = color;
        self
    }

    pub fn border(mut self, color: Color) -> Self {
        self.theme.border = color;
        self
    }

    pub fn border_width(mut self, width: u32) -> Self {
        self.theme.border_width = width;
        self
    }

    pub fn border_radius(mut self, radius: f64) -> Self {
        self.theme.border_radius = radius;
        self
    }

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.theme.font_family = family.into();
        self
    }

    pub fn font_size(mut self, size: f64) -> Self {
        self.theme.font_size = size;
        self
    }

    pub fn selection_background(mut self, color: Color) -> Self {
        self.theme.selection_background = color;
        self
    }

    pub fn selection_foreground(mut self, color: Color) -> Self {
        self.theme.selection_foreground = color;
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.theme.padding = padding;
        self
    }

    pub fn item_spacing(mut self, spacing: u32) -> Self {
        self.theme.item_spacing = spacing;
        self
    }

    pub fn build(self) -> Theme {
        self.theme
    }
}

impl Default for ThemeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
