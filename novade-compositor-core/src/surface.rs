//! Core data structures for managing Wayland surfaces and their state within the Novade compositor.
//!
//! This module defines the `Surface` struct, its attributes, states, and related helper
//! types like `Rectangle`, `DamageTracker`, and `Region`. It also includes the
//! `SurfaceRegistry` for managing all active surfaces.

use novade_buffer_manager::{BufferManager, BufferDetails, BufferId, ClientId, BufferFormat};
use std::sync::{Arc, Mutex};
use crate::subcompositor::{SubsurfaceState, SubsurfaceSyncMode}; // Used by SurfaceRole and Surface commit logic

/// Represents a transformation that can be applied to a buffer or surface.
///
/// This typically corresponds to `wl_output.transform` enum values, which describe
/// how image content should be transformed (e.g., rotated or flipped) before display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlOutputTransform {
    /// No transformation.
    Normal,
    /// Rotated 90 degrees counter-clockwise.
    Rotated90,
    /// Rotated 180 degrees.
    Rotated180,
    /// Rotated 270 degrees counter-clockwise.
    Rotated270,
    /// Flipped horizontally.
    Flipped,
    /// Flipped horizontally and then rotated 90 degrees counter-clockwise.
    FlippedRotated90,
    /// Flipped horizontally and then rotated 180 degrees (effectively a vertical flip).
    FlippedRotated180,
    /// Flipped horizontally and then rotated 270 degrees counter-clockwise.
    FlippedRotated270,
}

/// A simple 3x3 matrix placeholder for graphical transformations.
///
/// In a full compositor, this would likely be replaced by a more robust matrix math library
/// (e.g., `cgmath`, `nalgebra`, or `glam`). It is used here primarily as a placeholder
/// within `SurfaceAttributes` for future surface transformations.
/// The default state of this matrix is an identity matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3x3 {
    /// Matrix elements stored in row-major order:
    /// `[m11, m12, m13, m21, m22, m23, m31, m32, m33]`
    pub m: [f32; 9],
}

impl Default for Mat3x3 {
    /// Returns an identity matrix.
    fn default() -> Self {
        Self {
            m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Unique identifier for a `Surface`.
///
/// This newtype wrapper around `u64` provides type safety for surface identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceId(u64);

impl SurfaceId {
    /// Creates a new, unique `SurfaceId`.
    ///
    /// This implementation uses a global atomic counter to ensure uniqueness.
    /// This is suitable for single-threaded or multi-threaded contexts where surfaces
    /// might be created from different threads managing different clients.
    pub fn new_unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1); // Start IDs from 1.
        SurfaceId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Represents the lifecycle state of a `Surface`.
///
/// The state machine for a surface dictates what operations are valid and how
/// it interacts with buffer attachments, commits, and rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceState {
    /// The surface has been created but has no content defined or role assigned.
    /// It is not yet ready to be displayed.
    Created,
    /// A buffer has been attached via `wl_surface.attach`, and/or other state
    /// like damage or transformations has been set. This state is pending a `wl_surface.commit`.
    PendingBuffer,
    /// All pending state has been atomically applied via `wl_surface.commit`.
    /// The surface has defined content and attributes, and is ready for rendering or display.
    Committed,
    /// The surface is currently being processed by the rendering pipeline. (Conceptual)
    Rendering,
    /// The surface's content, as of its last commit, has been presented to the display. (Conceptual)
    Presented,
    /// The surface has been destroyed (e.g., client destroyed the `wl_surface` object).
    /// It should no longer be used or referenced.
    Destroyed,
}

/// Holds the collection of attributes that define a surface's appearance and behavior.
///
/// These attributes include its position (interpreted by its role), size, transformations,
/// opacity, and buffer-related settings. Surfaces maintain both a `pending_attributes`
/// (modified by client requests) and `current_attributes` (applied after a `commit`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceAttributes {
    /// Position of the surface. Interpretation depends on the surface role.
    /// For toplevels, this might be global coordinates. For subsurfaces, relative to parent.
    pub position: (i32, i32),
    /// Size of the surface in surface-local logical pixels.
    /// Derived from buffer dimensions and `buffer_scale` if a buffer is attached.
    pub size: (u32, u32),
    /// A 2D transformation matrix applied to the surface content. (Currently placeholder)
    pub transform: Mat3x3,
    /// Opacity of the surface, from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub alpha: f32,
    /// Scale factor applied to the attached buffer.
    pub buffer_scale: i32,
    /// Transformation applied to the attached buffer (e.g., rotation, flip).
    pub buffer_transform: WlOutputTransform,
    /// Offset for the attached buffer, in surface-local coordinates.
    /// Corresponds to `dx, dy` in `wl_surface.attach`.
    pub buffer_offset: (i32, i32),
}

impl Default for SurfaceAttributes {
    /// Provides default attributes for a new surface.
    /// Default values are:
    /// - position: `(0, 0)`
    /// - size: `(0, 0)` (meaning undefined until a buffer is attached or role sets it)
    /// - transform: Identity matrix
    /// - alpha: `1.0` (fully opaque)
    /// - buffer_scale: `1` (no scaling)
    /// - buffer_transform: `WlOutputTransform::Normal` (no transform)
    /// - buffer_offset: `(0, 0)`
    fn default() -> Self {
        Self {
            position: (0, 0),
            size: (0, 0),
            transform: Mat3x3::default(),
            alpha: 1.0,
            buffer_scale: 1,
            buffer_transform: WlOutputTransform::Normal,
            buffer_offset: (0, 0),
        }
    }
}

/// Defines a 2D rectangle with integer coordinates and dimensions.
///
/// Used for representing surface areas, damage regions, input regions, etc.
/// The coordinate system is typically top-left origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rectangle {
    /// X-coordinate of the top-left corner.
    pub x: i32,
    /// Y-coordinate of the top-left corner.
    pub y: i32,
    /// Width of the rectangle. Should be positive for a non-empty rectangle.
    pub width: i32,
    /// Height of the rectangle. Should be positive for a non-empty rectangle.
    pub height: i32,
}

impl Rectangle {
    /// Creates a new `Rectangle`.
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    /// Checks if the rectangle has zero or negative width or height, making it effectively empty.
    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    /// Calculates the area of the rectangle.
    /// Returns 0 if the rectangle `is_empty()`.
    pub fn area(&self) -> i32 {
        if self.is_empty() {
            0
        } else {
            self.width * self.height
        }
    }

