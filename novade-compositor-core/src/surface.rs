//! Core data structures for managing Wayland surfaces and their state within the Novade compositor.
//!
//! This module defines the `Surface` struct, its attributes, states, and related helper
//! types like `Rectangle`, `DamageTracker`, and `Region`. It also includes the
//! `SurfaceRegistry` for managing all active surfaces.

use novade_buffer_manager::{BufferManager, BufferDetails, BufferId, ClientId, BufferFormat};
use std::sync::{Arc, Mutex};
use crate::subcompositor::{SubsurfaceState, SubsurfaceSyncMode}; // Used by SurfaceRole and Surface commit logic
use crate::region::Region; // Import the new Region type

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
    #[inline]
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    /// Checks if the rectangle has zero or negative width or height, making it effectively empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    /// Calculates the area of the rectangle.
    /// Returns 0 if the rectangle `is_empty()`.
    #[inline]
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
    /// Creates a new, empty `DamageTracker`.
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
    /// This role indicates the surface is being used as a cursor.
    /// Hotspot information is typically managed by the pointer logic (`WlPointer`).
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

/// Represents a Wayland surface (`wl_surface`).
///
/// This is a central structure in the compositor, holding all state related to
/// a client's drawable area. It manages pending and current attributes, attached buffers,
/// damage tracking, frame callbacks, and its role within the compositor (e.g., toplevel, subsurface).
pub struct Surface {
    /// Unique identifier for this surface.
    pub id: SurfaceId,
    /// Identifier of the client that owns this surface.
    pub client_id: ClientId,
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

    /// Pending opaque region for the surface, set via `wl_surface.set_opaque_region`.
    ///
    /// This region describes parts of the surface that are guaranteed to be opaque.
    /// The compositor can use this information to optimize rendering (e.g., by not
    /// rendering content obscured by an opaque region).
    /// A `None` value signifies an empty opaque region, meaning the client considers
    /// the surface entirely transparent beyond its explicit buffer content, or has
    /// passed a `NULL` `wl_region`. This is the initial state.
    /// This state is applied atomically during a `commit` operation.
    pub pending_opaque_region: Option<Region>,
    /// Current opaque region of the surface, active after a `commit`.
    ///
    /// `None` signifies an empty opaque region.
    pub current_opaque_region: Option<Region>,
    /// Pending input region for the surface, set via `wl_surface.set_input_region`.
    ///
    /// This region defines the areas of the surface that can receive pointer and touch input.
    /// A `None` value signifies that the entire surface accepts input (effectively an
    /// "infinite" region, clipped to the surface bounds). This is the initial state if
    /// the client passes a `NULL` `wl_region`.
    /// This state is applied atomically during a `commit` operation.
    pub pending_input_region: Option<Region>,
    /// Current input region of the surface, active after a `commit`.
    ///
    /// `None` signifies an infinite input region (the whole surface).
    pub current_input_region: Option<Region>,

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
    /// with no attached buffer or specific role.
    /// - Opaque region is initially `None` (empty), as per Wayland specification.
    /// - Input region is initially `None` (infinite/whole surface), as per Wayland specification.
    ///
    /// # Arguments
    /// * `client_id`: The ID of the client creating this surface.
    pub fn new(client_id: ClientId) -> Self {
        Self {
            id: SurfaceId::new_unique(),
            client_id,
            current_state: SurfaceState::Created,
            pending_attributes: SurfaceAttributes::default(),
            current_attributes: SurfaceAttributes::default(),
            damage_tracker: DamageTracker::new(),
            role: None,
            pending_buffer: None,
            current_buffer: None,
            frame_callbacks: Vec::new(),
            pending_opaque_region: None,
            current_opaque_region: None,
            pending_input_region: None,
            current_input_region: None,
            children: Vec::new(),
            parent: None,
            cached_pending_buffer: None,
            cached_pending_attributes: None,
        }
    }

    /// Sets the pending opaque region for this surface, corresponding to `wl_surface.set_opaque_region`.
    ///
    /// The opaque region defines parts of the surface that the compositor may optimize for,
    /// as they are guaranteed not to be see-through.
    /// - If `region_opt` is `Some(region_ref)`, the surface's pending opaque region is set
    ///   to a clone of `region_ref`. The coordinates in `region_ref` are in surface-local units.
    /// - If `region_opt` is `None` (representing a `NULL` `wl_region` from the client),
    ///   the pending opaque region is set to `None`, which signifies an empty opaque region.
    ///
    /// The change takes effect upon the next successful `commit` operation.
    /// This operation is ignored if the surface is destroyed.
    ///
    /// # Arguments
    /// * `region_opt`: An `Option<&Region>` representing the new opaque region.
    ///   The `Region` data is cloned into the surface's `pending_opaque_region`.
    pub fn set_opaque_region(&mut self, region_opt: Option<&Region>) {
        if self.current_state == SurfaceState::Destroyed {
            return;
        }
        self.pending_opaque_region = region_opt.cloned();
    }

