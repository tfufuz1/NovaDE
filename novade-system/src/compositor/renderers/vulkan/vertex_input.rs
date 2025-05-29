//! Defines the vertex structure and input descriptions for Vulkan graphics pipelines.
//!
//! This module specifies the layout of vertex data that will be passed to vertex shaders.
//! It includes the `Vertex` struct itself and functions to generate the necessary
//! Vulkan descriptor structures (`VkVertexInputBindingDescription` and
//! `VkVertexInputAttributeDescription`) that inform the graphics pipeline about
//! how to interpret raw vertex buffer data.

use ash::vk;
use bytemuck::{Pod, Zeroable}; // Traits for safe casting and zero-initialization
use memoffset::offset_of; // Macro to get byte offset of struct members

/// Represents a single vertex in a 2D graphics application.
///
/// Each vertex contains a 2D position (`pos`) and 2D texture coordinates (`tex_coord`).
/// This struct is marked `#[repr(C)]` to ensure a C-compatible memory layout,
/// which is crucial for Vulkan to correctly interpret vertex data from buffers.
/// It also derives `Pod` and `Zeroable` from the `bytemuck` crate, which are
/// marker traits indicating that this type is "Plain Old Data" and can be safely
/// transmuted or zero-initialized. This is useful for creating vertex buffers from slices of `Vertex`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    /// The 2D position of the vertex (x, y).
    /// Corresponds to `layout(location = 0)` in the vertex shader.
    pub pos: [f32; 2],
    /// The 2D texture coordinates of the vertex (u, v).
    /// Corresponds to `layout(location = 1)` in the vertex shader.
    pub tex_coord: [f32; 2],
}

impl Vertex {
    /// Returns the Vulkan vertex input binding description for this `Vertex` type.
    ///
    /// A `VkVertexInputBindingDescription` defines how a group of vertex attributes
    /// (which are defined by `VkVertexInputAttributeDescription`) are fetched from a
    /// vertex buffer. It specifies:
    /// - `binding`: The index of the vertex buffer binding. Typically 0 if all vertex
    ///   data comes from a single buffer.
    /// - `stride`: The byte distance between consecutive `Vertex` elements in the buffer.
    ///   This is calculated as `std::mem::size_of::<Self>()`.
    /// - `input_rate`: How data is consumed from this binding. `VERTEX` means per-vertex
    ///   data, while `INSTANCE` would mean per-instance data for instanced rendering.
    ///
    /// # Returns
    /// A `vk::VertexInputBindingDescription` configured for the `Vertex` struct.
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0) // Binding index for this vertex type
            .stride(std::mem::size_of::<Self>() as u32) // Size of one Vertex struct
            .input_rate(vk::VertexInputRate::VERTEX) // Data is per-vertex
            .build()
    }

    /// Returns the Vulkan vertex input attribute descriptions for this `Vertex` type.
    ///
    /// A `VkVertexInputAttributeDescription` describes a single vertex attribute, such as
    /// position or texture coordinates. For each attribute, it specifies:
    /// - `location`: The location of the attribute in the vertex shader (e.g., `layout(location = 0)`).
    /// - `binding`: Which vertex input binding this attribute belongs to (matches the `binding`
    ///   in `get_binding_description`).
    /// - `format`: The data type and format of the attribute in the buffer (e.g., `R32G32_SFLOAT` for a 2-component 32-bit float vector).
    /// - `offset`: The byte offset of this attribute from the beginning of a `Vertex` element in the buffer.
    ///   This is calculated using the `offset_of!` macro from the `memoffset` crate.
    ///
    /// This implementation defines two attributes:
    /// 1. `pos` ([f32; 2]) at location 0.
    /// 2. `tex_coord` ([f32; 2]) at location 1.
    ///
    /// # Returns
    /// A `Vec<vk::VertexInputAttributeDescription>` containing descriptions for all attributes of the `Vertex` struct.
    pub fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            // Position attribute (shader location 0)
            vk::VertexInputAttributeDescription::builder()
                .location(0) // Corresponds to `layout(location = 0) in vec2 a_Pos;` in vertex shader
                .binding(0)  // From the binding description at index 0
                .format(vk::Format::R32G32_SFLOAT) // Two 32-bit floats (vec2)
                .offset(offset_of!(Self, pos) as u32) // Offset of the 'pos' field within the Vertex struct
                .build(),
            // Texture coordinates attribute (shader location 1)
            vk::VertexInputAttributeDescription::builder()
                .location(1) // Corresponds to `layout(location = 1) in vec2 a_TexCoord;` in vertex shader
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT) // Two 32-bit floats (vec2)
                .offset(offset_of!(Self, tex_coord) as u32) // Offset of the 'tex_coord' field
                .build(),
        ]
    }
}