    /// Checks if this rectangle intersects with another rectangle.
    /// Empty rectangles do not intersect.
    pub fn intersects(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    /// Computes the smallest rectangle that encloses both this rectangle and another.
    /// If one rectangle is empty, the other is returned. If both are empty, an empty rectangle results.
    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);

        Self { x: x1, y: y1, width: x2 - x1, height: y2 - y1 }
    }

    /// Computes the intersection of this rectangle with another.
    /// Returns an empty rectangle (width/height <= 0) if they do not intersect.
    pub fn intersection(&self, other: &Self) -> Self {
        if !self.intersects(other) {
            return Self { x: 0, y: 0, width: 0, height: 0 };
        }

        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        Self { x: x1, y: y1, width: x2 - x1, height: y2 - y1 }
    }

    /// Clips this rectangle to the bounds of another `clipping_rect`.
    /// This is equivalent to their intersection.
    pub fn clipped_to(&self, clipping_rect: &Self) -> Self {
        self.intersection(clipping_rect)
    }

    /// Returns a new rectangle translated by `(dx, dy)`.
    pub fn translate(&self, dx: i32, dy: i32) -> Self {
        Self { x: self.x + dx, y: self.y + dy, width: self.width, height: self.height }
    }

    /// Scales the rectangle's position and dimensions by an integer factor.
    /// Uses integer division, consistent with `wl_surface.buffer_scale` behavior.
    /// A scale factor of 0 results in an empty rectangle at (0,0).
    pub fn scale(&self, factor: i32) -> Self {
        if factor == 0 { return Self::new(0,0,0,0); }
        if factor == 1 { return *self; }
        Self {
            x: self.x / factor,
            y: self.y / factor,
            width: self.width / factor,
            height: self.height / factor,
        }
    }

    /// Transforms a single point according to `WlOutputTransform`. (Internal helper)
    /// `s_width` and `s_height` are the dimensions of the space *before* this transform.
    #[allow(dead_code)]
    fn transform_point(x: i32, y: i32, transform: WlOutputTransform, s_width: i32, s_height: i32) -> (i32, i32) {
        match transform {
            WlOutputTransform::Normal => (x, y),
            WlOutputTransform::Rotated90 => (s_height - y - 1, x),
            // TODO: Implement other point transformations if needed for more complex rect transforms.
            _ => (x,y)
        }
    }

    /// Transforms the rectangle according to a `WlOutputTransform`.
    /// `s_width` and `s_height` are the dimensions of the coordinate space *before* this transform
    /// is applied (e.g., for buffer damage, this would be the buffer's scaled dimensions).
    pub fn transform(&self, transform: WlOutputTransform, s_width: i32, s_height: i32) -> Self {
        let mut new_x = self.x;
        let mut new_y = self.y;
        let mut new_width = self.width;
        let mut new_height = self.height;

        match transform {
            WlOutputTransform::Normal => { /* No change */ }
            WlOutputTransform::Rotated90 => {
                new_x = self.y;
                new_y = s_width - (self.x + self.width);
                new_width = self.height;
                new_height = self.width;
            }
            WlOutputTransform::Rotated180 => {
                new_x = s_width - (self.x + self.width);
                new_y = s_height - (self.y + self.height);
            }
            WlOutputTransform::Rotated270 => {
                new_x = s_height - (self.y + self.height);
                new_y = self.x;
                new_width = self.height;
                new_height = self.width;
            }
            WlOutputTransform::Flipped => {
                new_x = s_width - (self.x + self.width);
            }
            WlOutputTransform::FlippedRotated90 => {
                new_x = self.y;
                new_y = self.x;
                new_width = self.height;
                new_height = self.width;
            }
            WlOutputTransform::FlippedRotated180 => { // Effectively a vertical flip
                new_x = self.x;
                new_y = s_height - (self.y + self.height);
            }
            WlOutputTransform::FlippedRotated270 => {
                new_x = s_height - (self.y + self.height);
                new_y = s_width - (self.x + self.width);
                new_width = self.height;
                new_height = self.width;
            }
        }
        Self::new(new_x, new_y, new_width, new_height)
    }
}

/// Maximum number of rectangles to maintain in `DamageTracker::current_damage`
/// before falling back to a single rectangle covering the whole surface.
const MAX_DAMAGE_RECTS: usize = 100;

