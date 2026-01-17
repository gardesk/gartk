use serde::{Deserialize, Serialize};

/// A 2D point
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const ORIGIN: Self = Self::new(0, 0);
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

/// A 2D size
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn area(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub const ZERO: Self = Self::new(0, 0);
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Self::new(width, height)
    }
}

/// A rectangle with position and size
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_point_size(point: Point, size: Size) -> Self {
        Self::new(point.x, point.y, size.width, size.height)
    }

    pub const fn position(self) -> Point {
        Point::new(self.x, self.y)
    }

    pub const fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    pub const fn right(self) -> i32 {
        self.x + self.width as i32
    }

    pub const fn bottom(self) -> i32 {
        self.y + self.height as i32
    }

    pub fn center(self) -> Point {
        Point::new(
            self.x + (self.width / 2) as i32,
            self.y + (self.height / 2) as i32,
        )
    }

    pub fn contains_point(self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.right()
            && point.y >= self.y
            && point.y < self.bottom()
    }

    pub fn intersects(self, other: Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    pub fn intersection(self, other: Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if x < right && y < bottom {
            Some(Rect::new(x, y, (right - x) as u32, (bottom - y) as u32))
        } else {
            None
        }
    }

    /// Inset the rectangle by the given amounts
    pub fn inset(self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Self::new(
            self.x + left,
            self.y + top,
            (self.width as i32 - left - right).max(0) as u32,
            (self.height as i32 - top - bottom).max(0) as u32,
        )
    }

    /// Inset uniformly on all sides
    pub fn inset_uniform(self, amount: i32) -> Self {
        self.inset(amount, amount, amount, amount)
    }

    pub const ZERO: Self = Self::new(0, 0, 0, 0);
}

/// Padding/margin specification
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Edges {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Edges {
    pub const fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn uniform(value: i32) -> Self {
        Self::new(value, value, value, value)
    }

    pub const fn symmetric(vertical: i32, horizontal: i32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    pub const fn horizontal(self) -> i32 {
        self.left + self.right
    }

    pub const fn vertical(self) -> i32 {
        self.top + self.bottom
    }

    pub const ZERO: Self = Self::new(0, 0, 0, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 100, 100);
        assert!(rect.contains_point(Point::new(50, 50)));
        assert!(rect.contains_point(Point::new(10, 10)));
        assert!(!rect.contains_point(Point::new(110, 50)));
        assert!(!rect.contains_point(Point::new(5, 50)));
    }

    #[test]
    fn test_rect_intersection() {
        let a = Rect::new(0, 0, 100, 100);
        let b = Rect::new(50, 50, 100, 100);
        let c = Rect::new(200, 200, 50, 50);

        assert!(a.intersects(b));
        assert!(!a.intersects(c));

        let intersection = a.intersection(b).unwrap();
        assert_eq!(intersection, Rect::new(50, 50, 50, 50));
    }

    #[test]
    fn test_rect_inset() {
        let rect = Rect::new(0, 0, 100, 100);
        let inset = rect.inset_uniform(10);
        assert_eq!(inset, Rect::new(10, 10, 80, 80));
    }
}
