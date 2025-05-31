//! Geometric primitives like points, sizes, and rectangles.

use num_traits::{Float, Num, Signed, Zero};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

// --- Generic Point<T> ---

/// Represents a 2D point with generic coordinates.
///
/// # Type Parameters
///
/// * `T`: The numeric type for the coordinates (e.g., `i32`, `f32`).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize + Num + Copy",
    deserialize = "T: Deserialize<'de> + Num + Copy"
))]
pub struct Point<T: Num + Copy> {
    /// The x-coordinate of the point.
    pub x: T,
    /// The y-coordinate of the point.
    pub y: T,
}

// Implement Eq and Hash if T supports them
impl<T: Num + Copy + Eq> Eq for Point<T> {}
impl<T: Num + Copy + std::hash::Hash> std::hash::Hash for Point<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl<T: Num + Copy> Point<T> {
    /// Creates a new point with the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x`: The x-coordinate.
    /// * `y`: The y-coordinate.
    pub const fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    /// Calculates the squared Euclidean distance to another point.
    /// This is often preferred over `distance` to avoid a square root calculation
    /// if only comparing distances.
    ///
    /// Requires `T` to support subtraction and multiplication, resulting in `T`.
    pub fn distance_squared(&self, other: &Self) -> T
    where
        T: Sub<Output = T> + Add<Output = T> + Mul<Output = T>,
    {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl<T: Num + Copy + Float> Point<T> {
    /// Calculates the Euclidean distance to another point.
    ///
    /// Requires `T` to be a floating-point type (`Float`).
    pub fn distance(&self, other: &Self) -> T {
        self.distance_squared(other).sqrt()
    }
}

impl<T: Num + Copy + Signed> Point<T> {
    /// Calculates the Manhattan distance (L1 norm) to another point.
    ///
    /// Requires `T` to be a signed numeric type (`Signed`).
    pub fn manhattan_distance(&self, other: &Self) -> T {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl<T: Num + Copy + Zero> Point<T> {
    /// A point at the origin (0,0) for integer types.
    pub const ZERO_I32: Point<i32> = Point::new(0, 0);
    pub const ZERO_U32: Point<u32> = Point::new(0, 0);
    /// A point at the origin (0.0, 0.0) for f32.
    pub const ZERO_F32: Point<f32> = Point::new(0.0, 0.0);
    /// A point at the origin (0.0, 0.0) for f64.
    pub const ZERO_F64: Point<f64> = Point::new(0.0, 0.0);
}

impl<T: Num + Copy + Add<Output = T>> Add for Point<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Num + Copy + Sub<Output = T>> Sub for Point<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// --- Generic Size<T> ---

/// Represents a 2D size (width and height) with generic dimensions.
///
/// # Type Parameters
///
/// * `T`: The numeric type for the dimensions (e.g., `u32`, `f32`).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize + Num + Copy",
    deserialize = "T: Deserialize<'de> + Num + Copy"
))]
pub struct Size<T: Num + Copy> {
    /// The width component of the size.
    pub width: T,
    /// The height component of the size.
    pub height: T,
}

// Implement Eq and Hash if T supports them
impl<T: Num + Copy + Eq> Eq for Size<T> {}
impl<T: Num + Copy + std::hash::Hash> std::hash::Hash for Size<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl<T: Num + Copy> Size<T> {
    /// Creates a new size with the given width and height.
    pub const fn new(width: T, height: T) -> Self {
        Size { width, height }
    }

    /// Calculates the area of the size (width * height).
    ///
    /// Requires `T` to support multiplication.
    pub fn area(&self) -> T
    where
        T: Mul<Output = T>,
    {
        self.width * self.height
    }

    /// Checks if the area is zero (width or height is zero).
    ///
    /// Requires `T` to support `Zero`.
    pub fn is_empty(&self) -> bool
    where
        T: Zero + PartialEq,
    {
        self.width.is_zero() || self.height.is_zero()
    }

    /// Checks if the width and height are non-negative.
    /// For unsigned types, this always returns true.
    /// For signed types, it checks if width >= 0 and height >= 0.
    ///
    /// Requires `T` to support `PartialOrd<T::zero()>`.
    pub fn is_valid(&self) -> bool
    where
        T: PartialOrd + Zero,
    {
        self.width >= T::zero() && self.height >= T::zero()
    }
}

impl<T: Num + Copy + Zero> Size<T> {
    /// A size of (0,0) for integer types.
    pub const ZERO_I32: Size<i32> = Size::new(0, 0);
    pub const ZERO_U32: Size<u32> = Size::new(0, 0);
    /// A size of (0.0,0.0) for f32.
    pub const ZERO_F32: Size<f32> = Size::new(0.0, 0.0);
    /// A size of (0.0,0.0) for f64.
    pub const ZERO_F64: Size<f64> = Size::new(0.0, 0.0);
}

// --- Generic Rect<T> ---

/// Represents a 2D rectangle defined by an origin point and a size.
///
/// # Type Parameters
///
/// * `T`: The numeric type for the coordinates and dimensions (e.g., `i32`, `f32`).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize + Num + Copy",
    deserialize = "T: Deserialize<'de> + Num + Copy"
))]
pub struct Rect<T: Num + Copy> {
    /// The origin point (top-left corner) of the rectangle.
    pub origin: Point<T>,
    /// The size (width and height) of the rectangle.
    pub size: Size<T>,
}