/// Tracks damage regions for a surface.
///
/// Damage can be reported in buffer coordinates (relative to the attached buffer)
/// or surface coordinates (relative to the surface's top-left, legacy).
/// On commit, pending damage is transformed, merged, and clipped to become current damage.
#[derive(Debug, Clone)]
pub struct DamageTracker {
    /// Damage reported in buffer coordinates via `wl_surface.damage_buffer`.
    pending_damage_buffer: Vec<Rectangle>,
    /// Damage reported in surface-local coordinates via `wl_surface.damage` (legacy).
    pending_damage_surface: Vec<Rectangle>,
    /// Damage that has been committed, in surface-local coordinates, clipped to surface bounds.
    /// This list should ideally contain disjoint rectangles.
    current_damage: Vec<Rectangle>,
    /// Counter used for buffer age damage optimization. Incremented when damage is added.
    /// Reset when new content (buffer) is committed to the surface.
    damage_age: u32,
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DamageTracker {
    /// Creates a new, empty `DamageTracker`.
    pub fn new() -> Self {
        Self {
            pending_damage_buffer: Vec::new(),
            pending_damage_surface: Vec::new(),
            current_damage: Vec::new(),
            damage_age: 0,
        }
    }

    /// Merges overlapping or adjacent rectangles in a list to simplify the region.
    /// This is a helper function for `commit_pending_damage`.
    /// The current implementation is a naive O(N^2) pass that might require multiple
    /// iterations for complex cases if not restarted from the beginning on merge.
    fn merge_rectangles(rects: &mut Vec<Rectangle>) {
        if rects.len() <= 1 {
            return;
        }
        let mut i = 0;
        while i < rects.len() {
            let mut j = i + 1;
            let mut merged_current = false;
            while j < rects.len() {
                let r1 = rects[i];
                let r2 = rects[j];
                // Check for intersection or exact adjacency.
                if r1.intersects(&r2) ||
                   (r1.x == r2.x + r2.width && r1.y == r2.y && r1.height == r2.height && r1.width > 0 && r2.width > 0) ||
                   (r1.x + r1.width == r2.x && r1.y == r2.y && r1.height == r2.height && r1.width > 0 && r2.width > 0) ||
                   (r1.y == r2.y + r2.height && r1.x == r2.x && r1.width == r2.width && r1.height > 0 && r2.height > 0) ||
                   (r1.y + r1.height == r2.y && r1.x == r2.x && r1.width == r2.width && r1.height > 0 && r2.height > 0)
                {
                    rects[i] = r1.union(&r2);
                    rects.remove(j); // This shifts subsequent elements, j does not need increment.
                    merged_current = true;
                } else {
                    j += 1;
                }
            }
            if merged_current {
                i = 0; // Restart scan as rects[i] changed and might merge with earlier ones.
            } else {
                i += 1;
            }
        }
    }

    /// Adds damage in buffer coordinates (from `wl_surface.damage_buffer`).
    /// Increments `damage_age`.
    pub fn add_damage_buffer(&mut self, rect: Rectangle) {
        if rect.is_empty() { return; }
        self.pending_damage_buffer.push(rect);
        self.damage_age += 1;
    }

    /// Adds damage in surface-local coordinates (from `wl_surface.damage`).
    /// Increments `damage_age`.
    pub fn add_damage_surface(&mut self, rect: Rectangle) {
        if rect.is_empty() { return; }
        self.pending_damage_surface.push(rect);
        self.damage_age += 1;
    }

    /// Transforms a list of damage rectangles (either buffer or surface coordinates)
    /// into clipped, surface-local coordinates.
    fn transform_and_clip_damage_list(
        damage_list: &mut Vec<Rectangle>,
        attributes: &SurfaceAttributes,
        is_buffer_damage: bool,
        surface_boundary: &Rectangle,
    ) {
        let mut transformed_damage: Vec<Rectangle> = Vec::new();
        for rect in damage_list.iter() {
            if rect.is_empty() { continue; }

            let transformed_rect = if is_buffer_damage {
                if attributes.buffer_scale <= 0 {
                    Rectangle::new(0,0,0,0) // Invalid scale results in empty rect
                } else {
                    // 1. Scale from buffer to surface logical pixels
                    let scaled_rect = rect.scale(attributes.buffer_scale);
                    // 2. Transform according to wl_output transform (e.g., rotation)
                    //    The s_width/s_height for this are the surface's dimensions *after* scaling
                    //    but *before* this wl_output_transform. These are what surface_boundary represents.
                    scaled_rect.transform(attributes.buffer_transform, surface_boundary.width, surface_boundary.height)
                }
            } else {
                // Damage is already in surface coordinates, no transform needed other than clipping.
                *rect
            };

            if !transformed_rect.is_empty() {
                 let clipped_rect = transformed_rect.clipped_to(surface_boundary);
                 if !clipped_rect.is_empty() {
                    transformed_damage.push(clipped_rect);
                 }
            }
        }
        *damage_list = transformed_damage;
    }

    /// Processes pending damage, transforming it to surface coordinates, merging,
    /// clipping, and then setting it as the `current_damage`.
    ///
    /// This is called when a surface's state is committed.
    ///
    /// # Arguments
    /// * `attributes`: The `SurfaceAttributes` that are now current for the surface
    ///   (used for scaling, transformation, and clipping bounds).
    /// * `new_buffer_committed`: True if a new buffer (or NULL buffer state) was just committed,
    ///   which typically resets `damage_age`.
    pub fn commit_pending_damage(&mut self, attributes: &SurfaceAttributes, new_buffer_committed: bool) {
        let surface_boundary = Rectangle::new(0, 0, attributes.size.0 as i32, attributes.size.1 as i32);

        // If surface has no size, damage is effectively empty or undefined.
        if surface_boundary.is_empty() && !(self.pending_damage_buffer.is_empty() && self.pending_damage_surface.is_empty()) {
            self.current_damage.clear();
            self.pending_damage_buffer.clear();
            self.pending_damage_surface.clear();
            self.damage_age = 0; // Reset age as damage context is lost.
            return;
        }

        // Transform pending_damage_buffer to surface coordinates and clip.
        DamageTracker::transform_and_clip_damage_list(&mut self.pending_damage_buffer, attributes, true, &surface_boundary);

        // Clip pending_damage_surface (already in surface coordinates).
        DamageTracker::transform_and_clip_damage_list(&mut self.pending_damage_surface, attributes, false, &surface_boundary);

        // Combine all processed damage.
        let mut combined_damage: Vec<Rectangle> = self.pending_damage_buffer.drain(..).collect();
        combined_damage.extend(self.pending_damage_surface.drain(..));

        // Merge overlapping/adjacent rectangles in the combined list.
        DamageTracker::merge_rectangles(&mut combined_damage);

        // Handle damage region overflow: if too many rects or too much area, damage whole surface.
        let total_surface_area = surface_boundary.area();
        let mut current_total_damage_area = 0;
        for rect in &combined_damage {
            current_total_damage_area += rect.area();
        }

        if combined_damage.len() > MAX_DAMAGE_RECTS ||
           (total_surface_area > 0 && combined_damage.len() > 1 && // Avoid full damage for single small rect
            current_total_damage_area as f32 / total_surface_area as f32 > 0.75) {
            self.current_damage = vec![surface_boundary]; // Fallback to full surface damage
        } else {
            self.current_damage = combined_damage;
        }

        // Reset damage_age if a new buffer was committed.
        if new_buffer_committed {
            self.damage_age = 0;
        }
        // If no new buffer, damage_age (incremented by add_damage_*) persists.
    }

    /// Returns a copy of the `current_damage` list, with all rectangles
    /// additionally clipped to the provided `surface_size`.
    /// This is what a renderer would typically query.
    pub fn get_current_damage_clipped(&self, surface_size: (u32, u32)) -> Vec<Rectangle> {
        let surface_boundary = Rectangle::new(0, 0, surface_size.0 as i32, surface_size.1 as i32);
        if surface_boundary.is_empty() { return Vec::new(); } // No damage if surface has no size.
        self.current_damage.iter()
            .map(|rect| rect.clipped_to(&surface_boundary))
            .filter(|rect| !rect.is_empty()) // Filter out any rects that become empty after clipping
            .collect()
    }

    /// Clears all pending and current damage, and resets `damage_age`.
    pub fn clear_all_damage(&mut self) {
        self.pending_damage_buffer.clear();
        self.pending_damage_surface.clear();
        self.current_damage.clear();
        self.damage_age = 0;
    }
}

use crate::subcompositor::{SubsurfaceState, SubsurfaceSyncMode};

/// Defines the role of a surface (e.g., toplevel, subsurface, cursor).
/// Each variant can hold role-specific state.
#[derive(Debug, Clone)]
pub enum SurfaceRole {
    /// A regular toplevel window (e.g., `xdg_toplevel`).
    /// (Placeholder: would contain specific toplevel state).
    Toplevel,
    /// A subsurface, part of a surface tree (`wl_subsurface`).
    Subsurface(SubsurfaceState),
    /// A hardware or software cursor image.
    /// (Placeholder: would contain specific cursor state).
    Cursor,
    /// A drag-and-drop icon.
    /// (Placeholder: would contain specific DND icon state).
    DragIcon,
}

/// Represents a Wayland frame callback (`wl_callback`).
/// Clients request these to be notified when a frame is rendered and presented.
#[derive(Debug, Clone)]
pub struct WlCallback {
    /// The Wayland resource ID of the `wl_callback` object.
    pub id: u64,
}

/// Represents a Wayland region (`wl_region`).
///
/// A region is a collection of rectangles. In this simplified version, it's directly
/// used for opaque and input regions on a surface. A more complete `wl_region`
/// implementation would have its own module and registry.
/// See `novade_compositor_core::region` for a more dedicated region implementation.
#[derive(Debug, Clone, Default)]
pub struct Region {
    /// The list of rectangles defining this region.
    /// For surface opaque/input regions, these are in surface-local coordinates.
    pub rectangles: Vec<Rectangle>,
}

impl Region {
    /// Creates a new, empty `Region`.
    pub fn new() -> Self {
        Self { rectangles: Vec::new() }
    }
}

/// Represents a Wayland surface (`wl_surface`).
///
/// This is a central structure in the compositor, holding all state related to
/// a client's drawable area. It manages pending and current attributes, attached buffers,
/// damage tracking, frame callbacks, and its role within the compositor (e.g., toplevel, subsurface).
pub struct Surface {
    /// Unique identifier for this surface.
    pub id: SurfaceId,
    /// Current lifecycle state of the surface.
    pub current_state: SurfaceState,
    /// Attributes pending to be applied on the next commit.
    pub pending_attributes: SurfaceAttributes,
    /// Currently active attributes of the surface.
    pub current_attributes: SurfaceAttributes,
    /// Tracks damage regions for this surface.
    pub damage_tracker: DamageTracker,
    /// The role assigned to this surface (e.g., Toplevel, Subsurface).
    pub role: Option<SurfaceRole>,
    /// Buffer attached by the client, pending commit.
    pub pending_buffer: Option<Arc<Mutex<BufferDetails>>>,
    /// The buffer whose content is currently displayed or ready to be displayed.
    pub current_buffer: Option<Arc<Mutex<BufferDetails>>>,
    /// List of pending frame callbacks requested by the client.
    pub frame_callbacks: Vec<WlCallback>,
    /// The opaque region of the surface, in surface-local coordinates.
    /// `None` means the entire surface is considered transparent beyond its buffer content.
    /// `Some(region)` where region is empty means fully transparent.
    /// `Some(region)` with rectangles means those parts are opaque.
    /// Wayland spec: "Initially the opaque region is empty."
    pub opaque_region: Option<Region>,
    /// The input region of the surface, in surface-local coordinates.
    /// Defines where the surface accepts input. `None` implies the entire surface accepts input
    /// if it's mapped and has dimensions. An empty `Region` (Some(Region{rectangles:[]}))
    /// would mean no part of the surface accepts input.
    /// Wayland spec: "Initially the input region covers the entire surface."
    pub input_region: Option<Region>,

