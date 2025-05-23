//! Geometric Primitives.
//!
//! This module provides fundamental 2D geometric primitives, such as points, sizes, and rectangles.
//! These types are generic over the coordinate type `T`, allowing them to be used with
//! various numeric types (e.g., `i32` for pixel coordinates, `f32` for fractional coordinates).
//!
//! # Core Types
//!
//! - [`Point<T>`]: Represents a 2D point with `x` and `y` coordinates.
//! - [`Size<T>`]: Represents a 2D size with `width` and `height` dimensions.
//! - [`Rect<T>`]: Represents a generic rectangle defined by a `Point<T>` (top-left) and a `Size<T>`.
//! - [`RectInt`]: A specialized rectangle using `i32` for its top-left coordinates and `u32` for
//!   its width and height, commonly used for pixel-based screen regions.
//!
//! # Usage
//!
//! These types support common geometric operations, including arithmetic (addition, subtraction,
//! scaling), distance calculations, and intersection/union for rectangles.
//!
//! ```
//! use novade_core::types::{Point, Size, Rect, RectInt};
//!
//! // Working with generic Points and Sizes
//! let p1 = Point::new(10, 20);
//! let p2 = Point::new(5, 5);
//! let p3 = p1 + p2; // p3 is (15, 25)
//! assert_eq!(p3, Point::new(15, 25));
//!
//! let size = Size::new(100.0, 50.0);
//! let area = size.area(); // area is 5000.0
//! assert_eq!(area, 5000.0);
//!
//! // Working with generic Rects
//! let rect_f = Rect::from_coords(10.0, 10.0, 20.0, 20.0);
//! assert!(rect_f.contains_point(Point::new(15.0, 15.0)));
//!
//! // Working with RectInt for pixel coordinates
//! let rect_i = RectInt::new(0, 0, 800, 600);
//! assert_eq!(rect_i.right(), 800);
//! assert!(rect_i.contains_point_coords(100, 100));
//! ```

use std::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign};
use std::fmt;
use num_traits::{Float, Signed, Zero, FromPrimitive};
use std::cmp::{min, max};


/// A point in 2D space with generic coordinate type `T`.
///
/// This struct represents a point with `x` and `y` coordinates. The type `T`
/// can be any type that supports the required operations for points, typically
/// a numeric type like `i32` or `f32`.
///
/// # Examples
///
/// ```
/// use novade_core::types::Point;
///
/// let int_point = Point::new(10, 20);
/// let float_point = Point::new(5.5, 10.1);
///
/// assert_eq!(int_point.x, 10);
/// assert_eq!(float_point.y, 10.1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point<T> {
    /// The x-coordinate of the point.
    pub x: T,
    /// The y-coordinate of the point.
    pub y: T,
}

impl<T> Point<T> {
    /// Creates a new point with the given `x` and `y` coordinates.
    ///
    /// # Arguments
    ///
    /// * `x`: The x-coordinate.
    /// * `y`: The y-coordinate.
    ///
    /// # Returns
    ///
    /// A new `Point<T>` instance.
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    /// A constant representing the origin point (0,0) for `i32` coordinates.
    pub const ZERO_I32: Point<i32> = Point { x: 0, y: 0 };
    /// A constant representing the origin point (0.0,0.0) for `f32` coordinates.
    pub const ZERO_F32: Point<f32> = Point { x: 0.0, y: 0.0 };
}