// Implement Eq and Hash if T supports them
impl<T: Num + Copy + Eq> Eq for Rect<T> {}
impl<T: Num + Copy + std::hash::Hash> std::hash::Hash for Rect<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.origin.hash(state);
        self.size.hash(state);
    }
}

impl<T: Num + Copy> Rect<T> {
    /// Creates a new rectangle from an origin point and a size.
    pub const fn new(origin: Point<T>, size: Size<T>) -> Self {
        Rect { origin, size }
    }

    /// Creates a new rectangle from individual coordinate and dimension values.
    pub const fn from_coords(x: T, y: T, width: T, height: T) -> Self {
        Rect {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// Returns the x-coordinate of the rectangle's origin (left edge).
    pub fn x(&self) -> T {
        self.origin.x
    }

    /// Returns the y-coordinate of the rectangle's origin (top edge).
    pub fn y(&self) -> T {
        self.origin.y
    }

    /// Returns the width of the rectangle.
    pub fn width(&self) -> T {
        self.size.width
    }

    /// Returns the height of the rectangle.
    pub fn height(&self) -> T {
        self.size.height
    }

    /// Returns the y-coordinate of the top edge. (Same as `y()`)
    pub fn top(&self) -> T {
        self.origin.y
    }

    /// Returns the x-coordinate of the left edge. (Same as `x()`)
    pub fn left(&self) -> T {
        self.origin.x
    }

    /// Calculates the x-coordinate of the right edge. (origin.x + size.width)
    ///
    /// Requires `T` to support addition.
    pub fn right(&self) -> T
    where
        T: Add<Output = T>,
    {
        self.origin.x + self.size.width
    }

    /// Calculates the y-coordinate of the bottom edge. (origin.y + size.height)
    ///
    /// Requires `T` to support addition.
    pub fn bottom(&self) -> T
    where
        T: Add<Output = T>,
    {
        self.origin.y + self.size.height
    }

    /// Calculates the center point of the rectangle.
    ///
    /// Requires `T` to support addition and division by 2 (often via `From<i16>`).
    /// For integer types, this might truncate.
    pub fn center(&self) -> Point<T>
    where
        T: Add<Output = T> + Sub<Output = T> + From<i16> + Div<Output = T>, // Using From<i16> for 2
    {
        let two = T::from(2);
        Point::new(
            self.origin.x + self.size.width / two,
            self.origin.y + self.size.height / two,
        )
    }

    /// Checks if a point is contained within the rectangle (inclusive of edges).
    /// Assumes standard coordinate system (y increases downwards).
    ///
    /// Requires `T` to support `PartialOrd` and addition.
    pub fn contains_point(&self, point: &Point<T>) -> bool
    where
        T: PartialOrd + Add<Output = T> + Sub<Output = T>, // Sub for right/bottom calculation if not using self.right/bottom
    {
        point.x >= self.left()
            && point.x < self.right()
            && point.y >= self.top()
            && point.y < self.bottom()
    }

    /// Checks if this rectangle intersects with another rectangle.
    ///
    /// Requires `T` to support `PartialOrd` and addition.
    pub fn intersects(&self, other: &Rect<T>) -> bool
    where
        T: PartialOrd + Add<Output = T> + Sub<Output = T>,
    {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Calculates the intersection of this rectangle with another.
    /// Returns `None` if they do not intersect.
    ///
    /// Requires `T` to support `PartialOrd`, `Add`, `Sub`, and `Ord` (for min/max like behavior).
    /// This implementation uses manual min/max logic for broader `T` compatibility.
    pub fn intersection(&self, other: &Rect<T>) -> Option<Rect<T>>
    where
        T: PartialOrd + Add<Output = T> + Sub<Output = T>, // Ord required for min/max
    {
        let x1 = if self.left() > other.left() { self.left() } else { other.left() };
        let y1 = if self.top() > other.top() { self.top() } else { other.top() };
        let x2 = if self.right() < other.right() { self.right() } else { other.right() };
        let y2 = if self.bottom() < other.bottom() { self.bottom() } else { other.bottom() };

        if x1 < x2 && y1 < y2 {
            Some(Rect::from_coords(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    /// Calculates the smallest rectangle that contains both this and another rectangle (bounding box).
    ///
    /// Requires `T` to support `PartialOrd`, `Add`, and `Sub`.
    pub fn union(&self, other: &Rect<T>) -> Rect<T>
    where
        T: PartialOrd + Add<Output = T> + Sub<Output = T>,
    {
        let x1 = if self.left() < other.left() { self.left() } else { other.left() };
        let y1 = if self.top() < other.top() { self.top() } else { other.top() };
        let x2 = if self.right() > other.right() { self.right() } else { other.right() };
        let y2 = if self.bottom() > other.bottom() { self.bottom() } else { other.bottom() };

        Rect::from_coords(x1, y1, x2 - x1, y2 - y1)
    }

    /// Returns a new rectangle translated by a given delta point.
    ///
    /// Requires `T` to support addition.
    pub fn translated(&self, delta: &Point<T>) -> Rect<T>
    where
        T: Add<Output = T>,
    {
        Rect::new(self.origin + *delta, self.size)
    }

    /// Returns a new rectangle scaled by a given factor (applied to size).
    /// Origin remains the same.
    ///
    /// Requires `T` to support multiplication.
    pub fn scaled(&self, factor_x: T, factor_y: T) -> Rect<T>
    where
        T: Mul<Output = T>,
    {
        Rect::new(
            self.origin,
            Size::new(self.size.width * factor_x, self.size.height * factor_y),
        )
    }
    
    /// Checks if the rectangle's size is valid (non-negative width and height).
    ///
    /// Requires `T` to support `PartialOrd<T::zero()>`.
    pub fn is_valid(&self) -> bool
        where T: PartialOrd + Zero
    {
        self.size.is_valid()
    }
}

impl<T: Num + Copy + Zero> Rect<T> {
    /// A rectangle at (0,0) with size (0,0) for integer types.
    pub const ZERO_I32: Rect<i32> = Rect::from_coords(0, 0, 0, 0);
    pub const ZERO_U32: Rect<u32> = Rect::from_coords(0, 0, 0, 0);
    /// A rectangle at (0.0,0.0) with size (0.0,0.0) for f32.
    pub const ZERO_F32: Rect<f32> = Rect::from_coords(0.0, 0.0, 0.0, 0.0);
    /// A rectangle at (0.0,0.0) with size (0.0,0.0) for f64.
    pub const ZERO_F64: Rect<f64> = Rect::from_coords(0.0, 0.0, 0.0, 0.0);
}

// --- Integer-specific PointInt, SizeInt, RectInt (as per A4 Kernschicht) ---

/// An integer point with `i32` coordinates.
/// As defined in "A4 Kernschicht", Section 6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct PointInt {
    pub x: i32,
    pub y: i32,
}

impl PointInt {
    /// Creates a new `PointInt`.
    pub const fn new(x: i32, y: i32) -> Self {
        PointInt { x, y }
    }
}

/// An integer size with `u32` dimensions.
/// As defined in "A4 Kernschicht", Section 6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SizeInt {
    pub width: u32,
    pub height: u32,
}

impl SizeInt {
    /// Creates a new `SizeInt`.
    pub const fn new(width: u32, height: u32) -> Self {
        SizeInt { width, height }
    }

    /// Checks if the area is zero.
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// An integer rectangle with `i32` origin and `u32` size.
/// As defined in "A4 Kernschicht", Section 6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct RectInt {
    /// The origin point (top-left corner) of the rectangle.
    pub origin: PointInt,
    /// The size (width and height) of the rectangle.
    pub size: SizeInt,
}

impl RectInt {
    /// Creates a new `RectInt` from an origin point and a size.
    pub const fn new(origin: PointInt, size: SizeInt) -> Self {
        RectInt { origin, size }
    }

    /// Creates a new `RectInt` from individual coordinate and dimension values.
    pub const fn from_coords(x: i32, y: i32, width: u32, height: u32) -> Self {
        RectInt {
            origin: PointInt::new(x, y),
            size: SizeInt::new(width, height),
        }
    }
    
    /// Returns the x-coordinate of the rectangle's origin.
    pub fn x(&self) -> i32 { self.origin.x }
    /// Returns the y-coordinate of the rectangle's origin.
    pub fn y(&self) -> i32 { self.origin.y }
    /// Returns the width of the rectangle.
    pub fn width(&self) -> u32 { self.size.width }
    /// Returns the height of the rectangle.
    pub fn height(&self) -> u32 { self.size.height }

    /// Returns the y-coordinate of the top edge.
    pub fn top(&self) -> i32 { self.origin.y }
    /// Returns the x-coordinate of the left edge.
    pub fn left(&self) -> i32 { self.origin.x }

    /// Calculates the x-coordinate of the right edge.
    pub fn right(&self) -> i32 {
        self.origin.x + self.size.width as i32 // Potentially lossy if width is large, but consistent with i32 rects
    }
    /// Calculates the y-coordinate of the bottom edge.
    pub fn bottom(&self) -> i32 {
        self.origin.y + self.size.height as i32 // Potentially lossy if height is large
    }
    
    /// Checks if a point (`PointInt`) is contained within the rectangle.
    /// Edges are inclusive for left/top, exclusive for right/bottom.
    pub fn contains_point(&self, point: PointInt) -> bool {
        point.x >= self.left() && point.x < self.right() &&
        point.y >= self.top() && point.y < self.bottom()
    }

    /// Checks if this rectangle intersects with another `RectInt`.
    pub fn intersects(&self, other: &RectInt) -> bool {
        self.left() < other.right() && self.right() > other.left() &&
        self.top() < other.bottom() && self.bottom() > other.top()
    }

    /// Calculates the intersection of this rectangle with another `RectInt`.
    /// Returns `None` if they do not intersect.
    pub fn intersection(&self, other: &RectInt) -> Option<RectInt> {
        let x1 = self.left().max(other.left());
        let y1 = self.top().max(other.top());
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x1 < x2 && y1 < y2 {
            Some(RectInt::from_coords(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32))
        } else {
            None
        }
    }
    
    /// Calculates the smallest rectangle that contains both this and another `RectInt`.
    pub fn union(&self, other: &RectInt) -> RectInt {
        let x1 = self.left().min(other.left());
        let y1 = self.top().min(other.top());
        let x2 = self.right().max(other.right());
        let y2 = self.bottom().max(other.bottom());
        
        RectInt::from_coords(x1, y1, (x2-x1) as u32, (y2-y1) as u32)
    }

    /// Checks if the rectangle has zero width or height.
    pub fn is_empty(&self) -> bool {
        self.size.is_empty()
    }

    /// Creates a `RectInt` from two points, ensuring positive width and height.
    /// The rectangle will encompass both points.
    pub fn from_points(p1: PointInt, p2: PointInt) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p1.x - p2.x).abs() as u32;
        let height = (p1.y - p2.y).abs() as u32;
        RectInt::from_coords(x, y, width, height)
    }

    /// Translates the rectangle by a given delta (dx, dy).
    /// The origin is moved by (dx, dy), size remains the same.
    /// Uses saturating arithmetic to prevent overflow.
    pub fn translate(&self, dx: i32, dy: i32) -> Self {
        RectInt::from_coords(
            self.origin.x.saturating_add(dx),
            self.origin.y.saturating_add(dy),
            self.size.width,
            self.size.height,
        )
    }

    /// Inflates the rectangle by a given delta (dw, dh) from its center.
    /// `dw` is added to each side for width (total width change is 2*dw).
    /// `dh` is added to each side for height (total height change is 2*dh).
    /// The origin is shifted by (-dw, -dh).
    /// Width and height will not go below zero.
    /// Uses saturating arithmetic.
    pub fn inflate(&self, dw: i32, dh: i32) -> Self {
        let new_x = self.origin.x.saturating_sub(dw);
        let new_y = self.origin.y.saturating_sub(dh);

        // Calculate new width and height with i64 to avoid overflow before max(0)
        let new_width_signed = (self.size.width as i64).saturating_add(2 * dw as i64);
        let new_height_signed = (self.size.height as i64).saturating_add(2 * dh as i64);

        RectInt::from_coords(
            new_x,
            new_y,
            new_width_signed.max(0) as u32,
            new_height_signed.max(0) as u32,
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;
    use std::fmt; // Required for fmt::Debug
    use std::hash::Hash; // Required for std::hash::Hash

    // --- Type Assertions ---
    assert_impl_all!(Point<i32>: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);
    assert_impl_all!(Point<f32>: std::fmt::Debug, Clone, Copy, PartialEq, Default, Serialize, Send, Sync); // No Eq/Hash for f32
    assert_impl_all!(Size<u32>: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);
    assert_impl_all!(Rect<i32>: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);
    
    assert_impl_all!(PointInt: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);
    assert_impl_all!(SizeInt: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);
    assert_impl_all!(RectInt: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);

    // Additional Send/Sync assertions as per plan
    assert_impl_all!(Point<i32>: Send, Sync);
    assert_impl_all!(Size<u32>: Send, Sync); // Note: Size<u32> was already asserted with Send, Sync above. This is redundant but harmless.
    assert_impl_all!(Rect<i32>: Send, Sync); // Note: Rect<i32> was already asserted with Send, Sync above. This is redundant but harmless.
    assert_impl_all!(PointInt: Send, Sync);  // Note: PointInt was already asserted with Send, Sync above. This is redundant but harmless.
    // SizeInt Send, Sync is already covered by the line:
    // assert_impl_all!(SizeInt: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);
    // RectInt Send, Sync is already covered by the line:
    // assert_impl_all!(RectInt: std::fmt::Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Default, Serialize, Send, Sync);


    // --- Point<T> Tests ---
    #[test]
    fn point_new_and_coordinates() {
        let p_i32 = Point::new(10, 20);
        assert_eq!(p_i32.x, 10);
        assert_eq!(p_i32.y, 20);

        let p_f32 = Point::new(10.5, 20.5);
        assert_eq!(p_f32.x, 10.5);
        assert_eq!(p_f32.y, 20.5);
    }

    #[test]
    fn rect_int_from_points() {
        let p1 = PointInt::new(10, 20);
        let p2 = PointInt::new(0, 5);
        let r = RectInt::from_points(p1, p2);
        assert_eq!(r.x(), 0);
        assert_eq!(r.y(), 5);
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 15);

        let r_same_points = RectInt::from_points(p1, p1);
        assert_eq!(r_same_points.x(), p1.x);
        assert_eq!(r_same_points.y(), p1.y);
        assert_eq!(r_same_points.width(), 0);
        assert_eq!(r_same_points.height(), 0);
    }

    #[test]
    fn rect_int_translate() {
        let r = RectInt::from_coords(10, 20, 30, 40);
        let translated = r.translate(5, -5);
        assert_eq!(translated.x(), 15);
        assert_eq!(translated.y(), 15);
        assert_eq!(translated.width(), 30);
        assert_eq!(translated.height(), 40);

        // Test saturating_add
        let r_max = RectInt::from_coords(i32::MAX - 5, i32::MAX - 5, 10, 10);
        let translated_max = r_max.translate(10, 10);
        assert_eq!(translated_max.x(), i32::MAX);
        assert_eq!(translated_max.y(), i32::MAX);

        let r_min = RectInt::from_coords(i32::MIN + 5, i32::MIN + 5, 10, 10);
        let translated_min = r_min.translate(-10, -10);
        assert_eq!(translated_min.x(), i32::MIN);
        assert_eq!(translated_min.y(), i32::MIN);
    }

    #[test]
    fn rect_int_inflate() {
        let r = RectInt::from_coords(10, 20, 30, 40);

        // Positive inflation
        let inflated_positive = r.inflate(5, 10); // dw=5, dh=10
        assert_eq!(inflated_positive.x(), 10 - 5);     // x - dw
        assert_eq!(inflated_positive.y(), 20 - 10);    // y - dh
        assert_eq!(inflated_positive.width(), 30 + 2*5); // width + 2*dw
        assert_eq!(inflated_positive.height(), 40 + 2*10);// height + 2*dh

        // Negative inflation (shrinking)
        let inflated_negative = r.inflate(-5, -10);
        assert_eq!(inflated_negative.x(), 10 - (-5));
        assert_eq!(inflated_negative.y(), 20 - (-10));
        assert_eq!(inflated_negative.width(), 30 + 2*(-5));
        assert_eq!(inflated_negative.height(), 40 + 2*(-10));

        // Inflation resulting in zero width/height
        let inflated_to_zero_width = r.inflate(-15, 5); // width becomes 30 - 30 = 0
        assert_eq!(inflated_to_zero_width.width(), 0);
        assert_eq!(inflated_to_zero_width.x(), 10 - (-15));

        let inflated_to_zero_height = r.inflate(5, -20); // height becomes 40 - 40 = 0
        assert_eq!(inflated_to_zero_height.height(), 0);
        assert_eq!(inflated_to_zero_height.y(), 20 - (-20));

        // Inflation that would result in negative width/height (should be clamped to 0)
        let inflated_past_zero = r.inflate(-20, -25); // width would be 30-40 = -10, height 40-50 = -10
        assert_eq!(inflated_past_zero.width(), 0);
        assert_eq!(inflated_past_zero.height(), 0);
        assert_eq!(inflated_past_zero.x(), 10 - (-20));
        assert_eq!(inflated_past_zero.y(), 20 - (-25));

        // Test saturation at origin shift
        let r_at_min = RectInt::from_coords(i32::MIN, i32::MIN, 100, 100);
        let inflated_at_min = r_at_min.inflate(10, 10); // x = MIN - 10 (saturates to MIN)
        assert_eq!(inflated_at_min.x(), i32::MIN);
        assert_eq!(inflated_at_min.y(), i32::MIN);
        assert_eq!(inflated_at_min.width(), 100 + 2*10);
        assert_eq!(inflated_at_min.height(), 100 + 2*10);

        let r_near_max_size = RectInt::from_coords(0, 0, u32::MAX - 10, u32::MAX - 10);
        let inflated_max_size = r_near_max_size.inflate(10, 10); // width/height would exceed u32::MAX if not for i64 intermediate
        assert_eq!(inflated_max_size.width(), u32::MAX); // width = MAX-10 + 20 -> MAX+10, clamped to MAX
        assert_eq!(inflated_max_size.height(), u32::MAX); // height = MAX-10 + 20 -> MAX+10, clamped to MAX
    }

    #[test]
    fn point_default() {
        let p_i32: Point<i32> = Default::default();
        assert_eq!(p_i32, Point::new(0, 0));
        let p_f32: Point<f32> = Default::default();
        assert_eq!(p_f32, Point::new(0.0, 0.0));
    }
    
    #[test]
    fn point_constants() {
        assert_eq!(Point::<i32>::ZERO_I32, Point::new(0,0));
        assert_eq!(Point::<u32>::ZERO_U32, Point::new(0u32,0u32));
        assert_eq!(Point::<f32>::ZERO_F32, Point::new(0.0f32,0.0f32));
    }

    #[test]
    fn point_distance_squared() {
        let p1 = Point::new(1, 2);
        let p2 = Point::new(4, 6);
        assert_eq!(p1.distance_squared(&p2), (4-1)*(4-1) + (6-2)*(6-2)); // 3*3 + 4*4 = 9 + 16 = 25

        let p1_f = Point::new(1.0, 2.0);
        let p2_f = Point::new(4.0, 6.0);
        assert_eq!(p1_f.distance_squared(&p2_f), 25.0);
    }

    #[test]
    fn point_distance_float() {
        let p1_f = Point::new(1.0, 2.0);
        let p2_f = Point::new(4.0, 6.0);
        assert_eq!(p1_f.distance(&p2_f), 5.0);
    }

    #[test]
    fn point_manhattan_distance_signed() {
        let p1 = Point::new(1, 2);
        let p2 = Point::new(4, -2);
        assert_eq!(p1.manhattan_distance(&p2), (4-1).abs() + (-2-2).abs()); // 3 + 4 = 7
    }
    
    #[test]
    fn point_ops() {
        let p1 = Point::new(1, 2);
        let p2 = Point::new(3, 4);
        assert_eq!(p1 + p2, Point::new(4, 6));
        assert_eq!(p2 - p1, Point::new(2, 2));
    }

    // --- Size<T> Tests ---
    #[test]
    fn size_new_and_dimensions() {
        let s_u32 = Size::new(100, 200);
        assert_eq!(s_u32.width, 100);
        assert_eq!(s_u32.height, 200);

        let s_f32 = Size::new(10.5, 20.5);
        assert_eq!(s_f32.width, 10.5);
        assert_eq!(s_f32.height, 20.5);
    }

    #[test]
    fn size_default() {
        let s_u32: Size<u32> = Default::default();
        assert_eq!(s_u32, Size::new(0,0));
    }

    #[test]
    fn size_constants() {
        assert_eq!(Size::<i32>::ZERO_I32, Size::new(0,0));
        assert_eq!(Size::<u32>::ZERO_U32, Size::new(0u32,0u32));
        assert_eq!(Size::<f32>::ZERO_F32, Size::new(0.0f32,0.0f32));
    }

    #[test]
    fn size_area() {
        let s = Size::new(10, 5);
        assert_eq!(s.area(), 50);
        let s_f = Size::new(10.0, 0.5);
        assert_eq!(s_f.area(), 5.0);
    }

    #[test]
    fn size_is_empty() {
        assert!(Size::new(0, 10).is_empty());
        assert!(Size::new(10, 0).is_empty());
        assert!(Size::new(0, 0).is_empty());
        assert!(!Size::new(10, 10).is_empty());
    }

    #[test]
    fn size_is_valid() {
        assert!(Size::new(10, 10).is_valid());
        assert!(Size::new(0, 0).is_valid());
        assert!(Size::new(10u32, 10u32).is_valid()); // Unsigned always valid
        assert!(!Size::new(-10, 10).is_valid());
        assert!(!Size::new(10, -10).is_valid());
    }

    // --- Rect<T> Tests ---
    #[test]
    fn rect_new_and_components() {
        let r_i32 = Rect::new(Point::new(10, 20), Size::new(30, 40));
        assert_eq!(r_i32.origin, Point::new(10, 20));
        assert_eq!(r_i32.size, Size::new(30, 40));

        let r_f32 = Rect::from_coords(1.5, 2.5, 3.5, 4.5);
        assert_eq!(r_f32.x(), 1.5);
        assert_eq!(r_f32.y(), 2.5);
        assert_eq!(r_f32.width(), 3.5);
        assert_eq!(r_f32.height(), 4.5);
    }

    #[test]
    fn rect_default() {
        let r_i32: Rect<i32> = Default::default();
        assert_eq!(r_i32, Rect::new(Point::new(0,0), Size::new(0,0)));
    }

    #[test]
    fn rect_constants() {
        assert_eq!(Rect::<i32>::ZERO_I32, Rect::from_coords(0,0,0,0));
        assert_eq!(Rect::<u32>::ZERO_U32, Rect::from_coords(0u32,0u32,0u32,0u32));
        assert_eq!(Rect::<f32>::ZERO_F32, Rect::from_coords(0.0f32,0.0f32,0.0f32,0.0f32));
    }
    
    #[test]
    fn rect_edges() {
        let r = Rect::from_coords(10, 20, 30, 40);
        assert_eq!(r.left(), 10);
        assert_eq!(r.top(), 20);
        assert_eq!(r.right(), 10 + 30);
        assert_eq!(r.bottom(), 20 + 40);
    }

    #[test]
    fn rect_center() {
        let r = Rect::from_coords(10, 20, 30, 40); // Center x = 10 + 30/2 = 25, y = 20 + 40/2 = 40
        assert_eq!(r.center(), Point::new(25, 40));
        let r_f = Rect::from_coords(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r_f.center(), Point::new(25.0, 40.0));
    }

    #[test]
    fn rect_contains_point() {
        let r = Rect::from_coords(10, 20, 30, 40); // x: [10, 40), y: [20, 60)
        assert!(r.contains_point(&Point::new(10, 20)));
        assert!(r.contains_point(&Point::new(39, 59)));
        assert!(!r.contains_point(&Point::new(9, 20)));
        assert!(!r.contains_point(&Point::new(10, 19)));
        assert!(!r.contains_point(&Point::new(40, 20))); // right edge is exclusive
        assert!(!r.contains_point(&Point::new(10, 60))); // bottom edge is exclusive
    }

    #[test]
    fn rect_intersects() {
        let r1 = Rect::from_coords(0, 0, 10, 10);
        let r2 = Rect::from_coords(5, 5, 10, 10);
        let r3 = Rect::from_coords(10, 0, 5, 5); // Touches r1 at right edge, no intersection
        let r4 = Rect::from_coords(20, 20, 5, 5); // No intersection

        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
        assert!(!r1.intersects(&r3));
        assert!(!r3.intersects(&r1));
        assert!(!r1.intersects(&r4));
    }

    #[test]
    fn rect_intersection() {
        let r1 = Rect::from_coords(0, 0, 10, 10);
        let r2 = Rect::from_coords(5, 5, 10, 10);
        let expected_isect = Rect::from_coords(5, 5, 5, 5);
        assert_eq!(r1.intersection(&r2), Some(expected_isect));
        assert_eq!(r2.intersection(&r1), Some(expected_isect));

        let r3 = Rect::from_coords(10, 0, 5, 5);
        assert_eq!(r1.intersection(&r3), None);
        
        let r4 = Rect::from_coords(0,0,5,5);
        let r5 = Rect::from_coords(0,0,5,5);
        assert_eq!(r4.intersection(&r5), Some(Rect::from_coords(0,0,5,5)));
    }
    
    #[test]
    fn rect_union() {
        let r1 = Rect::from_coords(0, 0, 10, 10);
        let r2 = Rect::from_coords(5, 5, 10, 10); // Extends to (15,15)
        let expected_union = Rect::from_coords(0, 0, 15, 15);
        assert_eq!(r1.union(&r2), expected_union);
        assert_eq!(r2.union(&r1), expected_union);

        let r3 = Rect::from_coords(20, 20, 5, 5);
        let expected_union_disjoint = Rect::from_coords(0,0, 20+5, 20+5);
        assert_eq!(r1.union(&r3), expected_union_disjoint);
    }

    #[test]
    fn rect_translated() {
        let r = Rect::from_coords(10, 20, 30, 40);
        let delta = Point::new(5, -5);
        let expected = Rect::from_coords(15, 15, 30, 40);
        assert_eq!(r.translated(&delta), expected);
    }

    #[test]
    fn rect_scaled() {
        let r = Rect::from_coords(10, 20, 30, 40);
        let expected = Rect::from_coords(10, 20, 30 * 2, 40 * 3);
        assert_eq!(r.scaled(2, 3), expected);
    }

    #[test]
    fn rect_is_valid() {
        assert!(Rect::from_coords(0,0,10,10).is_valid());
        assert!(Rect::from_coords(0,0,0,0).is_valid());
        assert!(!Rect::from_coords(0,0,-10,10).is_valid());
        assert!(!Rect::from_coords(0,0,10,-10).is_valid());
        assert!(Rect::from_coords(0,0,10u32,10u32).is_valid());
    }
    
    // --- PointInt Tests ---
    #[test]
    fn point_int_works() {
        let p = PointInt::new(1, 2);
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
        let p_default: PointInt = Default::default();
        assert_eq!(p_default, PointInt::new(0,0));
    }

    // --- SizeInt Tests ---
    #[test]
    fn size_int_works() {
        let s = SizeInt::new(10, 20);
        assert_eq!(s.width, 10);
        assert_eq!(s.height, 20);
        assert!(!s.is_empty());
        assert!(SizeInt::new(0, 20).is_empty());
        let s_default: SizeInt = Default::default();
        assert_eq!(s_default, SizeInt::new(0,0));
    }
    
    // --- RectInt Tests ---
    #[test]
    fn rect_int_creation_and_accessors() {
        let r = RectInt::from_coords(10, 20, 30, 40);
        assert_eq!(r.x(), 10);
        assert_eq!(r.y(), 20);
        assert_eq!(r.width(), 30);
        assert_eq!(r.height(), 40);
        assert_eq!(r.origin, PointInt::new(10, 20));
        assert_eq!(r.size, SizeInt::new(30, 40));
        assert_eq!(r.left(), 10);
        assert_eq!(r.top(), 20);
        assert_eq!(r.right(), 10 + 30);
        assert_eq!(r.bottom(), 20 + 40);
        assert!(!r.is_empty());
    }

    #[test]
    fn rect_int_contains_point() {
        let r = RectInt::from_coords(0, 0, 10, 10); // x: [0,10), y: [0,10)
        assert!(r.contains_point(PointInt::new(0, 0)));
        assert!(r.contains_point(PointInt::new(9, 9)));
        assert!(!r.contains_point(PointInt::new(10, 0)));
        assert!(!r.contains_point(PointInt::new(0, 10)));
        assert!(!r.contains_point(PointInt::new(-1, 0)));
    }

    #[test]
    fn rect_int_intersects() {
        let r1 = RectInt::from_coords(0, 0, 10, 10);
        let r2 = RectInt::from_coords(5, 5, 10, 10);
        let r3 = RectInt::from_coords(10, 0, 5, 5);
        let r4 = RectInt::from_coords(20, 20, 5, 5);

        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3)); // Touches, but right edge is exclusive in contains_point logic
        assert!(!r1.intersects(&r4));
    }
    
    #[test]
    fn rect_int_intersection() {
        let r1 = RectInt::from_coords(0, 0, 10, 10);
        let r2 = RectInt::from_coords(5, 5, 10, 10);
        let expected = RectInt::from_coords(5, 5, 5, 5);
        assert_eq!(r1.intersection(&r2), Some(expected));

        let r3 = RectInt::from_coords(10, 0, 5, 5);
        assert_eq!(r1.intersection(&r3), None);
        
        let r4 = RectInt::from_coords(0,0,10,10);
        let r5 = RectInt::from_coords(2,2,6,6);
        assert_eq!(r4.intersection(&r5), Some(RectInt::from_coords(2,2,6,6)));
    }
    
    #[test]
    fn rect_int_union() {
        let r1 = RectInt::from_coords(0, 0, 10, 10);
        let r2 = RectInt::from_coords(5, 5, 10, 10); // right=15, bottom=15
        let expected = RectInt::from_coords(0, 0, 15, 15);
        assert_eq!(r1.union(&r2), expected);

        let r3 = RectInt::from_coords(-5, -5, 5, 5); // x=[-5,0), y=[-5,0)
        let expected2 = RectInt::from_coords(-5, -5, 15, 15); // x=[-5,10), y=[-5,10)
        assert_eq!(r1.union(&r3), expected2);
    }

    #[test]
    fn rect_int_is_empty() {
        assert!(RectInt::from_coords(0,0,0,10).is_empty());
        assert!(RectInt::from_coords(0,0,10,0).is_empty());
        assert!(!RectInt::from_coords(0,0,1,1).is_empty());
    }

    #[test]
    fn rect_int_serde() {
        let r = RectInt::from_coords(1,2,3,4);
        let serialized = serde_json::to_string(&r).unwrap();
        let deserialized: RectInt = serde_json::from_str(&serialized).unwrap();
        assert_eq!(r, deserialized);
    }

    #[test]
    fn point_serde() {
        let p = Point::<i32>::new(1,2);
        let serialized = serde_json::to_string(&p).unwrap();
        let deserialized: Point<i32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(p, deserialized);
    }

    #[test]
    fn size_serde() {
        let s = Size::<u32>::new(3,4);
        let serialized = serde_json::to_string(&s).unwrap();
        let deserialized: Size<u32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(s, deserialized);
    }

    #[test]
    fn rect_serde() {
        let r = Rect::<f32>::from_coords(1.0,2.0,3.0,4.0);
        let serialized = serde_json::to_string(&r).unwrap();
        let deserialized: Rect<f32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(r, deserialized);
    }
}