    // Subsurface-specific fields
    /// List of `SurfaceId`s for direct children if this surface is a parent.
    pub children: Vec<SurfaceId>,
    /// `SurfaceId` of the parent if this surface is a subsurface.
    pub parent: Option<SurfaceId>,

    // Cache for synchronized subsurface state
    /// Cached buffer for a synchronized subsurface, applied on parent commit.
    cached_pending_buffer: Option<Arc<Mutex<BufferDetails>>>,
    /// Cached attributes for a synchronized subsurface, applied on parent commit.
    cached_pending_attributes: Option<SurfaceAttributes>,
}

impl Surface {
    /// Creates a new `Surface` with default attributes and state.
    ///
    /// The surface is initialized in the `SurfaceState::Created` state,
    /// with no attached buffer or specific role. Opaque region is initially empty.
    /// Input region is initially `None`, implying the whole surface accepts input once sized.
    pub fn new() -> Self {
        Self {
            id: SurfaceId::new_unique(),
            current_state: SurfaceState::Created,
            pending_attributes: SurfaceAttributes::default(),
            current_attributes: SurfaceAttributes::default(),
            damage_tracker: DamageTracker::new(),
            role: None,
            pending_buffer: None,
            current_buffer: None,
            frame_callbacks: Vec::new(),
            opaque_region: Some(Region::new()), // Initially empty
            input_region: None,  // Initially covers whole surface (represented by None)
            children: Vec::new(),
            parent: None,
            cached_pending_buffer: None,
            cached_pending_attributes: None,
        }
    }
}

/// Errors that can occur when attaching a buffer to a surface.
/// Corresponds to `wl_surface.attach` error conditions.
#[derive(Debug)]
pub enum BufferAttachError {
    /// The provided `BufferId` was not found in the `BufferManager`.
    BufferNotFound,
    /// The client attempting to attach the buffer does not own it.
    /// (This is a compositor policy, not strictly a Wayland protocol error for `attach` itself).
    ClientMismatch,
    /// The buffer has invalid dimensions (e.g., zero width or height).
    /// This can also map to `wl_surface.error.invalid_size` if checked at attach time.
    InvalidBufferSize,
    /// The surface is in a state that does not allow buffer attachment (e.g., `Destroyed`).
    InvalidState,
    /// A non-zero offset was provided for `dx` or `dy` with `wl_surface` version 5+
    /// when attaching a non-NULL buffer. Corresponds to `wl_surface.error.invalid_offset`.
    InvalidBufferOffset,
}

/// Errors that can occur during a `Surface::commit` operation.
#[derive(Debug)]
pub enum CommitError {
    /// The surface is in a state that does not allow committing (e.g., `Destroyed`, or
    /// potentially `Rendering` or `Presented` depending on compositor state machine).
    InvalidState,
    /// The pending buffer state is invalid (e.g., its dimensions are not divisible by the
    /// pending `buffer_scale` factor). Corresponds to `wl_surface.error.invalid_size`.
    InvalidBufferSize,
}

/// Errors related to surface state transitions.
#[derive(Debug)]
pub enum SurfaceTransitionError {
    /// The requested state transition is not allowed (e.g., from `Destroyed` to `Created`).
    InvalidTransition,
}

impl Surface {
    /// Releases the current buffer associated with the surface, if any. (Internal helper)
    /// This involves decrementing its reference count via the `BufferManager`.
    fn release_current_buffer(&mut self, buffer_manager: &mut BufferManager) {
        if let Some(buffer_arc) = self.current_buffer.take() {
            let buffer_id = buffer_arc.lock().unwrap().id;
            buffer_manager.release_buffer(buffer_id);
        }
    }