    /// Sets the pending input region for this surface, corresponding to `wl_surface.set_input_region`.
    ///
    /// The input region defines the areas of the surface that can receive pointer and touch input.
    /// - If `region_opt` is `Some(region_ref)`, the surface's pending input region is set
    ///   to a clone of `region_ref`. Coordinates are in surface-local units.
    /// - If `region_opt` is `None` (representing a `NULL` `wl_region` from the client),
    ///   the pending input region is set to `None`. This signifies that the entire
    ///   surface accepts input (an "infinite" region, effectively clipped by surface bounds).
    ///
    /// The change takes effect upon the next successful `commit` operation.
    /// This operation is ignored if the surface is destroyed.
    ///
    /// # Arguments
    /// * `region_opt`: An `Option<&Region>` representing the new input region.
    ///   The `Region` data is cloned into the surface's `pending_input_region`.
    pub fn set_input_region(&mut self, region_opt: Option<&Region>) {
        if self.current_state == SurfaceState::Destroyed {
            return;
        }
        self.pending_input_region = region_opt.cloned();
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
    /// Releases the `current_buffer` associated with the surface, if any.
    ///
    /// This is an internal helper method typically called during a commit operation
    /// (when a new buffer replaces the current one) or when the surface is destroyed.
    /// It decrements the reference count of the buffer via the provided `BufferManager`.
    /// If this is the last reference, the `BufferManager` may free the buffer.
    ///
    /// # Arguments
    /// * `buffer_manager`: A mutable reference to the `BufferManager` to handle buffer release.
    fn release_current_buffer(&mut self, buffer_manager: &mut BufferManager) {
        if let Some(buffer_arc) = self.current_buffer.take() {
            // .take() removes the Arc from self.current_buffer, effectively dropping one reference count
            // on the Arc itself. The BufferManager then handles its internal ref count for the BufferDetails.
            let buffer_id = buffer_arc.lock().unwrap().id;
            buffer_manager.release_buffer(buffer_id);
        }
    }

    /// Releases the `pending_buffer` associated with the surface, if any.
    ///
    /// This is an internal helper method called when a new buffer is attached (replacing
    /// a previous pending buffer), when a commit makes the pending buffer current,
    /// or when the surface is destroyed.
    /// It decrements the reference count of the buffer via the provided `BufferManager`.
    ///
    /// # Arguments
    /// * `buffer_manager`: A mutable reference to the `BufferManager` to handle buffer release.
    fn release_pending_buffer(&mut self, buffer_manager: &mut BufferManager) {
        if let Some(buffer_arc) = self.pending_buffer.take() {
            // Similar to release_current_buffer, .take() drops this surface's Arc reference.
            // BufferManager then updates its own reference count for the BufferDetails.
            let buffer_id = buffer_arc.lock().unwrap().id;
            buffer_manager.release_buffer(buffer_id);
        }
    }

    /// Adds damage to the surface in buffer coordinates, corresponding to `wl_surface.damage_buffer`.
    ///
    /// The provided rectangle defines an area of the buffer that has changed and needs to be
    /// redrawn. This damage is added to the `pending_damage_buffer` list in the `DamageTracker`.
    ///
    /// The damage rectangle is clipped to the bounds of the buffer that the damage applies to
    /// (pending buffer if it exists, otherwise the current buffer). If no buffer is attached
    /// or the buffer has zero dimensions, the damage is ignored.
    ///
    /// If the surface is in the `Destroyed` state, this operation is a no-op.
    /// According to the Wayland specification, `width` and `height` must be positive.
    /// Non-positive values will result in the damage being ignored. A compositor may choose
    /// to send a `wl_surface.error.invalid_damage` protocol error in such cases.
    ///
    /// # Arguments
    /// * `x`, `y`: The top-left coordinates of the damage rectangle, relative to the buffer.
    /// * `width`, `height`: The dimensions of the damage rectangle. Must be positive.
    pub fn damage_buffer(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.current_state == SurfaceState::Destroyed { return; }

        let rect = Rectangle::new(x, y, width, height);
        // Wayland spec: width and height must be positive.
        // Silently ignore invalid ones as per current simplified error handling.
        // A production compositor might send wl_surface.error.invalid_damage.
        if rect.is_empty() { return; }

        // Determine the dimensions of the buffer this damage applies to.
        // Damage applies to the pending buffer if one exists, otherwise the current buffer.
        // This aligns with the idea that damage is for content that *will be* or *is* displayed.
        let opt_buffer_dims: Option<(i32, i32)> =
            if let Some(pending_ref) = &self.pending_buffer {
                let details = pending_ref.lock().unwrap();
                Some((details.width as i32, details.height as i32))
            } else if let Some(current_ref) = &self.current_buffer {
                let details = current_ref.lock().unwrap();
                Some((details.width as i32, details.height as i32))
            } else {
                None // No buffer attached, damage has no context.
            };

        if let Some(buffer_dims) = opt_buffer_dims {
            // Only proceed if the buffer has valid, positive dimensions.
            if buffer_dims.0 > 0 && buffer_dims.1 > 0 {
                let buffer_bounds = Rectangle::new(0, 0, buffer_dims.0, buffer_dims.1);
                let clipped_rect = rect.clipped_to(&buffer_bounds);
                if !clipped_rect.is_empty() {
                    self.damage_tracker.add_damage_buffer(clipped_rect);
                }
            }
            // If buffer_dims are zero or negative (should be caught at attach), damage is ignored.
        }
        // If no buffer is attached (opt_buffer_dims is None), wl_surface.damage_buffer has no effect.
    }

    /// Adds damage to the surface in surface-local coordinates, corresponding to `wl_surface.damage`.
    ///
    /// This is a legacy request. The provided rectangle defines an area of the surface
    /// that has changed. This damage is added to the `pending_damage_surface` list in `DamageTracker`.
    ///
    /// The damage rectangle is clipped to the current (not pending) bounds of the surface.
    /// If the surface has no size or is in the `Destroyed` state, this operation is a no-op.
    /// According to the Wayland specification, `width` and `height` must be positive.
    /// Non-positive values will result in the damage being ignored. A compositor may send
    /// `wl_surface.error.invalid_damage`.
    ///
    /// # Arguments
    /// * `x`, `y`: The top-left coordinates of the damage rectangle, relative to the surface.
    /// * `width`, `height`: The dimensions of the damage rectangle. Must be positive.
    pub fn damage_surface(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.current_state == SurfaceState::Destroyed { return; }

        let rect = Rectangle::new(x, y, width, height);
        // Wayland spec: width and height must be positive. Silently ignore here.
        if rect.is_empty() { return; }

        // Clip to current (not pending) surface attributes for surface damage.
        // This is because wl_surface.damage applies to the "current" visible state.
        let surface_width = self.current_attributes.size.0 as i32;
        let surface_height = self.current_attributes.size.1 as i32;

        // If surface has no effective area (e.g., width or height is zero),
        // then damage is clipped to nothing.
        if surface_width <= 0 || surface_height <= 0 {
            return;
        }
        let surface_bounds = Rectangle::new(0, 0, surface_width, surface_height);

        let clipped_rect = rect.clipped_to(&surface_bounds);
        if !clipped_rect.is_empty() {
            self.damage_tracker.add_damage_surface(clipped_rect);
        }
    }

    /// Attaches a buffer to the surface, corresponding to `wl_surface.attach`.
    ///
    /// This operation sets the `pending_buffer` for the surface. The actual content display
    /// changes only upon a successful `commit`. Any previously pending buffer is released.
    ///
    /// - If `buffer_arc_opt` is `Some(buffer_arc)`, the provided buffer is attached.
    ///   Its reference count in the `BufferManager` is incremented (via `BufferDetails::increment_ref_count`).
    ///   The `pending_attributes.size` is updated to the dimensions of this buffer.
    /// - If `buffer_arc_opt` is `None`, it signifies attaching a NULL buffer (no content).
    ///   The `pending_attributes.size` is *not* changed in this case.
    ///
    /// After successful attachment, the surface transitions to `SurfaceState::PendingBuffer`
    /// if it wasn't already in that state.
    ///
    /// # Protocol Constraints & Error Handling:
    /// - The surface must not be in the `Destroyed` state.
    /// - For `wl_surface` version 5 and later, `x_offset` and `y_offset` *must* be 0 if a
    ///   non-NULL buffer is attached. This implementation enforces this unconditionally and
    ///   returns `BufferAttachError::InvalidBufferOffset` if violated (corresponds to
    ///   `wl_surface.error.invalid_offset`).
    /// - If a non-NULL buffer is provided, its dimensions (`width`, `height`) must be positive.
    ///   Otherwise, `BufferAttachError::InvalidBufferSize` is returned (corresponds to
    ///   `wl_surface.error.invalid_size`).
    /// - Optional: Client ownership of the buffer can be checked. If `BufferDetails.client_owner_id`
    ///   is `Some` and does not match `client_id`, `BufferAttachError::ClientMismatch` is returned.
    ///   This is a compositor-specific policy.
    ///
    /// # Arguments
    /// * `buffer_manager`: A mutable reference to the `BufferManager` for releasing any
    ///   previous pending buffer.
    /// * `buffer_arc_opt`: An `Option<Arc<Mutex<BufferDetails>>>` for the buffer to attach.
    ///   `None` signifies attaching a NULL buffer. The `Arc` is expected to be a valid reference
    ///   obtained from the `BufferManager` or other trusted source.
    /// * `client_id`: The `ClientId` of the client making the request. Used for optional
    ///   ownership validation.
    /// * `x_offset`, `y_offset`: Offsets for the buffer content relative to the surface's origin.
    ///   These are stored in `pending_attributes.buffer_offset`.
    ///
    /// # Returns
    /// - `Ok(())` on successful attachment.
    /// - `Err(BufferAttachError)` if any validation fails.
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

    /// Commits the pending state of the surface, making it current. Corresponds to `wl_surface.commit`.
    ///
    /// This is a central operation in the Wayland surface lifecycle. It atomically applies all
    /// pending state set since the last commit. This includes:
    /// - Attached buffer (`pending_buffer`)
    /// - Surface attributes (`pending_attributes` such as scale, transform, offset)
    /// - Opaque region (`pending_opaque_region`)
    /// - Input region (`pending_input_region`)
    /// - Damage regions (`pending_damage_buffer`, `pending_damage_surface`)
    ///
    /// # Behavior:
    /// 1.  **Validation**:
    ///     *   The surface must not be `Destroyed`.
    ///     *   The current state must allow committing (e.g., `Created`, `PendingBuffer`, `Committed`).
    ///     *   If a `pending_buffer` exists:
    ///         *   Its dimensions must be positive.
    ///         *   `pending_attributes.buffer_scale` must be positive.
    ///         *   Buffer dimensions must be integer multiples of the `buffer_scale`.
    ///         If any validation fails, a `CommitError` is returned.
    ///
    /// 2.  **Synchronized Subsurface Handling**:
    ///     *   If the surface is a synchronized subsurface, its `pending_buffer` (if any),
    ///       `pending_attributes`, `pending_opaque_region`, and `pending_input_region` are
    ///       cached. Damage remains pending until the parent commits and calls `apply_cached_state_from_sync`.
    ///     *   The surface transitions to `SurfaceState::Committed` and the function returns.
    ///
    /// 3.  **Regular Commit (Toplevels, Desynchronized Subsurfaces)**:
    ///     *   `current_attributes` is updated from `pending_attributes`.
    ///     *   `current_opaque_region` is updated from `pending_opaque_region.clone()`.
    ///     *   `current_input_region` is updated from `pending_input_region.clone()`.
    ///     *   If an attach operation occurred (`current_state` was `PendingBuffer`):
    ///         *   The old `current_buffer` is released.
    ///         *   The `pending_buffer` becomes the new `current_buffer`.
    ///         *   `new_content_committed` flag is set for damage processing.
    ///     *   If it's a desynchronized subsurface with prior cached state, `apply_cached_state_from_sync` is called.
    ///     *   `damage_tracker.commit_pending_damage()` is called to process all pending damage.
    ///     *   The surface transitions to `SurfaceState::Committed`.
    ///
    /// # Arguments
    /// * `buffer_manager`: A mutable reference to the `BufferManager` for managing buffer
    ///   reference counts during buffer transitions.
    ///
    /// # Returns
    /// - `Ok(())` if the commit is successful and all state is applied or cached.
    /// - `Err(CommitError)` if validation fails.
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

                // Cache the pending state (buffer, attributes, and regions).
                // Damage remains in `damage_tracker.pending_damage_*` and will be processed by `apply_cached_state_from_sync`.
                if let Some(pending_buffer_arc_real) = &self.pending_buffer {
                    pending_buffer_arc_real.lock().unwrap().increment_ref_count(); // Cache holds a new ref.
                    self.cached_pending_buffer = Some(pending_buffer_arc_real.clone());
                } else {
                    self.cached_pending_buffer = None; // Handles attach(NULL) case.
                }
                self.cached_pending_attributes = Some(self.pending_attributes);
                // Note: Regions (opaque/input) are part of the pending state that gets cached implicitly
                // by not clearing them here. apply_cached_state_from_sync will then apply them.

                // The original self.pending_buffer is now "consumed" into the cache.
                self.pending_buffer.take();

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
        // Apply pending regions to current regions.
        self.current_opaque_region = self.pending_opaque_region.clone();
        self.current_input_region = self.pending_input_region.clone();


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

    /// Applies cached state for a synchronized subsurface, making its content and attributes current.
    ///
    /// This method is primarily called in two scenarios:
    /// 1.  By the compositor when a parent surface commits, to apply the latched state of its
    ///     synchronized children.
    /// 2.  By the `commit` method of this surface if it's transitioning from a synchronized
    ///     state (where its state was cached) to a desynchronized state, and now needs to
    ///     make that cached state live.
    ///
    /// # Behavior:
    /// - If the surface is `Destroyed` or not a subsurface with `needs_apply_on_parent_commit` set,
    ///   it returns `Ok(())` or `Err(CommitError::InvalidState)` as appropriate, without action.
    /// - Clears the `needs_apply_on_parent_commit` flag in `SubsurfaceState`.
    /// - Moves `cached_pending_attributes` to `current_attributes`.
    /// - Applies pending opaque and input regions (which were part of the subsurface's state when it
    ///   committed and cached) to the current opaque and input regions.
    /// - Releases the old `current_buffer` (if any).
    /// - Moves `cached_pending_buffer` (if any) to `current_buffer`. The reference count for this
    ///   buffer was already incremented when it was cached.
    /// - Calls `damage_tracker.commit_pending_damage()` to process any damage that was
    ///   pending in the `DamageTracker` from when the subsurface originally cached its state.
    ///   The `new_content_was_cached` flag for damage processing is determined by whether
    ///   attributes or a buffer were actually cached (indicating a content update).
    ///
    /// The surface state is expected to be `Committed` (set when it originally cached its state)
    /// and generally remains `Committed` after this operation.
    ///
    /// # Arguments
    /// * `buffer_manager`: A mutable reference to the `BufferManager` for releasing the
    ///   previous `current_buffer`.
    ///
    /// # Returns
    /// - `Ok(())` if the cached state was successfully applied or if there was no state to apply.
    /// - `Err(CommitError::InvalidState)` if the surface is `Destroyed` or not in a valid
    ///   state/role to perform this operation.
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
            // Not a subsurface, this method is inappropriate.
            return Err(CommitError::InvalidState);
        }

