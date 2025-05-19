//! Geometry module for the NovaDE core layer.
//!
//! This module provides geometric primitives used throughout the
//! NovaDE desktop environment, including points, sizes, and rectangles.

use std::ops::{Add, Sub, Mul, Div};
use std::fmt;

/// A point in 2D space with generic coordinate type.
///
/// This struct represents a point with x and y coordinates of type T,
/// which can be any numeric type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point<T> {
    /// The x-coordinate
    pub x: T,
    /// The y-coordinate
    pub y: T,
}

impl<T> Point<T> {
    /// Creates a new point with the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate
    /// * `y` - The y-coordinate
    ///
    /// # Returns
    ///
    /// A new `Point<T>` with the specified coordinates.
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

impl<T: Default> Point<T> {
    /// Creates a new point at the origin (0, 0).
    ///
    /// # Returns
    ///
    /// A new `Point<T>` with default values for coordinates.
    pub fn zero() -> Self {
        Point {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T: Copy + Add<Output = T>> Add for Point<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub for Point<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Point<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for Point<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Point {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T: Copy + Add<Output = T> + Mul<Output = T>> Point<T> {
    /// Calculates the dot product of this point and another point.
    ///
    /// # Arguments
    ///
    /// * `other` - The other point
    ///
    /// # Returns
    ///
    /// The dot product of the two points.
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }
}

impl<T: Copy + Add<Output = T> + Mul<Output = T> + Sub<Output = T>> Point<T> {
    /// Calculates the squared distance between this point and another point.
    ///
    /// # Arguments
    ///
    /// * `other` - The other point
    ///
    /// # Returns
    ///
    /// The squared distance between the two points.
    pub fn distance_squared(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

/// A size in 2D space with generic dimension type.
///
/// This struct represents dimensions with width and height of type T,
/// which can be any numeric type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size<T> {
    /// The width
    pub width: T,
    /// The height
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new size with the given dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - The width
    /// * `height` - The height
    ///
    /// # Returns
    ///
    /// A new `Size<T>` with the specified dimensions.
    pub fn new(width: T, height: T) -> Self {
        Size { width, height }
    }
}

impl<T: Default> Size<T> {
    /// Creates a new size with zero dimensions.
    ///
    /// # Returns
    ///
    /// A new `Size<T>` with default values for dimensions.
    pub fn zero() -> Self {
        Size {
            width: T::default(),
            height: T::default(),
        }
    }
}

impl<T: Copy + Add<Output = T>> Add for Size<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Size {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub for Size<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Size {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Size<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Size {
            width: self.width * scalar,
            height: self.height * scalar,
        }
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for Size<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Size {
            width: self.width / scalar,
            height: self.height / scalar,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Size<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{}", self.width, self.height)
    }
}

impl<T: Copy + Mul<Output = T>> Size<T> {
    /// Calculates the area of this size.
    ///
    /// # Returns
    ///
    /// The area (width * height).
    pub fn area(&self) -> T {
        self.width * self.height
    }
}

/// A rectangle in 2D space with generic coordinate type.
///
/// This struct represents a rectangle with position and size of type T,
/// which can be any numeric type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect<T> {
    /// The position (top-left corner)
    pub position: Point<T>,
    /// The size (width and height)
    pub size: Size<T>,
}

impl<T> Rect<T> {
    /// Creates a new rectangle with the given position and size.
    ///
    /// # Arguments
    ///
    /// * `position` - The position (top-left corner)
    /// * `size` - The size (width and height)
    ///
    /// # Returns
    ///
    /// A new `Rect<T>` with the specified position and size.
    pub fn new(position: Point<T>, size: Size<T>) -> Self {
        Rect { position, size }
    }
}

impl<T: Copy> Rect<T> {
    /// Creates a new rectangle with the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner
    /// * `y` - The y-coordinate of the top-left corner
    /// * `width` - The width
    /// * `height` - The height
    ///
    /// # Returns
    ///
    /// A new `Rect<T>` with the specified coordinates.
    pub fn from_coords(x: T, y: T, width: T, height: T) -> Self {
        Rect {
            position: Point::new(x, y),
            size: Size::new(width, height),
        }
    }
}

impl<T: Copy + Add<Output = T>> Rect<T> {
    /// Gets the x-coordinate of the right edge of the rectangle.
    ///
    /// # Returns
    ///
    /// The x-coordinate of the right edge.
    pub fn right(&self) -> T {
        self.position.x + self.size.width
    }

    /// Gets the y-coordinate of the bottom edge of the rectangle.
    ///
    /// # Returns
    ///
    /// The y-coordinate of the bottom edge.
    pub fn bottom(&self) -> T {
        self.position.y + self.size.height
    }
}

impl<T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T>> Rect<T> {
    /// Checks if this rectangle contains the given point.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to check
    ///
    /// # Returns
    ///
    /// `true` if the rectangle contains the point, `false` otherwise.
    pub fn contains(&self, point: Point<T>) -> bool {
        point.x >= self.position.x && point.x < self.right() &&
        point.y >= self.position.y && point.y < self.bottom()
    }

    /// Calculates the intersection of this rectangle with another rectangle.
    ///
    /// # Arguments
    ///
    /// * `other` - The other rectangle
    ///
    /// # Returns
    ///
    /// An `Option<Rect<T>>` containing the intersection if the rectangles overlap,
    /// or `None` if they don't.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let left = if self.position.x > other.position.x { self.position.x } else { other.position.x };
        let top = if self.position.y > other.position.y { self.position.y } else { other.position.y };
        let right = if self.right() < other.right() { self.right() } else { other.right() };
        let bottom = if self.bottom() < other.bottom() { self.bottom() } else { other.bottom() };

        if left < right && top < bottom {
            Some(Rect::from_coords(
                left,
                top,
                right - left,
                bottom - top,
            ))
        } else {
            None
        }
    }
}

impl<T: Default> Rect<T> {
    /// Creates a new rectangle at the origin with zero size.
    ///
    /// # Returns
    ///
    /// A new `Rect<T>` with default values for position and size.
    pub fn zero() -> Self {
        Rect {
            position: Point::zero(),
            size: Size::zero(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} at {}]", self.size, self.position)
    }
}

/// A rectangle with integer coordinates.
///
/// This type alias represents a rectangle with i32 coordinates,
/// which is commonly used for pixel-based operations.
pub type RectInt = Rect<i32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let p = Point::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    #[test]
    fn test_point_zero() {
        let p: Point<i32> = Point::zero();
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
    }

    #[test]
    fn test_point_add() {
        let p1 = Point::new(10, 20);
        let p2 = Point::new(5, 7);
        let result = p1 + p2;
        assert_eq!(result.x, 15);
        assert_eq!(result.y, 27);
    }

    #[test]
    fn test_point_sub() {
        let p1 = Point::new(10, 20);
        let p2 = Point::new(5, 7);
        let result = p1 - p2;
        assert_eq!(result.x, 5);
        assert_eq!(result.y, 13);
    }

    #[test]
    fn test_point_mul() {
        let p = Point::new(10, 20);
        let result = p * 2;
        assert_eq!(result.x, 20);
        assert_eq!(result.y, 40);
    }

    #[test]
    fn test_point_div() {
        let p = Point::new(10, 20);
        let result = p / 2;
        assert_eq!(result.x, 5);
        assert_eq!(result.y, 10);
    }

    #[test]
    fn test_point_dot() {
        let p1 = Point::new(3, 4);
        let p2 = Point::new(5, 6);
        let result = p1.dot(&p2);
        assert_eq!(result, 3 * 5 + 4 * 6);
    }

    #[test]
    fn test_point_distance_squared() {
        let p1 = Point::new(3, 4);
        let p2 = Point::new(6, 8);
        let result = p1.distance_squared(&p2);
        assert_eq!(result, 9 + 16); // (6-3)² + (8-4)² = 3² + 4² = 9 + 16 = 25
    }

    #[test]
    fn test_size_new() {
        let s = Size::new(100, 200);
        assert_eq!(s.width, 100);
        assert_eq!(s.height, 200);
    }

    #[test]
    fn test_size_zero() {
        let s: Size<i32> = Size::zero();
        assert_eq!(s.width, 0);
        assert_eq!(s.height, 0);
    }

    #[test]
    fn test_size_add() {
        let s1 = Size::new(100, 200);
        let s2 = Size::new(50, 70);
        let result = s1 + s2;
        assert_eq!(result.width, 150);
        assert_eq!(result.height, 270);
    }

    #[test]
    fn test_size_sub() {
        let s1 = Size::new(100, 200);
        let s2 = Size::new(50, 70);
        let result = s1 - s2;
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 130);
    }

    #[test]
    fn test_size_mul() {
        let s = Size::new(100, 200);
        let result = s * 2;
        assert_eq!(result.width, 200);
        assert_eq!(result.height, 400);
    }

    #[test]
    fn test_size_div() {
        let s = Size::new(100, 200);
        let result = s / 2;
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 100);
    }