    /// Releases the pending buffer associated with the surface, if any. (Internal helper)
    /// This involves decrementing its reference count via the `BufferManager`.
    fn release_pending_buffer(&mut self, buffer_manager: &mut BufferManager) {
        if let Some(buffer_arc) = self.pending_buffer.take() {
            let buffer_id = buffer_arc.lock().unwrap().id;
            buffer_manager.release_buffer(buffer_id);
        }
    }

    /// Adds damage to the surface in buffer coordinates.
    /// Corresponds to `wl_surface.damage_buffer`.
    /// The damage rectangle is clipped to the bounds of the currently pending or current buffer.
    ///
    /// # Arguments
    /// * `x`, `y`, `width`, `height`: The rectangle defining the damaged area, in buffer coordinates.
    ///   Non-positive `width` or `height` will result in the damage being ignored.
    pub fn damage_buffer(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.current_state == SurfaceState::Destroyed { return; }
        // Wayland spec: width and height must be positive.
        // TODO: Consider sending a wl_surface protocol error `invalid_damage` if width/height are non-positive.
        let rect = Rectangle::new(x, y, width, height);
        if rect.is_empty() { return; } // Silently ignore empty or invalid damage rects for now.

        // Determine dimensions of the buffer this damage applies to.
        // It applies to the pending buffer if one exists, otherwise the current one.
        let mut opt_buffer_dims: Option<(i32,i32)> = None;
        if let Some(pending_ref) = &self.pending_buffer {
             let details = pending_ref.lock().unwrap();
             opt_buffer_dims = Some((details.width as i32, details.height as i32));
        } else if let Some(current_ref) = &self.current_buffer {
            let details = current_ref.lock().unwrap();
            opt_buffer_dims = Some((details.width as i32, details.height as i32));
        }

        if let Some(buffer_dims) = opt_buffer_dims {
            if buffer_dims.0 > 0 && buffer_dims.1 > 0 { // Only if buffer has actual dimensions
                let buffer_bounds = Rectangle::new(0, 0, buffer_dims.0, buffer_dims.1);
                let clipped_rect = rect.clipped_to(&buffer_bounds);
                if !clipped_rect.is_empty() {
                    self.damage_tracker.add_damage_buffer(clipped_rect);
                }
            }
        }
        // If no buffer is attached, wl_surface.damage_buffer has no effect according to some interpretations,
        // or might be an error. Silently ignoring seems safest if buffer dimensions are unknown/zero.
    }

    /// Adds damage to the surface in surface-local coordinates.
    /// Corresponds to the legacy `wl_surface.damage` request.
    /// The damage rectangle is clipped to the current surface bounds.
    ///
    /// # Arguments
    /// * `x`, `y`, `width`, `height`: The rectangle defining the damaged area, in surface-local coordinates.
    ///   Non-positive `width` or `height` will result in the damage being ignored.
    pub fn damage_surface(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.current_state == SurfaceState::Destroyed { return; }
        // Wayland spec: width and height must be positive.
        // TODO: Consider sending a wl_surface protocol error `invalid_damage` if width/height are non-positive.
        let rect = Rectangle::new(x, y, width, height);
        if rect.is_empty() { return; } // Silently ignore.

        // Clip to current (not pending) surface attributes.
        let surface_bounds = Rectangle::new(0, 0, self.current_attributes.size.0 as i32, self.current_attributes.size.1 as i32);
         if surface_bounds.is_empty() && (self.current_attributes.size.0 !=0 || self.current_attributes.size.1 !=0) {
            // This case (e.g. size is (positive, 0)) means surface_bounds is empty. Damage is clipped to nothing.
            return;
        }
        let clipped_rect = rect.clipped_to(&surface_bounds);
        if !clipped_rect.is_empty() {
            self.damage_tracker.add_damage_surface(clipped_rect);
        }
    }

    /// Attaches a buffer to the surface. Corresponds to `wl_surface.attach`.
    ///
    /// The `buffer_arc_opt` being `None` signifies attaching a NULL buffer, which means
    /// the surface has no content.
    /// The `x_offset` and `y_offset` are relative to the top-left corner of the surface.
    /// For `wl_surface` version 5+, these offsets must be zero if a non-NULL buffer is provided.
    ///
    /// # Arguments
    /// * `buffer_manager`: Reference to the `BufferManager` for ref-counting.
    /// * `buffer_arc_opt`: An `Option<Arc<Mutex<BufferDetails>>>` for the buffer to attach.
    ///                     `None` for attaching a NULL buffer.
    /// * `client_id`: The `ClientId` of the client making the request (for ownership validation).
    /// * `x_offset`, `y_offset`: Offsets for the buffer.
    ///
    /// # Returns
    /// `Ok(())` on successful attachment.
    /// `Err(BufferAttachError)` if validation fails (e.g., invalid state, offset, or buffer).
    pub fn attach_buffer(
        &mut self,
        buffer_manager: &mut BufferManager,
        buffer_arc_opt: Option<Arc<Mutex<BufferDetails>>>,
        client_id: ClientId,
        x_offset: i32,
        y_offset: i32,
    ) -> Result<(), BufferAttachError> {
        if self.current_state == SurfaceState::Destroyed {
            return Err(BufferAttachError::InvalidState);
        }
        // Wayland spec: wl_surface.attach (version >= 5) - dx and dy must be 0 if buffer is not NULL.
        // If buffer_arc_opt is None (NULL buffer), offsets are ignored by the protocol.
        if buffer_arc_opt.is_some() && (x_offset != 0 || y_offset != 0) {
            // TODO: This check should ideally be conditional on the client's bound wl_surface version.
            // Assuming version >= 5 for now as per the subtask requirements.
            return Err(BufferAttachError::InvalidBufferOffset);
        }

        // Release any previously pending buffer.
        self.release_pending_buffer(buffer_manager);
        self.pending_buffer = None; // Explicitly clear.

        if let Some(buffer_arc) = buffer_arc_opt {
            // A non-NULL buffer is being attached.
            { // Scope for buffer_details lock
                let buffer_details = buffer_arc.lock().unwrap();
                // Validate client ownership (optional, depends on compositor policy).
                if let Some(owner_id) = buffer_details.client_owner_id {
                    if owner_id != client_id { return Err(BufferAttachError::ClientMismatch); }
                }
                // Validate buffer size (basic check for non-zero).
                if buffer_details.width == 0 || buffer_details.height == 0 {
                    return Err(BufferAttachError::InvalidBufferSize);
                }
            } // Lock released.

            buffer_arc.lock().unwrap().increment_ref_count(); // New reference from this surface.
            self.pending_buffer = Some(buffer_arc.clone());

            // Surface size is determined by the attached buffer dimensions.
            let (buffer_width, buffer_height) = {
                let details = buffer_arc.lock().unwrap();
                (details.width, details.height)
            };
            self.pending_attributes.size = (buffer_width, buffer_height);
        } else {
            // Attaching a NULL buffer. self.pending_buffer is already None.
            // The surface size (pending_attributes.size) is NOT changed by attach(NULL).
            self.pending_buffer = None;
        }

        self.pending_attributes.buffer_offset = (x_offset, y_offset);

        if self.current_state != SurfaceState::PendingBuffer {
            let _ = self.transition_to(SurfaceState::PendingBuffer);
        }
        Ok(())
    }

