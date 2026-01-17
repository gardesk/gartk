use cairo::Context;
use gartk_core::{Color, Rect};
use std::f64::consts::PI;

/// Set the source color on a Cairo context
pub fn set_color(ctx: &Context, color: Color) {
    ctx.set_source_rgba(color.r, color.g, color.b, color.a);
}

/// Draw a rectangle path (does not fill or stroke)
pub fn rect_path(ctx: &Context, rect: Rect) {
    ctx.rectangle(
        rect.x as f64,
        rect.y as f64,
        rect.width as f64,
        rect.height as f64,
    );
}

/// Draw a filled rectangle
pub fn fill_rect(ctx: &Context, rect: Rect, color: Color) {
    set_color(ctx, color);
    rect_path(ctx, rect);
    let _ = ctx.fill();
}

/// Draw a stroked rectangle
pub fn stroke_rect(ctx: &Context, rect: Rect, color: Color, line_width: f64) {
    set_color(ctx, color);
    ctx.set_line_width(line_width);
    rect_path(ctx, rect);
    let _ = ctx.stroke();
}

/// Draw a rounded rectangle path
pub fn rounded_rect_path(ctx: &Context, rect: Rect, radius: f64) {
    let x = rect.x as f64;
    let y = rect.y as f64;
    let w = rect.width as f64;
    let h = rect.height as f64;

    // Clamp radius to half the smaller dimension
    let r = radius.min(w / 2.0).min(h / 2.0);

    if r <= 0.0 {
        rect_path(ctx, rect);
        return;
    }

    ctx.new_path();
    ctx.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    ctx.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    ctx.arc(x + r, y + h - r, r, PI / 2.0, PI);
    ctx.arc(x + r, y + r, r, PI, 3.0 * PI / 2.0);
    ctx.close_path();
}

/// Draw a filled rounded rectangle
pub fn fill_rounded_rect(ctx: &Context, rect: Rect, radius: f64, color: Color) {
    set_color(ctx, color);
    rounded_rect_path(ctx, rect, radius);
    let _ = ctx.fill();
}

/// Draw a stroked rounded rectangle
pub fn stroke_rounded_rect(
    ctx: &Context,
    rect: Rect,
    radius: f64,
    color: Color,
    line_width: f64,
) {
    set_color(ctx, color);
    ctx.set_line_width(line_width);
    rounded_rect_path(ctx, rect, radius);
    let _ = ctx.stroke();
}

/// Draw a circle path
pub fn circle_path(ctx: &Context, center_x: f64, center_y: f64, radius: f64) {
    ctx.new_path();
    ctx.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
}

/// Draw a filled circle
pub fn fill_circle(ctx: &Context, center_x: f64, center_y: f64, radius: f64, color: Color) {
    set_color(ctx, color);
    circle_path(ctx, center_x, center_y, radius);
    let _ = ctx.fill();
}

/// Draw a stroked circle
pub fn stroke_circle(
    ctx: &Context,
    center_x: f64,
    center_y: f64,
    radius: f64,
    color: Color,
    line_width: f64,
) {
    set_color(ctx, color);
    ctx.set_line_width(line_width);
    circle_path(ctx, center_x, center_y, radius);
    let _ = ctx.stroke();
}

/// Draw a line
pub fn line(ctx: &Context, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, line_width: f64) {
    set_color(ctx, color);
    ctx.set_line_width(line_width);
    ctx.new_path();
    ctx.move_to(x1, y1);
    ctx.line_to(x2, y2);
    let _ = ctx.stroke();
}

/// Draw a horizontal line
pub fn hline(ctx: &Context, y: f64, x1: f64, x2: f64, color: Color, line_width: f64) {
    line(ctx, x1, y, x2, y, color, line_width);
}

/// Draw a vertical line
pub fn vline(ctx: &Context, x: f64, y1: f64, y2: f64, color: Color, line_width: f64) {
    line(ctx, x, y1, x, y2, color, line_width);
}