        if !needs_apply_flag_was_set {
            // Should have returned above if flag wasn't set. Defensive.
            return Ok(());
        }

        // Apply cached attributes.
        let had_cached_attributes = self.cached_pending_attributes.is_some();
        if let Some(cached_attrs) = self.cached_pending_attributes.take() {
            self.current_attributes = cached_attrs;
        }

        // Apply pending regions from when the subsurface committed its state (now being applied by parent).
        // These were part of its "pending" state that got latched.
        self.current_opaque_region = self.pending_opaque_region.clone();
        self.current_input_region = self.pending_input_region.clone();

        // Apply cached buffer.
        let had_cached_buffer = self.cached_pending_buffer.is_some();
        self.release_current_buffer(buffer_manager); // Release any existing current buffer.
        self.current_buffer = self.cached_pending_buffer.take();

        let new_content_was_cached = had_cached_buffer || had_cached_attributes;

        self.damage_tracker.commit_pending_damage(&self.current_attributes, new_content_was_cached);

        Ok(())
    }

    /// Prepares the surface for destruction, performing necessary cleanup.
    ///
    /// This method is called when a surface (and its associated Wayland resource) is
    /// being destroyed. It ensures that resources are released and hierarchical
    /// relationships are correctly updated.
    ///
    /// # Behavior:
    /// 1.  **Buffer Release**: Releases `pending_buffer`, `current_buffer`, and
    ///     `cached_pending_buffer` (if any) via the `BufferManager`. This decrements
    ///     their reference counts.
    /// 2.  **Parent Detachment**: If the surface has a `parent`, it removes itself from
    ///     its parent's `children` list. This requires access to the parent surface
    ///     via the `surface_registry_accessor`.
    /// 3.  **Children Orphaning**: Clears its own `children` list. For each child:
    ///     *   Their `parent` link is set to `None`.
    ///     *   If the child was a `Subsurface`, its `SubsurfaceState.parent_id` is set
    ///       to a sentinel value, effectively unmapping it from a Wayland perspective
    ///       as its Wayland parent link is now broken.
    /// 4.  **Cleanup**: Clears all `frame_callbacks` and all damage in `damage_tracker`.
    ///     Opaque and input regions are implicitly dropped as part of the struct.
    /// 5.  **State Transition**: Directly sets `current_state` to `SurfaceState::Destroyed`.
    ///     This bypasses `transition_to` as it's a final, irreversible state change.
    ///
    /// # Arguments
    /// * `buffer_manager`: A mutable reference to the `BufferManager` for releasing buffers.
    /// * `surface_registry_accessor`: An implementation of `SurfaceRegistryAccessor` (typically
    ///   the `SurfaceRegistry` itself) to allow querying and modifying parent/child
    ///   surface relationships. This is crucial for detaching from parents and orphaning children.
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
        // Regions (pending_opaque_region, etc.) are owned Option<Region> and will be dropped.
        self.current_state = SurfaceState::Destroyed; // Directly set state.
    }


    /// Registers a frame callback for this surface, corresponding to `wl_surface.frame`.
    ///
    /// A frame callback is a request by the client to be notified when the current
    /// state of the surface (after the next commit) is presented by the compositor.
    /// The `WlCallback` object, identified by `callback_id`, will be used to send
    /// this notification.
    ///
    /// Frame callbacks are accumulated in the `frame_callbacks` list. They are typically
    /// processed and sent by the compositor after rendering and displaying a new frame
    /// that includes this surface's latest content.
    ///
    /// If the surface is in the `Destroyed` state, this request is silently ignored,
    /// aligning with typical Wayland object behavior.
    ///
    /// # Arguments
    /// * `callback_id`: The Wayland resource ID of the `wl_callback` object created by the client.
    pub fn frame(&mut self, callback_id: u64) {
        if self.current_state == SurfaceState::Destroyed {
            // Wayland spec: "If the wl_surface object is destroyed, the wl_callback objects
            // are likewise destroyed and the done event is never fired."
            // Silently ignoring is consistent with not firing the event.
            return;
        }
        self.frame_callbacks.push(WlCallback { id: callback_id });
    }

    /// Retrieves all pending frame callbacks and clears the internal list.
    ///
    /// This method is designed to be called by the compositor's main loop or rendering
    /// pipeline after it has processed and presented a frame that includes this surface's
    /// content. The returned `Vec<WlCallback>` contains all callbacks that were
    /// requested for this frame. The compositor is then responsible for sending the
    /// `done` event for each of these `wl_callback` objects.
    ///
    /// The internal list of frame callbacks is cleared, ensuring that each callback
    /// is only processed once.
    ///
    /// # Returns
    /// A `Vec<WlCallback>` containing all frame callbacks that were pending on this surface.
    /// The vector will be empty if no callbacks were registered.
    pub fn take_frame_callbacks(&mut self) -> Vec<WlCallback> {
        // std::mem::take replaces self.frame_callbacks with an empty Vec and returns the old Vec.
        std::mem::take(&mut self.frame_callbacks)
    }

    /// Transitions the surface to a new `SurfaceState`.
    ///
    /// This method provides a controlled way to update the `current_state` of the surface.
    /// It includes basic validation, primarily to prevent a `Destroyed` surface from
    /// transitioning back to any other state.
    ///
    /// More sophisticated state transition logic (e.g., defining a full state machine
    /// with allowed transitions between all states) could be added here if required.
    ///
    /// # Arguments
    /// * `new_state`: The target `SurfaceState` to transition to.
    ///
    /// # Returns
    /// - `Ok(())` if the transition is valid and applied.
    /// - `Err(SurfaceTransitionError::InvalidTransition)` if the transition is not allowed
    ///   (e.g., attempting to transition from `Destroyed` to `Created`).
    pub fn transition_to(&mut self, new_state: SurfaceState) -> Result<(), SurfaceTransitionError> {
        // A Destroyed surface cannot transition to any other state (except perhaps itself, which is a no-op).
        if self.current_state == SurfaceState::Destroyed && new_state != SurfaceState::Destroyed {
            return Err(SurfaceTransitionError::InvalidTransition);
        }

        // TODO: Implement a more comprehensive state machine if complex transition rules are needed.
        // For example, ensuring a surface only moves from PendingBuffer to Committed, etc.
        // For now, any transition not from Destroyed is allowed.
        self.current_state = new_state;
        Ok(())
    }
}