    /// Commits the pending state of the surface, making it current.
    /// Corresponds to `wl_surface.commit`.
    ///
    /// This involves:
    /// 1. Validating the pending state (e.g., buffer size vs. scale).
    /// 2. If the surface is a synchronized subsurface, its state is cached for the parent's commit.
    /// 3. Otherwise, pending attributes and buffer are made current.
    /// 4. Pending damage is processed and becomes current damage.
    /// 5. The surface state transitions to `Committed`.
    ///
    /// # Arguments
    /// * `buffer_manager`: Reference to the `BufferManager` for buffer ref-counting.
    ///
    /// # Returns
    /// `Ok(())` if the commit is successful.
    /// `Err(CommitError)` if validation fails (e.g., invalid state or buffer properties).
    pub fn commit(&mut self, buffer_manager: &mut BufferManager) -> Result<(), CommitError> {
        if self.current_state == SurfaceState::Destroyed {
            return Err(CommitError::InvalidState);
        }
        // Only allow commit from these states. Others (like Rendering) might have different rules.
        if !matches!(self.current_state, SurfaceState::Created | SurfaceState::PendingBuffer | SurfaceState::Committed) {
            return Err(CommitError::InvalidState);
        }

        // --- Validation Phase ---
        // If a non-NULL buffer is pending, validate its properties against pending attributes.
        if let Some(pending_buffer_arc) = &self.pending_buffer {
            let pending_buffer_details = pending_buffer_arc.lock().unwrap();
            if pending_buffer_details.width == 0 || pending_buffer_details.height == 0 {
                return Err(CommitError::InvalidBufferSize); // Should ideally be caught at attach.
            }
            let scale = self.pending_attributes.buffer_scale;
            if scale <= 0 { // Scale must be positive.
                return Err(CommitError::InvalidBufferSize);
            }
            // Buffer dimensions must be an integer multiple of scale for wl_surface.
            if pending_buffer_details.width % (scale as u32) != 0 ||
               pending_buffer_details.height % (scale as u32) != 0 {
                return Err(CommitError::InvalidBufferSize); // This is wl_surface.error.invalid_size.
            }
            // Note: Buffer offset validation (dx, dy == 0 for v5+) is done in attach_buffer.
        }

        // --- Synchronized Subsurface Handling ---
        let mut is_currently_synchronized_subsurface = false;
        if let Some(SurfaceRole::Subsurface(ref mut subsurface_state)) = self.role {
            if subsurface_state.sync_mode == SubsurfaceSyncMode::Synchronized {
                is_currently_synchronized_subsurface = true;
                subsurface_state.needs_apply_on_parent_commit = true;

                // Cache the pending state (buffer and attributes).
                // Damage remains in `damage_tracker.pending_damage_*` and will be processed by `apply_cached_state_from_sync`.
                if let Some(pending_buffer_arc_real) = &self.pending_buffer {
                    pending_buffer_arc_real.lock().unwrap().increment_ref_count(); // Cache holds a new ref.
                    self.cached_pending_buffer = Some(pending_buffer_arc_real.clone());
                } else {
                    self.cached_pending_buffer = None; // Handles attach(NULL) case.
                }
                self.cached_pending_attributes = Some(self.pending_attributes);

                // The original self.pending_buffer is now "consumed" into the cache.
                self.pending_buffer.take(); // This drops the Arc formerly in self.pending_buffer, dec-ref-counting.

                // Surface state becomes Committed, indicating its state is latched for parent.
                let _ = self.transition_to(SurfaceState::Committed);
                return Ok(()); // Commit for synchronized subsurface ends here.
            }
        }

        // --- Regular Commit Logic (for Toplevels or Desynchronized Subsurfaces) ---
        let mut new_content_committed = false;
        // `new_content_committed` is true if an attach operation (to Some or None buffer) occurred,
        // indicated by being in `PendingBuffer` state.
        if self.current_state == SurfaceState::PendingBuffer {
            new_content_committed = true;
        }

        // Apply pending attributes to current attributes.
        self.current_attributes = self.pending_attributes;

        // Handle buffer transition if an attach operation occurred.
        if new_content_committed {
            self.release_current_buffer(buffer_manager); // Release old current_buffer.
            self.current_buffer = self.pending_buffer.take(); // Move new (possibly None) buffer to current.
        }
        // If no attach call was made (i.e., state wasn't PendingBuffer), current_buffer remains unchanged.
        // self.pending_buffer should be None at this point if it wasn't already taken.

        // Handle a desynchronized subsurface that might have cached state.
        // This occurs if it was: sync (commit) -> set_desync -> commit.
        let mut applied_cache_for_desync = false;
        if !is_currently_synchronized_subsurface { // Only if not currently sync (i.e., is toplevel or desync sub)
            if let Some(SurfaceRole::Subsurface(ref mut subsurface_state)) = self.role {
                 if subsurface_state.sync_mode == SubsurfaceSyncMode::Desynchronized &&
                    subsurface_state.needs_apply_on_parent_commit { // Was sync, cached, now desync.
                     self.apply_cached_state_from_sync(buffer_manager)?;
                     applied_cache_for_desync = true; // Damage was handled by apply_cached_state.
                 }
            }
        }

        // If cached state was applied for a desync transition, that process handled damage.
        // Otherwise, do standard damage processing for this commit.
        if !applied_cache_for_desync {
            self.damage_tracker.commit_pending_damage(&self.current_attributes, new_content_committed);
        }

        let _ = self.transition_to(SurfaceState::Committed);
        Ok(())
    }