impl<T: Zero> Default for Point<T> {
    /// Creates a new point at the origin (T::zero(), T::zero()), utilizing the [`Zero`] trait.
    /// This is the preferred way to get a zero point for generic types.
    fn default() -> Self {
        Point {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

// Original Default impl renamed to zero_legacy_default() and kept for compatibility if needed,
// but Default::default() is preferred.
impl<T: Default> Point<T> {
    /// Creates a new point at the origin using `T::default()`.
    ///
    /// **Note:** Prefer using `Point::<T>::default()` where `T: num_traits::Zero`.
    /// This method is kept for compatibility with types that implement `Default` but not `Zero`.
    ///
    /// # Returns
    ///
    /// A new `Point<T>` with default values for coordinates.
    #[deprecated(since="0.1.0", note="Prefer Point::<T>::default() where T: num_traits::Zero, or Point::ZERO_I32/ZERO_F32 for common types.")]
    pub fn zero_legacy_default() -> Self { // Renamed to avoid conflict
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

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Signed> Point<T> {
    /// Calculates the Manhattan distance to another point.
    ///
    /// The Manhattan distance is calculated as `|x1 - x2| + |y1 - y2|`.
    /// This requires `T` to support subtraction and the `Signed::abs()` method.
    pub fn manhattan_distance(&self, other: &Point<T>) -> T {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Float> Point<T> {
    /// Calculates the Euclidean distance to another point.
    ///
    /// This is calculated as `sqrt((x1 - x2)² + (y1 - y2)²)`.
    /// This requires `T` to support floating-point operations via `num_traits::Float`.
    pub fn distance(&self, other: &Point<T>) -> T {
        self.distance_squared(other).sqrt()
    }
}


/// A size in 2D space with generic dimension type `T`.
///
/// Represents a 2D extent with `width` and `height`. The type `T` can be any type
/// that supports the required operations for sizes, typically a numeric type.
///
/// # Examples
///
/// ```
/// use novade_core::types::Size;
///
/// let s1 = Size::new(800, 600);      // Integer size
/// let s2 = Size::new(100.5, 75.25); // Floating point size
///
/// assert_eq!(s1.width, 800);
/// assert_eq!(s2.height, 75.25);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size<T> {
    /// The width component of the size.
    pub width: T,
    /// The height component of the size.
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new size with the given `width` and `height`.
    ///
    /// # Arguments
    ///
    /// * `width`: The width.
    /// * `height`: The height.
    ///
    /// # Returns
    ///
    /// A new `Size<T>` instance.
    pub fn new(width: T, height: T) -> Self {
        Size { width, height }
    }

    /// A constant representing a zero size (0,0) for `i32` dimensions.
    pub const ZERO_I32: Size<i32> = Size { width: 0, height: 0 };
    /// A constant representing a zero size (0.0,0.0) for `f32` dimensions.
    pub const ZERO_F32: Size<f32> = Size { width: 0.0, height: 0.0 };
}

impl<T: Zero> Default for Size<T> {
    /// Creates a new size with zero dimensions (T::zero(), T::zero()), using the [`Zero`] trait.
    /// This is the preferred way to get a zero size for generic types.
    fn default() -> Self {
        Size {
            width: T::zero(),
            height: T::zero(),
        }
    }
}

// Original Default impl renamed to zero_legacy_default() and kept for compatibility if needed,
// but Default::default() is preferred.
impl<T: Default> Size<T> {
    /// Creates a new size with zero dimensions using `T::default()`.
    ///
    /// **Note:** Prefer using `Size::<T>::default()` where `T: num_traits::Zero`.
    /// This method is kept for compatibility with types that implement `Default` but not `Zero`.
    ///
    /// # Returns
    ///
    /// A new `Size<T>` with default values for dimensions.
    #[deprecated(since="0.1.0", note="Prefer Size::<T>::default() where T: num_traits::Zero, or Size::ZERO_I32/ZERO_F32 for common types.")]
    pub fn zero_legacy_default() -> Self { // Renamed to avoid conflict
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

impl<T: Copy + PartialEq + Zero> Size<T> {
    /// Checks if the width or height is zero.
    ///
    /// This requires `T` to support equality comparison and have a zero value
    /// through the [`Zero`] trait.
    pub fn is_empty(&self) -> bool {
        self.width == T::zero() || self.height == T::zero()
    }
}

impl<T: Copy + PartialOrd + Zero> Size<T> {
    /// Checks if width and height are non-negative.
    ///
    /// This requires `T` to support partial ordering and have a zero value
    /// through the [`Zero`] trait.
    pub fn is_valid(&self) -> bool {
        self.width >= T::zero() && self.height >= T::zero()
    }
}

/// A rectangle in 2D space with generic coordinate type `T`.
///
/// This struct represents an axis-aligned rectangle defined by its top-left [`Point<T>`]
/// and its [`Size<T>`]. The type `T` can be any type that supports the required
/// operations, typically a numeric type.
///
/// # Examples
///
/// ```
/// use novade_core::types::{Rect, Point, Size};
///
/// let rect = Rect::new(Point::new(10, 20), Size::new(100, 50));
/// assert_eq!(rect.x(), 10);
/// assert_eq!(rect.width(), 100);
/// assert_eq!(rect.right(), 110); // 10 (x) + 100 (width)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect<T> {
    /// The position of the top-left corner of the rectangle.
    pub position: Point<T>,
    /// The dimensions (width and height) of the rectangle.
    pub size: Size<T>,
}

impl<T> Rect<T> {
    /// Creates a new rectangle from a given top-left `position` and `size`.
    ///
    /// # Arguments
    ///
    /// * `position`: The [`Point<T>`] representing the top-left corner.
    /// * `size`: The [`Size<T>`] representing the width and height.
    ///
    /// # Returns
    ///
    /// A new `Rect<T>` instance.
    pub fn new(position: Point<T>, size: Size<T>) -> Self {
        Rect { position, size }
    }

    /// A constant representing a zero-sized rectangle at the origin for `i32` types.
    pub const ZERO_I32: Rect<i32> = Rect { position: Point::ZERO_I32, size: Size::ZERO_I32 };
    /// A constant representing a zero-sized rectangle at the origin for `f32` types.
    pub const ZERO_F32: Rect<f32> = Rect { position: Point::ZERO_F32, size: Size::ZERO_F32 };

    /// Gets the x-coordinate of the top-left corner. Requires `T: Copy`.
    pub fn x(&self) -> T where T: Copy { self.position.x }
    /// Gets the y-coordinate of the top-left corner. Requires `T: Copy`.
    pub fn y(&self) -> T where T: Copy { self.position.y }
    /// Gets the width of the rectangle. Requires `T: Copy`.
    pub fn width(&self) -> T where T: Copy { self.size.width }
    /// Gets the height of the rectangle. Requires `T: Copy`.
    pub fn height(&self) -> T where T: Copy { self.size.height }
    /// Gets the y-coordinate of the top edge (equivalent to `y()`). Requires `T: Copy`.
    pub fn top(&self) -> T where T: Copy { self.position.y }
    /// Gets the x-coordinate of the left edge (equivalent to `x()`). Requires `T: Copy`.
    pub fn left(&self) -> T where T: Copy { self.position.x }
}

impl<T: Copy> Rect<T> {
    /// Creates a new rectangle directly from `x`, `y`, `width`, and `height` values.
    ///
    /// # Arguments
    ///
    /// * `x`: The x-coordinate of the top-left corner.
    /// * `y`: The y-coordinate of the top-left corner.
    /// * `width`: The width of the rectangle.
    /// * `height`: The height of the rectangle.
    ///
    /// # Returns
    ///
    /// A new `Rect<T>` instance.
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

impl<T: Copy + Add<Output = T> + Div<Output = T> + FromPrimitive> Rect<T> {
    /// Calculates the center point of the rectangle.
    ///
    /// Requires `T` to support addition, division by 2 (via `FromPrimitive`),
    /// and be `Copy`.
    ///
    /// # Panics
    /// Panics if `T::from_u8(2)` returns `None`, which might happen for types
    /// that cannot represent the number 2.
    pub fn center(&self) -> Point<T> {
        let two = T::from_u8(2).unwrap_or_else(|| panic!("Cannot create '2' for type T"));
        Point::new(self.left() + self.width() / two, self.top() + self.height() / two)
    }
}

impl<T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T>> Rect<T> {
    /// Checks if this rectangle contains the given point.
    ///
    /// The containment check is inclusive for the top-left edge and exclusive
    /// for the bottom-right edge. That is, a point `(px, py)` is contained if:
    /// `rect.left() <= px < rect.right()` and `rect.top() <= py < rect.bottom()`.
    ///
    /// # Arguments
    ///
    /// * `point`: The [`Point<T>`] to check.
    ///
    /// # Returns
    ///
    /// `true` if the rectangle contains the point, `false` otherwise.
    pub fn contains_point(&self, point: Point<T>) -> bool {
        point.x >= self.left() && point.x < self.right() &&
        point.y >= self.top() && point.y < self.bottom()
    }

    /// Checks if this rectangle overlaps with another rectangle.
    ///
    /// Rectangles are considered overlapping if there is any common area between them.
    /// Touching edges without overlap does not count as intersection.
    ///
    /// # Arguments
    ///
    /// * `other`: The other `Rect<T>` to check against.
    ///
    /// # Returns
    ///
    /// `true` if the rectangles overlap, `false` otherwise.
    pub fn intersects(&self, other: &Self) -> bool {
        self.left() < other.right() && self.right() > other.left() &&
        self.top() < other.bottom() && self.bottom() > other.top()
    }
    
    /// Calculates the intersection of this rectangle with another rectangle.
    ///
    /// If the rectangles do not overlap, `None` is returned. Otherwise, a new `Rect<T>`
    /// representing the overlapping area is returned.
    ///
    /// # Arguments
    ///
    /// * `other`: The other `Rect<T>` to intersect with.
    ///
    /// # Returns
    ///
    /// An `Option<Rect<T>>` containing the intersection, or `None` if they don't overlap.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let x1 = self.left().max(other.left());
        let y1 = self.top().max(other.top());
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x1 < x2 && y1 < y2 {
            Some(Rect::from_coords(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }
}

impl<T: Copy + Ord + Add<Output = T> + Sub<Output = T>> Rect<T> {
    /// Calculates the smallest rectangle that encompasses both this rectangle and another.
    ///
    /// This is also known as the bounding box of the two rectangles.
    ///
    /// # Arguments
    ///
    /// * `other`: The other `Rect<T>` to include in the union.
    ///
    /// # Returns
    ///
    /// A new `Rect<T>` representing the union of the two rectangles.
    pub fn union(&self, other: &Self) -> Self {
        let x1 = self.left().min(other.left());
        let y1 = self.top().min(other.top());
        let x2 = self.right().max(other.right());
        let y2 = self.bottom().max(other.bottom());
        Rect::from_coords(x1, y1, x2 - x1, y2 - y1)
    }
}

impl<T: Copy + Add<Output = T>> Rect<T> {
    /// Returns a new rectangle translated by `dx` and `dy`.
    ///
    /// The size of the rectangle remains the same, but its position is shifted.
    ///
    /// # Arguments
    ///
    /// * `dx`: The amount to translate in the x-direction.
    /// * `dy`: The amount to translate in the y-direction.
    ///
    /// # Returns
    ///
    /// A new, translated `Rect<T>`.
    pub fn translated(&self, dx: T, dy: T) -> Self {
        Rect::new(Point::new(self.position.x + dx, self.position.y + dy), self.size)
    }
}

impl<T: Copy + Mul<Output = T>> Rect<T> {
    /// Returns a new rectangle with its position and size scaled by factors `sx` and `sy`.
    ///
    /// Both the top-left `position` coordinates (`x`, `y`) and the `size` dimensions
    /// (`width`, `height`) are multiplied by the respective scaling factors.
    ///
    /// # Arguments
    ///
    /// * `sx`: The scaling factor for the x-coordinate and width.
    /// * `sy`: The scaling factor for the y-coordinate and height.
    ///
    /// # Returns
    ///
    /// A new, scaled `Rect<T>`.
    pub fn scaled(&self, sx: T, sy: T) -> Self {
        Rect::new(
            Point::new(self.position.x * sx, self.position.y * sy),
            Size::new(self.size.width * sx, self.size.height * sy)
        )
    }
}

impl<T: Copy + PartialOrd + Zero> Rect<T> {
    /// Checks if the rectangle's size is valid (i.e., width and height are non-negative).
    /// This relies on `Size::is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.size.is_valid()
    }
}


impl<T: Zero> Default for Rect<T> {
    /// Creates a new rectangle at the origin with zero size, using `Point::<T>::default()`
    /// and `Size::<T>::default()`. This is the preferred way to get a zero rectangle for generic types.
    fn default() -> Self {
        Rect {
            position: Point::<T>::default(),
            size: Size::<T>::default(),
        }
    }
}

// Original Default impl renamed to zero_legacy_default() and kept for compatibility if needed,
// but Default::default() is preferred.
impl<T: Default> Rect<T> {
    /// Creates a new rectangle at the origin with zero size using `T::default()`.
    ///
    /// **Note:** Prefer using `Rect::<T>::default()` where `T: num_traits::Zero`.
    /// This method is kept for compatibility.
    ///
    /// # Returns
    ///
    /// A new `Rect<T>` with default values for position and size.
    #[deprecated(since="0.1.0", note="Prefer Rect::<T>::default() where T: num_traits::Zero, or Rect::ZERO_I32/ZERO_F32 for common types.")]
    pub fn zero_legacy_default() -> Self { // Renamed
        Rect {
            position: Point::zero_legacy_default(),
            size: Size::zero_legacy_default(),
        }
    }
}


impl<T: fmt::Display> fmt::Display for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} at {}]", self.size, self.position)
    }
}

/// A rectangle with `i32` coordinates for position and `u32` for size.
///
/// This struct is optimized for pixel-based operations where negative sizes are invalid
/// and fractional coordinates are not needed. It provides methods tailored for
/// integer arithmetic and screen coordinate systems.
///
/// # Fields
/// - `x`: The x-coordinate of the top-left corner.
/// - `y`: The y-coordinate of the top-left corner.
/// - `width`: The width of the rectangle (must be non-negative).
/// - `height`: The height of the rectangle (must be non-negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))] // If serde is a feature
pub struct RectInt {
    /// The x-coordinate of the top-left corner.
    pub x: i32,
    /// The y-coordinate of the top-left corner.
    pub y: i32,
    /// The width of the rectangle.
    pub width: u32,
    /// The height of the rectangle.
    pub height: u32,
}

impl RectInt {
    /// A constant representing a zero-sized `RectInt` at the origin (0,0).
    pub const ZERO: RectInt = RectInt { x: 0, y: 0, width: 0, height: 0 };

    /// Creates a new `RectInt` with the specified position and dimensions.
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        RectInt { x, y, width, height }
    }

    /// Creates a `RectInt` that encompasses the area between two points `p1` and `p2`.
    /// The resulting rectangle will have `p1` and `p2` as opposite corners.
    /// The width and height are calculated as the absolute difference between the points' coordinates.
    pub fn from_points(p1: Point<i32>, p2: Point<i32>) -> Self {
        let x = min(p1.x, p2.x);
        let y = min(p1.y, p2.y);
        let width = (p1.x - p2.x).abs() as u32; // abs() ensures non-negative before cast
        let height = (p1.y - p2.y).abs() as u32; // abs() ensures non-negative before cast
        RectInt { x, y, width, height }
    }

    /// Returns the top-left corner of the rectangle as a [`Point<i32>`].
    pub fn top_left(&self) -> Point<i32> {
        Point::new(self.x, self.y)
    }

    /// Returns the size of the rectangle as a [`Size<u32>`].
    pub fn size(&self) -> Size<u32> {
        Size::new(self.width, self.height)
    }
    
    /// Returns the x-coordinate of the left edge (equivalent to `self.x`).
    pub fn left(&self) -> i32 { self.x }
    /// Returns the y-coordinate of the top edge (equivalent to `self.y`).
    pub fn top(&self) -> i32 { self.y }

    /// Returns the x-coordinate of the right edge (exclusive).
    /// Calculated as `x + width`. Uses `saturating_add` to prevent overflow.
    pub fn right(&self) -> i32 {
        self.x.saturating_add(self.width as i32)
    }

    /// Returns the y-coordinate of the bottom edge (exclusive).
    /// Calculated as `y + height`. Uses `saturating_add` to prevent overflow.
    pub fn bottom(&self) -> i32 {
        self.y.saturating_add(self.height as i32)
    }

    /// Checks if the rectangle contains the given point coordinates (`p_x`, `p_y`).
    /// The containment check is inclusive for the top-left edge and exclusive
    /// for the bottom-right edge.
    pub fn contains_point_coords(&self, p_x: i32, p_y: i32) -> bool {
        p_x >= self.x && p_x < self.right() && p_y >= self.y && p_y < self.bottom()
    }
    
    /// Checks if this rectangle contains the given [`Point<i32>`].
    /// Delegates to [`RectInt::contains_point_coords`].
    pub fn contains_point(&self, p: Point<i32>) -> bool {
        self.contains_point_coords(p.x, p.y)
    }

    /// Checks if this `RectInt` overlaps with another `RectInt`.
    pub fn intersects(&self, other: RectInt) -> bool {
        self.x < other.right() && self.right() > other.x &&
        self.y < other.bottom() && self.bottom() > other.y
    }

    /// Calculates the intersection of this `RectInt` with another `RectInt`.
    /// Returns `None` if there is no overlap.
    pub fn intersection(&self, other: RectInt) -> Option<RectInt> {
        let res_x = max(self.x, other.x);
        let res_y = max(self.y, other.y);
        let res_right = min(self.right(), other.right());
        let res_bottom = min(self.bottom(), other.bottom());

        if res_x < res_right && res_y < res_bottom {
            Some(RectInt {
                x: res_x,
                y: res_y,
                width: (res_right - res_x) as u32, // Cast is safe as res_x < res_right
                height: (res_bottom - res_y) as u32, // Cast is safe as res_y < res_bottom
            })
        } else {
            None
        }
    }

    /// Calculates the smallest `RectInt` that encompasses both this rectangle and another.
    pub fn union(&self, other: RectInt) -> RectInt {
        let res_x = min(self.x, other.x);
        let res_y = min(self.y, other.y);
        let res_right = max(self.right(), other.right());
        let res_bottom = max(self.bottom(), other.bottom());
        RectInt {
            x: res_x,
            y: res_y,
            width: (res_right - res_x) as u32, // Cast is safe due to min/max logic
            height: (res_bottom - res_y) as u32, // Cast is safe due to min/max logic
        }
    }

    /// Returns a new `RectInt` translated by (`dx`, `dy`).
    /// Uses `saturating_add` for position to prevent overflow.
    pub fn translated(&self, dx: i32, dy: i32) -> RectInt {
        RectInt {
            x: self.x.saturating_add(dx),
            y: self.y.saturating_add(dy),
            width: self.width,
            height: self.height,
        }
    }

    /// Returns a new `RectInt` "inflated" by `dw` and `dh`.
    /// The `x` and `y` coordinates are decreased by `dw` and `dh` respectively.
    /// The `width` and `height` are increased by `2 * dw` and `2 * dh` respectively.
    /// If inflation results in a negative effective width or height, it's clamped to 0.
    pub fn inflate(&self, dw: i32, dh: i32) -> RectInt {
        let new_x = self.x - dw;
        let new_y = self.y - dh;
        let new_width = self.width as i32 + dw * 2;
        let new_height = self.height as i32 + dh * 2;

        RectInt {
            x: new_x,
            y: new_y,
            width: if new_width < 0 { 0 } else { new_width as u32 },
            height: if new_height < 0 { 0 } else { new_height as u32 },
        }
    }
    
    /// Checks if the rectangle has zero width or zero height.
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

impl Default for RectInt {
    /// Returns `RectInt::ZERO`.
    fn default() -> Self {
        RectInt::ZERO
    }
}


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
        let p_i32: Point<i32> = Default::default();
        assert_eq!(p_i32.x, 0);
        assert_eq!(p_i32.y, 0);
        assert_eq!(Point::<i32>::ZERO_I32, p_i32);


        let p_f32: Point<f32> = Default::default();
        assert_eq!(p_f32.x, 0.0);
        assert_eq!(p_f32.y, 0.0);
        assert_eq!(Point::<f32>::ZERO_F32, p_f32);
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
        assert_eq!(result, 25); // (6-3)² + (8-4)² = 3² + 4² = 9 + 16 = 25
    }

    #[test]
    fn test_point_manhattan_distance() {
        let p1 = Point::new(3, 4);
        let p2 = Point::new(6, 8);
        assert_eq!(p1.manhattan_distance(&p2), 7); // |3-6| + |4-8| = 3 + 4 = 7

        let p3 = Point::new(-1, -1);
        let p4 = Point::new(1, 1);
        assert_eq!(p3.manhattan_distance(&p4), 4); // |-1-1| + |-1-1| = 2 + 2 = 4
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(3.0, 4.0);
        let p2 = Point::new(6.0, 8.0);
        let result = p1.distance(&p2);
        assert!((result - 5.0).abs() < f32::EPSILON); // sqrt(25) = 5
    }

    #[test]
    fn test_size_new() {
        let s = Size::new(100, 200);
        assert_eq!(s.width, 100);
        assert_eq!(s.height, 200);
    }

    #[test]
    fn test_size_zero() {
        let s_i32: Size<i32> = Default::default();
        assert_eq!(s_i32.width, 0);
        assert_eq!(s_i32.height, 0);
        assert_eq!(Size::<i32>::ZERO_I32, s_i32);

        let s_f32: Size<f32> = Default::default();
        assert_eq!(s_f32.width, 0.0);
        assert_eq!(s_f32.height, 0.0);
        assert_eq!(Size::<f32>::ZERO_F32, s_f32);
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

        let s_f: Size<f32> = Size::new(10.0, 2.5);
        assert_eq!(s_f.area(), 25.0);
    }

    #[test]
    fn test_size_is_empty() {
        let s1: Size<i32> = Size::new(0, 10);
        assert!(s1.is_empty());
        let s2: Size<i32> = Size::new(10, 0);
        assert!(s2.is_empty());
        let s3: Size<i32> = Size::new(0, 0);
        assert!(s3.is_empty());
        let s4: Size<i32> = Size::new(10, 10);
        assert!(!s4.is_empty());

        let s_f1: Size<f32> = Size::new(0.0, 10.0);
        assert!(s_f1.is_empty());
        let s_f2: Size<f32> = Size::new(10.0, 0.0);
        assert!(s_f2.is_empty());
        let s_f3: Size<f32> = Size::new(10.0, 10.0);
        assert!(!s_f3.is_empty());
    }

    #[test]
    fn test_size_is_valid() {
        let s_i1: Size<i32> = Size::new(10, 20);
        assert!(s_i1.is_valid());
        let s_i2: Size<i32> = Size::new(0, 0);
        assert!(s_i2.is_valid());
        let s_i3: Size<i32> = Size::new(-1, 10);
        assert!(!s_i3.is_valid());
        let s_i4: Size<i32> = Size::new(10, -1);
        assert!(!s_i4.is_valid());
        let s_i5: Size<i32> = Size::new(-5, -1);
        assert!(!s_i5.is_valid());

        let s_u1: Size<u32> = Size::new(10, 20);
        assert!(s_u1.is_valid());
        let s_u2: Size<u32> = Size::new(0, 0);
        assert!(s_u2.is_valid());
        // Cannot test negative for u32 directly as it's not representable
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

        let rect_f = Rect::from_coords(10.0, 20.0, 100.0, 200.0);
        assert_eq!(rect_f.right(), 110.0);
        assert_eq!(rect_f.bottom(), 220.0);
    }

    #[test]
    fn test_rect_accessors() {
        let rect = Rect::from_coords(10, 20, 100, 200);
        assert_eq!(rect.x(), 10);
        assert_eq!(rect.y(), 20);
        assert_eq!(rect.width(), 100);
        assert_eq!(rect.height(), 200);
        assert_eq!(rect.top(), 20);
        assert_eq!(rect.left(), 10);
    }

    #[test]
    fn test_rect_center() {
        let rect_i = Rect::from_coords(10, 20, 100, 200); // center (10+50, 20+100) = (60, 120)
        assert_eq!(rect_i.center(), Point::new(60, 120));
        
        let rect_f = Rect::from_coords(10.0, 20.0, 100.0, 200.0); // center (10+50, 20+100) = (60, 120)
        assert_eq!(rect_f.center(), Point::new(60.0, 120.0));

        let rect_odd = Rect::from_coords(0, 0, 5, 5); // center (0+2, 0+2) = (2,2) for int
        assert_eq!(rect_odd.center(), Point::new(2,2));

        let rect_odd_f = Rect::from_coords(0.0, 0.0, 5.0, 5.0); // center (0+2.5, 0+2.5) = (2.5,2.5) for float
        assert_eq!(rect_odd_f.center(), Point::new(2.5,2.5));
    }

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect::from_coords(10, 20, 100, 200);
        
        // Points inside
        assert!(rect.contains_point(Point::new(10, 20)));
        assert!(rect.contains_point(Point::new(50, 50)));
        assert!(rect.contains_point(Point::new(109, 219))); // x < right (10+100=110), y < bottom (20+200=220)
        
        // Points outside
        assert!(!rect.contains_point(Point::new(9, 20)));  // left edge
        assert!(!rect.contains_point(Point::new(10, 19)));  // top edge
        assert!(!rect.contains_point(Point::new(110, 20))); // right edge (exclusive)
        assert!(!rect.contains_point(Point::new(10, 220))); // bottom edge (exclusive)
    }

    #[test]
    fn test_rect_intersects_and_intersection() {
        let rect1 = Rect::from_coords(10, 20, 100, 80); // 10,20 to 110,100
        
        // Overlapping rectangle
        let rect2 = Rect::from_coords(50, 60, 100, 80); // 50,60 to 150,140
        assert!(rect1.intersects(&rect2));
        assert!(rect2.intersects(&rect1));
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection, Rect::from_coords(50, 60, 60, 40)); // x: max(10,50)=50, y: max(20,60)=60, w: min(110,150)-50 = 60, h: min(100,140)-60 = 40

        // Non-overlapping rectangle (to the right)
        let rect3 = Rect::from_coords(120, 20, 50, 80);
        assert!(!rect1.intersects(&rect3));
        assert!(!rect3.intersects(&rect1));
        assert!(rect1.intersection(&rect3).is_none());

        // Non-overlapping rectangle (below)
        let rect4 = Rect::from_coords(10, 110, 100, 80);
        assert!(!rect1.intersects(&rect4));
        assert!(!rect4.intersects(&rect1));
        assert!(rect1.intersection(&rect4).is_none());

        // Touching edges, no overlap
        let rect5 = Rect::from_coords(110, 20, 50, 80); // rect1.right = rect5.left
        assert!(!rect1.intersects(&rect5));
        assert!(rect1.intersection(&rect5).is_none());
    }

    #[test]
    fn test_rect_union() {
        let rect1 = Rect::from_coords(10, 20, 100, 80); // Ends at 110, 100
        let rect2 = Rect::from_coords(50, 60, 100, 80); // Ends at 150, 140
        let union_rect = rect1.union(&rect2);
        // x1 = min(10,50)=10, y1=min(20,60)=20
        // x2 = max(110,150)=150, y2=max(100,140)=140
        // width = 150-10=140, height = 140-20=120
        assert_eq!(union_rect, Rect::from_coords(10, 20, 140, 120));
    }

    #[test]
    fn test_rect_translated() {
        let rect = Rect::from_coords(10, 20, 100, 200);
        let translated_rect = rect.translated(5, 15);
        assert_eq!(translated_rect, Rect::from_coords(15, 35, 100, 200));
    }

    #[test]
    fn test_rect_scaled() {
        let rect = Rect::from_coords(10, 20, 100, 50);
        let scaled_rect = rect.scaled(2, 3);
        assert_eq!(scaled_rect, Rect::from_coords(20, 60, 200, 150));
    }

    #[test]
    fn test_rect_is_valid() {
        assert!(Rect::from_coords(0,0,10,10).is_valid());
        assert!(Rect::from_coords(0,0,0,0).is_valid()); // Size::is_valid allows 0
        assert!(!Rect::from_coords(0,0,-10,10).is_valid());
        assert!(!Rect::from_coords(0,0,10,-10).is_valid());
    }

    #[test]
    fn test_rect_zero_consts() {
        let rect_i: Rect<i32> = Default::default();
        assert_eq!(rect_i.position.x, 0);
        assert_eq!(rect_i.position.y, 0);
        assert_eq!(rect_i.size.width, 0);
        assert_eq!(rect_i.size.height, 0);
        assert_eq!(Rect::<i32>::ZERO_I32, rect_i);

        let rect_f: Rect<f32> = Default::default();
        assert_eq!(rect_f.position.x, 0.0);
        assert_eq!(rect_f.position.y, 0.0);
        assert_eq!(rect_f.size.width, 0.0);
        assert_eq!(rect_f.size.height, 0.0);
        assert_eq!(Rect::<f32>::ZERO_F32, rect_f);
    }

    // --- RectInt Tests ---
    #[test]
    fn test_rectint_new_zero() {
        let r = RectInt::new(10, 20, 100, 200);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 200);

        assert_eq!(RectInt::ZERO, RectInt { x:0, y:0, width:0, height:0});
        assert_eq!(RectInt::default(), RectInt::ZERO);
    }