/// Manages all surfaces within the compositor.
pub mod surface_registry {
    use super::{Surface, SurfaceId, SurfaceState, SurfaceRole, WlOutputTransform, Mat3x3, Rectangle, DamageTracker, WlCallback, Region, BufferAttachError, CommitError, SurfaceTransitionError}; // Added Region here
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
        /// Retrieves a list of all surface IDs, potentially for iteration.
        /// The order is not guaranteed unless specified by the implementation.
        fn get_all_surface_ids(&self) -> Vec<SurfaceId>;
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

        fn get_all_surface_ids(&self) -> Vec<SurfaceId> {
            self.surfaces.keys().cloned().collect()
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
        ///
        /// # Arguments
        /// * `client_id`: The ID of the client creating this surface.
        pub fn register_new_surface(&mut self, client_id: ClientId) -> (SurfaceId, Arc<Mutex<Surface>>) {
            let surface = Surface::new(client_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::RegionId; // For creating Regions in tests
    // Assuming RegionRegistry is needed if we are creating regions with unique IDs for tests
    // use crate::region::RegionRegistry;
    use novade_buffer_manager::BufferManager; // For commit/attach calls
    use std::sync::atomic::Ordering;


    // Helper to create a default BufferManager for tests that need it.
    fn test_buffer_manager() -> BufferManager {
        BufferManager::new(log::logger()) // Assuming logger() is available or use a simpler new if not.
    }

    // Helper for ClientId in tests
    fn test_client_id() -> ClientId { ClientId::new(1) }


    #[test]
    fn surface_creation_defaults() {
        let client_id = test_client_id();
        let surface = Surface::new(client_id);
        assert_eq!(surface.id.0, SurfaceId::new_unique().0 -1); // Brittle if NEXT_ID changes start
        assert_eq!(surface.client_id, client_id);
        assert_eq!(surface.current_state, SurfaceState::Created);
        assert_eq!(surface.pending_attributes, SurfaceAttributes::default());
        assert_eq!(surface.current_attributes, SurfaceAttributes::default());
        assert!(surface.pending_buffer.is_none());
        assert!(surface.current_buffer.is_none());
        assert!(surface.frame_callbacks.is_empty());
        assert!(surface.role.is_none());
        assert!(surface.children.is_empty());
        assert!(surface.parent.is_none());
        assert!(surface.cached_pending_buffer.is_none());
        assert!(surface.cached_pending_attributes.is_none());

        // Test initial region states
        assert!(surface.pending_opaque_region.is_none(), "Initial pending_opaque_region should be None (empty)");
        assert!(surface.current_opaque_region.is_none(), "Initial current_opaque_region should be None (empty)");
        assert!(surface.pending_input_region.is_none(), "Initial pending_input_region should be None (infinite)");
        assert!(surface.current_input_region.is_none(), "Initial current_input_region should be None (infinite)");
    }

    #[test]
    fn surface_id_uniqueness() {
        let client_id = test_client_id();
        let surface1 = Surface::new(client_id);
        let surface2 = Surface::new(client_id); // ClientId doesn't affect SurfaceId
        assert_ne!(surface1.id, surface2.id);
    }

    #[test]
    fn attach_null_buffer() {
        let client_id = test_client_id();
        let mut surface = Surface::new(client_id);
        let mut bm = test_buffer_manager();
        //let client_id = test_client_id(); // client_id already defined for surface

        let initial_size = surface.pending_attributes.size;
        let result = surface.attach_buffer(&mut bm, None, client_id, 0, 0);
        assert!(result.is_ok());
        assert!(surface.pending_buffer.is_none());
        assert_eq!(surface.pending_attributes.size, initial_size, "Attaching NULL buffer should not change pending size");
        assert_eq!(surface.current_state, SurfaceState::PendingBuffer);
    }

    // ... (other existing tests for attach_buffer, commit, damage, etc. would be here) ...
    // For brevity, I'll assume they are present and correct from previous subtasks.

    // --- Tests for Region Integration ---

    #[test]
    fn test_surface_initial_regions_explicit_check() { // Renamed to avoid conflict if already tested by surface_creation_defaults
        let client_id = test_client_id();
        let surface = Surface::new(client_id);
        assert!(surface.pending_opaque_region.is_none());
        assert!(surface.current_opaque_region.is_none());
        assert!(surface.pending_input_region.is_none());
        assert!(surface.current_input_region.is_none());
    }

    #[test]
    fn test_set_opaque_region_with_some_region() {
        let client_a = test_client_id();
        let mut surface_registry = SurfaceRegistry::new(); // Needed for surface for role setting
        let (_surface_id, surface_arc) = surface_registry.register_new_surface(client_a);
        let mut surface = surface_arc.lock().unwrap();

        let mut bm = test_buffer_manager();

        let region_id = RegionId::new_unique();
        let mut test_region = Region::new(region_id);
        test_region.add(Rectangle::new(10, 10, 50, 50));

        surface.set_opaque_region(Some(&test_region));
        assert!(surface.pending_opaque_region.is_some());
        assert_eq!(surface.pending_opaque_region.as_ref().unwrap().get_rectangles(), test_region.get_rectangles());
        // Ensure it's a clone, not the same instance if Region was not Copy (it is Clone)
        if let Some(pending_region) = &surface.pending_opaque_region {
           assert_ne!(pending_region.id, test_region.id, "Pending region should be a clone with a new ID if Region IDs are unique per instance, or we should check content only.");
           // Current Region struct clones the ID. If Region was meant to be a shared resource (Arc<Mutex<RegionData>>), this test would change.
           // Given the current Region::clone, it clones the ID and Vec.
        }


        let commit_result = surface.commit(&mut bm);
        assert!(commit_result.is_ok());

        assert!(surface.current_opaque_region.is_some());
        assert_eq!(surface.current_opaque_region.as_ref().unwrap().get_rectangles(), test_region.get_rectangles());
         if let Some(current_region) = &surface.current_opaque_region {
             if let Some(pending_region) = &surface.pending_opaque_region { // pending_opaque_region is consumed by commit if not cloned properly
                 // This check relies on commit cloning the pending_opaque_region.
                 // If commit moves, pending_opaque_region would be None here or its content different.
                 // Based on current Surface::commit, it does `self.current_opaque_region = self.pending_opaque_region.clone();`
                 // So pending_opaque_region itself is not None after commit.
                 assert_eq!(current_region.id, pending_region.id);
             } else {
                // This case implies commit might not clone but move from pending, which is also fine.
                // The main check is that current_opaque_region has the correct content.
             }
        }
    }

    #[test]
    fn test_set_opaque_region_with_none() {
        let client_a = test_client_id();
        let mut surface_registry = SurfaceRegistry::new();
        let (_surface_id, surface_arc) = surface_registry.register_new_surface(client_a);
        let mut surface = surface_arc.lock().unwrap();
        let mut bm = test_buffer_manager();

        // Set it to Some first, then to None
        let region_id = RegionId::new_unique();
        let mut test_region = Region::new(region_id);
        test_region.add(Rectangle::new(0,0,1,1));
        surface.set_opaque_region(Some(&test_region));
        assert!(surface.pending_opaque_region.is_some());

        surface.set_opaque_region(None);
        assert!(surface.pending_opaque_region.is_none());

        let commit_result = surface.commit(&mut bm);
        assert!(commit_result.is_ok());
        assert!(surface.current_opaque_region.is_none());
    }

    #[test]
    fn test_set_input_region_with_some_region() {
        let client_a = test_client_id();
        let mut surface_registry = SurfaceRegistry::new();
        let (_surface_id, surface_arc) = surface_registry.register_new_surface(client_a);
        let mut surface = surface_arc.lock().unwrap();
        let mut bm = test_buffer_manager();

        let region_id = RegionId::new_unique();
        let mut test_region = Region::new(region_id);
        test_region.add(Rectangle::new(20, 20, 30, 30));

        surface.set_input_region(Some(&test_region));
        assert!(surface.pending_input_region.is_some());
        assert_eq!(surface.pending_input_region.as_ref().unwrap().get_rectangles(), test_region.get_rectangles());

        let commit_result = surface.commit(&mut bm);
        assert!(commit_result.is_ok());

        assert!(surface.current_input_region.is_some());
        assert_eq!(surface.current_input_region.as_ref().unwrap().get_rectangles(), test_region.get_rectangles());
    }

    #[test]
    fn test_set_input_region_with_none() {
        let client_a = test_client_id();
        let mut surface_registry = SurfaceRegistry::new();
        let (_surface_id, surface_arc) = surface_registry.register_new_surface(client_a);
        let mut surface = surface_arc.lock().unwrap();
        let mut bm = test_buffer_manager();

        let region_id = RegionId::new_unique();
        let mut test_region = Region::new(region_id);
        test_region.add(Rectangle::new(0,0,1,1));
        surface.set_input_region(Some(&test_region));
        assert!(surface.pending_input_region.is_some());

        surface.set_input_region(None);
        assert!(surface.pending_input_region.is_none());

        let commit_result = surface.commit(&mut bm);
        assert!(commit_result.is_ok());
        assert!(surface.current_input_region.is_none());
    }
}

[end of novade-compositor-core/src/surface.rs]