    /// Applies cached state for a synchronized subsurface.
    ///
    /// This is typically called by the compositor after the parent surface has committed
    /// and its state is being applied. It moves cached buffer and attributes to current,
    /// and processes pending damage. Also used when a surface transitions from sync to desync.
    ///
    /// # Arguments
    /// * `buffer_manager`: Reference to `BufferManager` for buffer operations.
    ///
    /// # Returns
    /// `Ok(())` on success, or `CommitError` if the surface is in an invalid state.
    pub fn apply_cached_state_from_sync(&mut self, buffer_manager: &mut BufferManager) -> Result<(), CommitError> {
        if self.current_state == SurfaceState::Destroyed {
            return Err(CommitError::InvalidState);
        }

        let mut needs_apply_flag_was_set = false;
        if let Some(SurfaceRole::Subsurface(ref mut subsurface_state)) = self.role {
            if subsurface_state.needs_apply_on_parent_commit {
                needs_apply_flag_was_set = true;
                subsurface_state.needs_apply_on_parent_commit = false; // Clear the flag immediately.
            } else {
                // No pending cached state to apply.
                return Ok(());
            }
        } else {
            // Not a subsurface, this method is inappropriate unless it's part of a desync transition.
            // The desync transition case in commit() calls this, so it might be a non-subsurface
            // if role was cleared before this. However, needs_apply_on_parent_commit should only be
            // true for subsurfaces.
            return Err(CommitError::InvalidState);
        }

        if !needs_apply_flag_was_set {
            // Should have returned above if flag wasn't set. Defensive.
            return Ok(());
        }

        // Apply cached attributes.
        if let Some(cached_attrs) = self.cached_pending_attributes.take() {
            self.current_attributes = cached_attrs;
        }
        // If no cached_attrs, current_attributes remains as it was (potentially from a previous commit).

        // Apply cached buffer.
        self.release_current_buffer(buffer_manager); // Release any existing current buffer.
        self.current_buffer = self.cached_pending_buffer.take();
        // The ref count for the buffer in cached_pending_buffer was incremented when it was cached.
        // Moving the Arc to current_buffer maintains that ref count.

        // Process damage from DamageTracker's pending lists.
        // `new_content_was_cached` indicates if the cache application represents a change in content
        // (e.g. a new buffer or significant attribute change that would reset damage age).
        let new_content_was_cached = self.cached_pending_attributes.is_some(); // True if attributes were cached.

        self.damage_tracker.commit_pending_damage(&self.current_attributes, new_content_was_cached);

        // The surface state should already be Committed (set by its own commit call when it cached state).
        // This function primarily applies the data, not transitions state further unless needed.
        Ok(())
    }

    /// Prepares the surface for destruction.
    ///
    /// This involves releasing its buffers, removing itself from its parent's children list,
    /// and orphaning its own children (subsurfaces). It also clears frame callbacks and
    /// sets the surface state to `Destroyed`.
    ///
    /// # Arguments
    /// * `buffer_manager`: For releasing attached buffers.
    /// * `surface_registry_accessor`: To access parent and child surface data from the registry.
    pub fn prepare_for_destruction(
        &mut self,
        buffer_manager: &mut BufferManager,
        surface_registry_accessor: &impl surface_registry::SurfaceRegistryAccessor,
    ) {
        // Release all held buffers
        if let Some(buffer_arc) = self.pending_buffer.take() {
            buffer_manager.release_buffer(buffer_arc.lock().unwrap().id);
        }
        if let Some(buffer_arc) = self.current_buffer.take() {
            buffer_manager.release_buffer(buffer_arc.lock().unwrap().id);
        }
        if let Some(cached_buffer_arc) = self.cached_pending_buffer.take() {
             buffer_manager.release_buffer(cached_buffer_arc.lock().unwrap().id);
        }

        // Remove self from parent's children list
        if let Some(parent_id) = self.parent.take() { // .take() also clears self.parent
            if let Some(parent_surface_arc) = surface_registry_accessor.get_surface(parent_id) {
                if let Ok(mut parent_surface) = parent_surface_arc.lock() {
                    parent_surface.children.retain(|child_id| *child_id != self.id);
                }
                // If parent lock fails or parent not found, we've at least cleared our own parent link.
            }
        }

        // Orphan children: clear their parent link and update their SubsurfaceState.
        let children_to_unmap = std::mem::take(&mut self.children); // Clears self.children
        for child_id in children_to_unmap {
            if let Some(child_surface_arc) = surface_registry_accessor.get_surface(child_id) {
                if let Ok(mut child_surface) = child_surface_arc.lock() {
                    child_surface.parent = None; // Mark child as orphaned
                    if let Some(SurfaceRole::Subsurface(ref mut sub_state)) = child_surface.role {
                        // Update SubsurfaceState to reflect it's no longer parented by this surface.
                        sub_state.parent_id = SurfaceId::new_unique(); // Sentinel for "no valid Wayland parent".
                    }
                }
            }
        }

        self.frame_callbacks.clear();
        self.damage_tracker.clear_all_damage();
        self.current_state = SurfaceState::Destroyed; // Directly set state.
    }


    /// Registers a frame callback for this surface. Corresponds to `wl_surface.frame`.
    ///
    /// The callback will be added to a list and is intended to be fired by the
    /// compositor after the next frame is rendered and presented.
    /// If the surface is already destroyed, this request is silently ignored.
    ///
    /// # Arguments
    /// * `callback_id`: The Wayland resource ID of the `wl_callback` object.
    pub fn frame(&mut self, callback_id: u64) {
        if self.current_state == SurfaceState::Destroyed {
            // As per Wayland spec for most requests on destroyed objects, silently ignore.
            return;
        }
        self.frame_callbacks.push(WlCallback { id: callback_id });
    }

    /// Retrieves all pending frame callbacks and clears the internal list.
    ///
    /// This is typically called by the compositor's rendering loop after a frame
    /// has been successfully presented, to notify clients.
    pub fn take_frame_callbacks(&mut self) -> Vec<WlCallback> {
        std::mem::take(&mut self.frame_callbacks)
    }