    #[test]
    fn test_rectint_from_points() {
        let r1 = RectInt::from_points(Point::new(10,20), Point::new(110,120));
        assert_eq!(r1, RectInt::new(10,20,100,100));

        let r2 = RectInt::from_points(Point::new(110,120), Point::new(10,20)); // Flipped
        assert_eq!(r2, RectInt::new(10,20,100,100));
    }

    #[test]
    fn test_rectint_accessors() {
        let r = RectInt::new(10, 20, 100, 200);
        assert_eq!(r.top_left(), Point::new(10,20));
        assert_eq!(r.size(), Size::new(100,200));
        assert_eq!(r.left(), 10);
        assert_eq!(r.top(), 20);
        assert_eq!(r.right(), 110);
        assert_eq!(r.bottom(), 220);
    }
    
    #[test]
    fn test_rectint_contains_point() {
        let r = RectInt::new(10, 20, 100, 80); // 10,20 to 110,100
        assert!(r.contains_point_coords(10,20));
        assert!(r.contains_point_coords(109,99));
        assert!(!r.contains_point_coords(110,99)); // right edge
        assert!(!r.contains_point_coords(109,100)); // bottom edge
        assert!(r.contains_point(Point::new(50,50)));
        assert!(!r.contains_point(Point::new(0,0)));
    }