    #[test]
    fn test_size_area() {
        let s = Size::new(10, 20);
        assert_eq!(s.area(), 200);
    }

    #[test]
    fn test_rect_new() {
        let position = Point::new(10, 20);
        let size = Size::new(100, 200);
        let rect = Rect::new(position, size);
        assert_eq!(rect.position.x, 10);
        assert_eq!(rect.position.y, 20);
        assert_eq!(rect.size.width, 100);
        assert_eq!(rect.size.height, 200);
    }

    #[test]
    fn test_rect_from_coords() {
        let rect = Rect::from_coords(10, 20, 100, 200);
        assert_eq!(rect.position.x, 10);
        assert_eq!(rect.position.y, 20);
        assert_eq!(rect.size.width, 100);
        assert_eq!(rect.size.height, 200);
    }

    #[test]
    fn test_rect_right_bottom() {
        let rect = Rect::from_coords(10, 20, 100, 200);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 220);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::from_coords(10, 20, 100, 200);
        
        // Points inside
        assert!(rect.contains(Point::new(10, 20)));
        assert!(rect.contains(Point::new(50, 50)));
        assert!(rect.contains(Point::new(109, 219)));
        
        // Points outside
        assert!(!rect.contains(Point::new(9, 20)));
        assert!(!rect.contains(Point::new(10, 19)));
        assert!(!rect.contains(Point::new(110, 20)));
        assert!(!rect.contains(Point::new(10, 220)));
    }

    #[test]
    fn test_rect_intersect() {
        let rect1 = Rect::from_coords(10, 20, 100, 200);
        
        // Overlapping rectangle
        let rect2 = Rect::from_coords(50, 60, 100, 200);
        let intersection = rect1.intersect(&rect2).unwrap();
        assert_eq!(intersection.position.x, 50);
        assert_eq!(intersection.position.y, 60);
        assert_eq!(intersection.size.width, 60);
        assert_eq!(intersection.size.height, 160);
        
        // Non-overlapping rectangle
        let rect3 = Rect::from_coords(200, 300, 50, 50);
        assert!(rect1.intersect(&rect3).is_none());
    }

    #[test]
    fn test_rect_zero() {
        let rect: Rect<i32> = Rect::zero();
        assert_eq!(rect.position.x, 0);
        assert_eq!(rect.position.y, 0);
        assert_eq!(rect.size.width, 0);
        assert_eq!(rect.size.height, 0);
    }
}