    /// Transitions the surface to a new state.
    ///
    /// # Arguments
    /// * `new_state`: The `SurfaceState` to transition to.
    ///
    /// # Returns
    /// `Ok(())` on a valid transition.
    /// `Err(SurfaceTransitionError::InvalidTransition)` if the transition is not allowed
    /// (e.g., transitioning from `Destroyed` to any other state except itself).
    pub fn transition_to(&mut self, new_state: SurfaceState) -> Result<(), SurfaceTransitionError> {
        // Basic safety: cannot un-destroy a surface.
        if self.current_state == SurfaceState::Destroyed && new_state != SurfaceState::Destroyed {
            return Err(SurfaceTransitionError::InvalidTransition);
        }
        // TODO: Add more sophisticated state transition validation if needed (e.g., specific allowed transitions).
        self.current_state = new_state;
        Ok(())
    }
}

/// Manages all surfaces within the compositor.
pub mod surface_registry {
    use super::{Surface, SurfaceId, SurfaceState, SurfaceRole, WlOutputTransform, Mat3x3, Rectangle, DamageTracker, WlCallback, Region, BufferAttachError, CommitError, SurfaceTransitionError};
    use crate::buffer_manager::BufferManager;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use crate::subcompositor::{SubsurfaceState, SubsurfaceSyncMode};


    /// Trait for providing read-only access to surfaces in the registry.
    ///
    /// This is used by methods like `Surface::prepare_for_destruction` to query
    /// parent or child surface information without needing a mutable borrow of the
    /// entire `SurfaceRegistry`, which can help avoid complex borrowing scenarios.
    pub trait SurfaceRegistryAccessor {
        /// Retrieves a shared pointer to a `Surface` by its `SurfaceId`.
        fn get_surface(&self, id: SurfaceId) -> Option<Arc<Mutex<Surface>>>;
    }

    /// Central registry for all surfaces known to the compositor.
    ///
    /// Provides methods for creating, retrieving, and unregistering surfaces.
    #[derive(Default)]
    pub struct SurfaceRegistry {
        surfaces: HashMap<SurfaceId, Arc<Mutex<Surface>>>,
    }

    impl SurfaceRegistryAccessor for SurfaceRegistry {
        fn get_surface(&self, id: SurfaceId) -> Option<Arc<Mutex<Surface>>> {
            self.surfaces.get(&id).cloned()
        }
    }

    impl SurfaceRegistry {
        /// Creates a new, empty `SurfaceRegistry`.
        pub fn new() -> Self {
            Self {
                surfaces: HashMap::new(),
            }
        }

        /// Creates a new `Surface`, registers it, and returns its `SurfaceId` and a shared pointer.
        ///
        /// # Returns
        /// A tuple `(SurfaceId, Arc<Mutex<Surface>>)` for the new surface.
        ///
        /// # Notes on Memory Exhaustion
        /// Rust's standard memory allocators (like for `Arc` and `Mutex`) will panic on
        /// out-of-memory (OOM) conditions. A production Wayland compositor should ideally use
        /// fallible allocation (e.g., `Box::try_new`) and propagate allocation errors
        /// to the Wayland protocol layer, typically by sending a `wl_display.error`
        /// with the `no_memory` error code to the client. This is not currently implemented.
        pub fn register_new_surface(&mut self) -> (SurfaceId, Arc<Mutex<Surface>>) {
            let surface = Surface::new();
            let id = surface.id;
            let arc_surface = Arc::new(Mutex::new(surface));
            self.surfaces.insert(id, arc_surface.clone());
            (id, arc_surface)
        }

        /// Registers an already existing `Arc<Mutex<Surface>>`.
        /// This might be used if surfaces are created externally and then added to the registry.
        pub fn register_surface_arc(&mut self, surface_arc: Arc<Mutex<Surface>>) -> SurfaceId {
            let id = surface_arc.lock().unwrap().id;
            self.surfaces.insert(id, surface_arc);
            id
        }

        /// Retrieves a shared pointer to a `Surface` by its `SurfaceId`.
        pub fn get_surface(&self, id: SurfaceId) -> Option<Arc<Mutex<Surface>>> {
            self.surfaces.get(&id).cloned()
        }

        /// Unregisters and effectively destroys a surface and its hierarchy.
        ///
        /// This process involves:
        /// 1. Calling `Surface::prepare_for_destruction()` on the target surface, which:
        ///    - Releases its Wayland buffers.
        ///    - Removes itself from its parent's list of children.
        ///    - Disassociates its own children (subsurfaces), effectively unmapping them.
        /// 2. Recursively calling `unregister_surface` for all children of the target surface.
        /// 3. Removing the target surface from the registry.
        ///
        /// # Arguments
        /// * `surface_id`: The `SurfaceId` of the surface to unregister.
        /// * `buffer_manager`: A mutable reference to the `BufferManager` for releasing buffers.
        ///
        /// # Returns
        /// An `Option<Arc<Mutex<Surface>>>` containing the unregistered surface if it was found.
        /// The `Surface` within the `Arc` will be in the `Destroyed` state.
        pub fn unregister_surface(
            &mut self,
            surface_id: SurfaceId,
            buffer_manager: &mut BufferManager,
        ) -> Option<Arc<Mutex<Surface>>> {
            if let Some(surface_arc) = self.surfaces.get(&surface_id).cloned() {
                // Step 1: Collect children IDs before modifying the surface's children list
                // during its prepare_for_destruction.
                let children_ids: Vec<SurfaceId> = {
                    let surface_guard = surface_arc.lock().unwrap(); // Handle potential poison
                    surface_guard.children.clone()
                };

                // Step 2: Prepare the current surface for destruction.
                // This modifies the surface's state, releases buffers, and updates parent/child links.
                {
                    let mut surface_guard = surface_arc.lock().unwrap();
                    if surface_guard.current_state != SurfaceState::Destroyed {
                        surface_guard.prepare_for_destruction(buffer_manager, self);
                    }
                }

                // Step 3: Recursively unregister children.
                // This ensures that each child also goes through `prepare_for_destruction`.
                for child_id in children_ids {
                    self.unregister_surface(child_id, buffer_manager);
                }

                // Step 4: Finally, remove the original target surface from the registry.
                self.surfaces.remove(&surface_id)
            } else {
                None // Surface not found in the registry.
            }
        }
    }
}
>>>>>>> REPLACE