    #[test]
    fn test_rectint_intersects_intersection() {
        let r1 = RectInt::new(10,20,100,80); // ends 110, 100
        let r2 = RectInt::new(50,60,100,80); // ends 150, 140
        assert!(r1.intersects(r2));
        let intersect = r1.intersection(r2).unwrap();
        assert_eq!(intersect, RectInt::new(50,60,60,40));

        let r3 = RectInt::new(110,20,10,10); // Touching
        assert!(!r1.intersects(r3));
        assert!(r1.intersection(r3).is_none());
    }
    
    #[test]
    fn test_rectint_union() {
        let r1 = RectInt::new(10,20,100,80); // ends 110, 100
        let r2 = RectInt::new(0,0,50,60);   // ends 50, 60
        let u = r1.union(r2);
        // x = min(10,0) = 0, y = min(20,0) = 0
        // right = max(110,50) = 110, bottom = max(100,60) = 100
        // width = 110-0 = 110, height = 100-0 = 100
        assert_eq!(u, RectInt::new(0,0,110,100));
    }

    #[test]
    fn test_rectint_translated() {
        let r = RectInt::new(10,20,30,40);
        assert_eq!(r.translated(5, -5), RectInt::new(15,15,30,40));
    }

    #[test]
    fn test_rectint_inflate() {
        let r = RectInt::new(10,20,30,40);
        // x = 10-5=5, y=20-10=10
        // w = 30+2*5=40, h = 40+2*10=60
        assert_eq!(r.inflate(5,10), RectInt::new(5,10,40,60));

        let r_shrink = RectInt::new(10,20,30,40);
        // x = 10 - (-5) = 15, y = 20 - (-25) = 45
        // w = 30 + 2*(-5) = 20, h = 40 + 2*(-25) = -10 -> 0
        assert_eq!(r_shrink.inflate(-5,-25), RectInt::new(15,45,20,0));

        let r_fully_shrunk = RectInt::new(10,20,30,40);
         // w = 30 + 2*(-20) = -10 -> 0
        assert_eq!(r_fully_shrunk.inflate(-20,-20), RectInt::new(30,40,0,0));
    }

    #[test]
    fn test_rectint_is_empty() {
        assert!(RectInt::new(0,0,0,10).is_empty());
        assert!(RectInt::new(0,0,10,0).is_empty());
        assert!(!RectInt::new(0,0,1,1).is_empty());
    }
}
