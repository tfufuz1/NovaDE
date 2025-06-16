// novade-system/src/renderer/wgpu_renderer.rs

use crate::compositor::renderer_interface::abstraction::{FrameRenderer, RenderElement, RenderableTexture, RendererError};
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle}; // For WGPU surface creation
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result; // For internal constructor
use smithay::utils::{Physical, Rectangle, Size}; // For FrameRenderer trait
use std::borrow::Cow; // For shader loading
use crate::renderer::wgpu_texture::WgpuRenderableTexture;
use smithay::reexports::wayland_server::protocol::wl_shm::Format as WlShmFormat;
use smithay::wayland::shm::with_buffer_contents_data;
use wgpu::util::DeviceExt; // For create_buffer_init

// EGL and GLES imports for DMABUF
use khronos_egl as egl_types;
use egl::{Instance as EglInstance, EGL1_4}; // Using egl crate's Instance
use glow; // For GLES bindings
use drm_fourcc::{DrmFourcc, DrmModifier}; // For interpreting DMABUF format and modifier

// WGPU HAL imports (assuming wgpu 0.7 based on Cargo.toml addition)
// For wgpu 0.7, the path might be wgpu_hal::gles rather than wgpu_hal::api::Gles
// and Device::from_hal might not exist. Need to check wgpu 0.7 docs.
// Wgpu 0.7's HAL structure:
// Adapter<B: wgpu_hal::Api>
// Device<B: wgpu_hal::Api>
// Instance::new takes hal_api::Instance
// For GLES, it would be wgpu_hal::gles::Adapter, wgpu_hal::gles::Device
// The from_hal pattern is more common in later WGPU versions (e.g., 0.9+).
// For wgpu 0.7, unsafe access to raw GL context might be via Adapter::raw_adapter_handle() if available
// or by assuming the EGL context used by Winit/WGPU is current.

// Placeholder for WgpuRenderableTexture to be defined in wgpu_texture.rs
// For now, we'll use a simple struct that doesn't fully implement RenderableTexture
// #[derive(Debug)] // Commented out as WgpuRenderableTexture is now imported
// pub struct WgpuRenderableTextureStub {
//     id: Uuid,
//     // wgpu_texture: wgpu::Texture,
    // wgpu_texture_view: wgpu::TextureView,
    // wgpu_sampler: wgpu::Sampler,
    // width: u32, // These fields were part of the stub, WgpuRenderableTexture has its own
    // height: u32,
// }
// impl RenderableTexture for WgpuRenderableTextureStub { ... } // Full impl later

// Vertex struct
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2], // Simplified to 2D for quad
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { // Position
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // Tex Coords
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] }, // Bottom Left
    Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 1.0] }, // Bottom Right
    Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 0.0] }, // Top Right
    Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 0.0] }, // Top Left
];

const QUAD_INDICES: &[u16] = &[
    0, 1, 2, // First triangle
    0, 2, 3, // Second triangle
];

pub struct NovaWgpuRenderer {
    id: Uuid,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    pub device: Arc<wgpu::Device>, // Arc for sharing with textures/buffers
    pub queue: Arc<wgpu::Queue>,   // Arc for sharing

    // Surface-related fields
    // These are typically per-output or per-window
    // For a compositor, each output (monitor) might have its own surface/swapchain.
    // For now, assuming a single primary surface, e.g., for a Winit window.
    surface: Option<wgpu::Surface>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    // swap_chain: Option<wgpu::SwapChain>, // For older wgpu, now SurfaceConfiguration

    screen_size_physical: Size<i32, Physical>, // Store the physical size of the surface

    // New fields for pipeline and buffers
    render_pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    quad_num_indices: u32,
    texture_bind_group_layout: wgpu::BindGroupLayout, // For group 0 (texture+sampler)
    transform_bind_group_layout: wgpu::BindGroupLayout, // For group 1 (transform matrix)
    default_sampler: Arc<wgpu::Sampler>,

    // --- Solid Color Rendering ---
    solid_color_pipeline: wgpu::RenderPipeline,
    solid_color_bind_group_layout: wgpu::BindGroupLayout,
    cursor_render_pipeline: wgpu::RenderPipeline, // Added for cursor alpha blending
    dummy_white_texture: Arc<WgpuRenderableTexture>, // For fallback
}

// WGSL Shaders for Solid Color
const SOLID_COLOR_VS_MAIN: &str = r#"
@group(1) @binding(0) var<uniform> transform_matrix: mat3x3<f32>;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    let transformed_position = transform_matrix * vec3<f32>(position, 1.0);
    return vec4<f32>(transformed_position.xy, 0.0, 1.0);
}
"#;

const SOLID_COLOR_FS_MAIN: &str = r#"
@group(0) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
"#;

impl NovaWgpuRenderer {
    /// Creates a new `NovaWgpuRenderer`.
    ///
    /// # Arguments
    /// * `window_handle_target`: An object that implements `HasRawWindowHandle` and `HasRawDisplayHandle`
    ///                           (e.g., a `winit::window::Window`). This is used to create the WGPU surface.
    /// * `initial_size`: The initial physical size of the rendering surface.
    pub async fn new<WHT>(window_handle_target: &WHT, initial_size: Size<u32, Physical>) -> Result<Self>
    where
        WHT: HasRawWindowHandle + HasRawDisplayHandle,
    {
        let id = Uuid::new_v4();
        tracing::info!("Initializing NovaWgpuRenderer (id: {})", id);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), // Or specify Vulkan: wgpu::Backends::VULKAN
            dx12_shader_compiler: Default::default(),
            ..Default::default() // Added for wgpu 0.18+
        });

        // Surface creation needs to be done before adapter selection on some platforms.
        // The surface is unsafe to create because the window handle must remain valid.
        let surface = unsafe { instance.create_surface(window_handle_target) }.map_err(|e| {
            anyhow::anyhow!("Failed to create WGPU surface: {}", e)
        })?;
        tracing::info!("WGPU Surface created.");

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.ok_or_else(|| anyhow::anyhow!("Failed to find a suitable WGPU adapter"))?;

        tracing::info!("WGPU Adapter selected: {:?}", adapter.get_info());

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("NovaDE WGPU Device"),
                features: wgpu::Features::empty(), // TODO: Check adapter.features() for DMABUF related features if any become available in core WGPU.
                limits: wgpu::Limits::default(),
            },
            None, // Optional trace path
        ).await.map_err(|e| anyhow::anyhow!("Failed to request WGPU device: {}", e))?;
        tracing::info!("WGPU Device and Queue obtained. DMABUF features: Not explicitly requested in this version (WGPU core lacks direct DMABUF import, typically requires EGL interop).");

        let surface_caps = surface.get_capabilities(&adapter);
        // Choose a supported format, prefer sRGB if available.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb()) // .is_srgb() for wgpu 0.18+
            .unwrap_or_else(|| surface_caps.formats.first().copied().unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb)); // Fallback

        tracing::info!("Selected WGPU surface format: {:?}", surface_format);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_size.w as u32,
            height: initial_size.h as u32,
            present_mode: surface_caps.present_modes.first().copied().unwrap_or(wgpu::PresentMode::Fifo), // Default to Fifo
            alpha_mode: surface_caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto), // Added for wgpu 0.18+
            view_formats: vec![], // Added for wgpu 0.18+
        };

        surface.configure(&device, &surface_config);
        tracing::info!("WGPU Surface configured with size: {}x{}", initial_size.w, initial_size.h);

        // Load shaders
        let shader_source = Cow::Borrowed(include_str!("shaders/textured_quad.wgsl"));
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Textured Quad Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source),
        });

        // Create default sampler
        let default_sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));

        // Texture Bind Group Layout
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry { // Diffuse texture
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // Diffuse sampler
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Texture Bind Group Layout"),
        });

        // Texture Bind Group Layout (Group 0 for texture/sampler - remains unchanged)
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry { // Diffuse texture
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, // Vertex for potential size query, Frag for sampling
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // Diffuse sampler
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Texture Bind Group Layout (Group 0)"),
        });

        // Transform Bind Group Layout (Group 1 for transform matrix)
        let transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX, // Matrix used in vertex shader
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None, // mat3x3<f32> is 3*3*4 = 36 bytes. Or use BufferSize::new(36)
                    },
                    count: None,
                }
            ],
            label: Some("Transform Bind Group Layout (Group 1)"),
        });

        // Render Pipeline Layout for Textured Quads (now includes transform)
        let textured_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Textured Quad Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &transform_bind_group_layout], // Group 0 and Group 1
            push_constant_ranges: &[],
        });

        // Render Pipeline for Textured Quads (self.render_pipeline)
        // This pipeline is used for elements that should replace the content behind them (e.g. opaque Wayland surfaces)
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Textured Quad Render Pipeline (Replace)"),
            layout: Some(&textured_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format, // Use the surface's format
                    blend: Some(wgpu::BlendState::REPLACE), // Opaque surfaces replace what's behind.
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Cursor Render Pipeline (for alpha blending)
        let cursor_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cursor Render Pipeline (Alpha Blend)"),
            layout: Some(&textured_pipeline_layout), // Reuse layout from textured quads
            vertex: wgpu::VertexState {
                module: &shader_module, // Reuse vertex shader
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module, // Reuse fragment shader
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format, // Use the surface's format
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Enable alpha blending for cursors
                    // blend: Some(wgpu::BlendState { // More explicit version of ALPHA_BLENDING
                    //     color: wgpu::BlendComponent {
                    //         src_factor: wgpu::BlendFactor::SrcAlpha,
                    //         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    //         operation: wgpu::BlendOperation::Add,
                    //     },
                    //     alpha: wgpu::BlendComponent {
                    //         src_factor: wgpu::BlendFactor::One, // Often One for pre-multiplied, SrcAlpha for non-pre-multiplied
                    //         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    //         operation: wgpu::BlendOperation::Add,
                    //     },
                    // }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false, // Requires Features::DEPTH_CLIP_CONTROL
                conservative: false,    // Requires Features::CONSERVATIVE_RASTERIZATION
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create Quad Vertex Buffer
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create Quad Index Buffer
        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let quad_num_indices = QUAD_INDICES.len() as u32;

        tracing::info!("WGPU render pipeline and quad buffers created.");

        let final_device = Arc::new(device.clone());
        let final_queue = Arc::new(queue.clone());

        // --- Dummy White Texture for Fallbacks ---
        let dummy_texture_data = [255u8, 255, 255, 255]; // Single white pixel (RGBA)
        let dummy_wgpu_texture = final_device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dummy White Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        final_queue.write_texture(
            wgpu::ImageCopyTexture { texture: &dummy_wgpu_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &dummy_texture_data,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let dummy_white_renderable_texture = Arc::new(WgpuRenderableTexture::new(
            final_device.clone(),
            dummy_wgpu_texture, // This needs to be Arc<wgpu::Texture> or WgpuRenderableTexture needs to own it.
                                // For now, assuming WgpuRenderableTexture::new takes ownership or an Arc.
                                // Let's assume WgpuRenderableTexture::new takes the texture itself, not an Arc.
                                // The struct definition has Arc<wgpu::Texture>, so this should be fine.
            final_device.create_texture_view(&dummy_wgpu_texture, &wgpu::TextureViewDescriptor::default()), // View needs to be from the original texture object
            default_sampler.as_ref().clone(), // Sampler can be cloned from default_sampler
            1, 1, wgpu::TextureFormat::Rgba8UnormSrgb, None,
        ));


        // --- Solid Color Pipeline Setup ---
        let solid_color_shader_module = final_device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Color Shader Module"),
            source: wgpu::ShaderSource::Wgsl(format!("{}\n{}", SOLID_COLOR_VS_MAIN, SOLID_COLOR_FS_MAIN).into()),
        });

        let solid_color_bind_group_layout = final_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry { // Color uniform
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None, //wgpu::BufferSize::new(16), // vec4<f32> is 16 bytes
                    },
                    count: None,
                },
            ],
            label: Some("Solid Color Bind Group Layout"),
        });

        let solid_color_pipeline_layout = final_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Solid Color Pipeline Layout"),
            bind_group_layouts: &[&solid_color_bind_group_layout, &self.transform_bind_group_layout], // Added transform_bind_group_layout
            push_constant_ranges: &[],
        });

        let solid_color_pipeline = final_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Solid Color Render Pipeline"),
            layout: Some(&solid_color_pipeline_layout), // Use updated layout
            vertex: wgpu::VertexState {
                module: &solid_color_shader_module, // Shader now uses transform
                entry_point: "vs_main",
                buffers: &[Vertex::desc()], // Re-use quad vertex buffer description
            },
            fragment: Some(wgpu::FragmentState {
                module: &solid_color_shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format, // Use the surface's format
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(), // count: 1, mask: !0, alpha_to_coverage_enabled: false
            multiview: None,
        });
        tracing::info!("Solid color render pipeline created.");


        Ok(Self {
            id,
            instance,
            adapter,
            device: final_device,
            queue: final_queue,
            surface: Some(surface),
            surface_config: Some(surface_config),
            screen_size_physical: Size::from((initial_size.w as i32, initial_size.h as i32)),
            render_pipeline,
            quad_vertex_buffer,
            quad_index_buffer,
            quad_num_indices,
            texture_bind_group_layout,
            default_sampler,
            transform_bind_group_layout,
            solid_color_pipeline,
            solid_color_bind_group_layout,
            cursor_render_pipeline, // Store the new pipeline
            dummy_white_texture: dummy_white_renderable_texture,
        })
    }

    /// Resizes the WGPU surface and swapchain.
    /// Called when the output window is resized.
    pub fn resize(&mut self, new_size: Size<u32, Physical>) {
        if new_size.w > 0 && new_size.h > 0 {
            if let (Some(surface), Some(config)) = (self.surface.as_ref(), self.surface_config.as_mut()) {
                config.width = new_size.w;
                config.height = new_size.h;
                surface.configure(&self.device, config);
                self.screen_size_physical = Size::from((new_size.w as i32, new_size.h as i32));
                tracing::info!("WGPU surface resized to: {}x{}", new_size.w, new_size.h);
            }
        } else {
            tracing::warn!("WGPU surface resize requested with zero dimension: {}x{}", new_size.w, new_size.h);
        }
    }
}

// Basic implementation of FrameRenderer trait
impl FrameRenderer for NovaWgpuRenderer {
    fn id(&self) -> Uuid {
        self.id
    }

    fn render_frame<'iter_elements>(
        &mut self,
        elements: impl IntoIterator<Item = RenderElement<'iter_elements>>,
        _output_geometry: Rectangle<i32, Physical>, // Used for viewport/scissor if not full-screen
        _output_scale: f64, // Could be used for scaling elements if needed
    ) -> Result<(), RendererError> {
        let surface = self.surface.as_ref().ok_or_else(|| {
            RendererError::Generic("WGPU surface not available in render_frame".to_string())
        })?;

        let output_frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                tracing::warn!("WGPU surface outdated, reconfiguring.");
                let config = self.surface_config.as_ref().unwrap(); // Should exist
                surface.configure(&self.device, config);
                surface.get_current_texture().map_err(|e| {
                    RendererError::BufferSwapFailed(format!(
                        "Failed to get WGPU texture after reconfigure: {}",
                        e
                    ))
                })?
            }
            Err(e) => {
                return Err(RendererError::BufferSwapFailed(format!(
                    "Failed to get WGPU texture: {}",
                    e
                )));
            }
        };

        let view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("NovaDE WGPU Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("NovaDE Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0, g: 0.0, b: 0.0, a: 1.0, // Clear to black for better visibility of surfaces
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None, // Added for wgpu 0.18+
                occlusion_query_set: None, // Added for wgpu 0.18+
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for element in elements {
                match element {
                    RenderElement::WaylandSurface { surface_data_arc, geometry, .. } => {
                        // The surface_data_arc is Arc<Mutex<WaylandSurfaceData>>
                        // Lock it to access WaylandSurfaceData
                        let surface_data_guard = surface_data_arc.lock().unwrap();

                        if let Some(renderable_texture_arc) = surface_data_guard.texture_handle.as_ref() {
                            // Downcast Arc<dyn RenderableTexture> to &WgpuRenderableTexture
                            if let Some(wgpu_tex) = renderable_texture_arc.as_any().downcast_ref::<WgpuRenderableTexture>() {
                                // Create a new bind group for this texture
                                let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    layout: &self.texture_bind_group_layout,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: wgpu::BindingResource::TextureView(wgpu_tex.view()),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: wgpu::BindingResource::Sampler(wgpu_tex.sampler()),
                                        },
                                    ],
                                    label: Some("Wayland Surface Texture Bind Group"),
                                });
                                render_pass.set_bind_group(0, &texture_bind_group, &[]);
                            } else {
                                // Fallback or error if downcast fails: render a magenta placeholder using solid_color_pipeline
                                tracing::warn!("Failed to downcast RenderableTexture to WgpuRenderableTexture for WaylandSurface. Rendering placeholder.");
                                render_pass.set_pipeline(&self.solid_color_pipeline); // Switch to solid color
                                let color = [1.0f32, 0.0, 1.0, 1.0]; // Magenta
                                let color_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Fallback Color Uniform Buffer"),
                                    contents: bytemuck::cast_slice(&color),
                                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                                });
                                let fallback_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    layout: &self.solid_color_bind_group_layout,
                                    entries: &[wgpu::BindGroupEntry { binding: 0, resource: color_buffer.as_entire_binding() }],
                                    label: Some("Fallback Color Bind Group"),
                                });
                                render_pass.set_bind_group(0, &fallback_bind_group, &[]);
                                // Ensure pipeline is reset for next element if it was changed
                                // This draw call will use the solid_color_pipeline.
                                // Need to reset to self.render_pipeline if next element is textured.
                            }

                            // Transformation for WaylandSurface (common for textured or placeholder)
                            let screen_w = self.screen_size_physical.w as f32;
                            let screen_h = self.screen_size_physical.h as f32;
                            let quad_width = geometry.size.w as f32;
                            let quad_height = geometry.size.h as f32;

                            let scale_x_ndc = quad_width / screen_w;
                            let scale_y_ndc = quad_height / screen_h;

                            let top_left_x_logical = geometry.loc.x as f32;
                            let top_left_y_logical = geometry.loc.y as f32;

                            let pos_x_ndc = (top_left_x_logical / screen_w) * 2.0 - 1.0;
                            let pos_y_ndc = (top_left_y_logical / screen_h) * -2.0 + 1.0;

                            let final_translate_x_ndc = pos_x_ndc + scale_x_ndc;
                            let final_translate_y_ndc = pos_y_ndc - scale_y_ndc;

                            let transform_matrix_col_major = [
                                scale_x_ndc, 0.0, 0.0,
                                0.0, scale_y_ndc, 0.0,
                                final_translate_x_ndc, final_translate_y_ndc, 1.0,
                            ];

                            let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("WaylandSurface Transform Uniform Buffer"),
                                contents: bytemuck::cast_slice(&transform_matrix_col_major),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            });

                            let transform_bind_group_group1 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &self.transform_bind_group_layout, // Layout for group 1 (transform)
                                entries: &[wgpu::BindGroupEntry { binding: 0, resource: transform_buffer.as_entire_binding() }],
                                label: Some("WaylandSurface Transform Bind Group (Group 1)"),
                            });

                            render_pass.set_bind_group(1, &transform_bind_group_group1, &[]); // Set transform for group 1
                            render_pass.draw_indexed(0..self.quad_num_indices, 0, 0..1);

                            // Reset to textured pipeline if it was changed by fallback
                            render_pass.set_pipeline(&self.render_pipeline);
                        } else {
                            tracing::trace!("WaylandSurface element (geom {:?}) has no texture_handle, skipping render.", geometry);
                        }
                    }
                    RenderElement::SolidColor { color, geometry } => {
                        render_pass.set_pipeline(&self.solid_color_pipeline);
                        let color_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Solid Color Uniform Buffer"),
                            contents: bytemuck::cast_slice(&color),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });
                        let solid_color_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.solid_color_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry { binding: 0, resource: color_buffer.as_entire_binding() }],
                            label: Some("Solid Color Bind Group"),
                        });
                        render_pass.set_bind_group(0, &solid_color_bind_group, &[]);

                        // Transformation for SolidColor
                        let screen_w = self.screen_size_physical.w as f32;
                        let screen_h = self.screen_size_physical.h as f32;
                        let quad_width = geometry.size.w as f32;
                        let quad_height = geometry.size.h as f32;
                        let scale_x_ndc = quad_width / screen_w;
                        let scale_y_ndc = quad_height / screen_h;
                        let top_left_x_logical = geometry.loc.x as f32;
                        let top_left_y_logical = geometry.loc.y as f32;
                        let pos_x_ndc = (top_left_x_logical / screen_w) * 2.0 - 1.0;
                        let pos_y_ndc = (top_left_y_logical / screen_h) * -2.0 + 1.0;
                        let final_translate_x_ndc = pos_x_ndc + scale_x_ndc;
                        let final_translate_y_ndc = pos_y_ndc - scale_y_ndc;
                        let transform_matrix_col_major = [
                            scale_x_ndc, 0.0, 0.0, 0.0, scale_y_ndc, 0.0, final_translate_x_ndc, final_translate_y_ndc, 1.0,
                        ];
                        let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("SolidColor Transform Uniform Buffer"),
                            contents: bytemuck::cast_slice(&transform_matrix_col_major),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });
                        let transform_bind_group_solid = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.transform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry { binding: 0, resource: transform_buffer.as_entire_binding() }],
                            label: Some("SolidColor Transform Bind Group (Group 1)"),
                        });
                        render_pass.set_bind_group(1, &transform_bind_group_solid, &[]);
                        render_pass.draw_indexed(0..self.quad_num_indices, 0, 0..1);
                        // Reset to textured pipeline for next element
                        render_pass.set_pipeline(&self.render_pipeline);
                    }
                    RenderElement::Cursor { texture_arc, position_logical, hotspot_logical, .. } => {
                        // --- Cursor Rendering ---
                        // The cursor's `position_logical` is its top-left point on the screen.
                        // The `hotspot_logical` is an offset from the cursor image's top-left corner
                        // to the actual pointer location (e.g., the tip of an arrow cursor).
                        // To render the cursor image correctly, its top-left on the screen should be:
                        // `position_logical - hotspot_logical`.
                        let actual_top_left_x = position_logical.x as f32 - hotspot_logical.x as f32;
                        let actual_top_left_y = position_logical.y as f32 - hotspot_logical.y as f32;

                        let (tex_view, tex_sampler, tex_width_u32, tex_height_u32) =
                            if let Some(wgpu_tex) = texture_arc.as_any().downcast_ref::<WgpuRenderableTexture>() {
                                if wgpu_tex.width_px() == 0 || wgpu_tex.height_px() == 0 {
                                    tracing::warn!("Cursor texture has zero dimension, using fallback.");
                                     (self.dummy_white_texture.view(), self.dummy_white_texture.sampler(), self.dummy_white_texture.width_px(), self.dummy_white_texture.height_px())
                                } else {
                                    (wgpu_tex.view(), wgpu_tex.sampler(), wgpu_tex.width_px(), wgpu_tex.height_px())
                                }
                            } else {
                                tracing::warn!("Failed to downcast RenderableTexture to WgpuRenderableTexture for cursor. Using dummy texture.");
                                (self.dummy_white_texture.view(), self.dummy_white_texture.sampler(), self.dummy_white_texture.width_px(), self.dummy_white_texture.height_px())
                            };

                        let tex_width = tex_width_u32 as f32;
                        let tex_height = tex_height_u32 as f32;

                        let cursor_tex_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(tex_view) },
                                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(tex_sampler) },
                            ],
                            label: Some("Cursor Texture Bind Group"),
                        });

                        // Use the dedicated cursor pipeline for alpha blending
                        render_pass.set_pipeline(&self.cursor_render_pipeline);
                        render_pass.set_bind_group(0, &cursor_tex_bind_group, &[]);

                        // Transformation for cursor quad:
                        // The quad vertices are from -1.0 to 1.0 (a 2x2 square centered at origin).
                        // 1. Scale: Scale the 2x2 quad to match the texture's dimensions in NDC.
                        //    - Quad width is 2, so scale_x_model = (tex_width / screen_width) / 2.0 * 2.0 = tex_width / screen_width.
                        //    - Or, simply, scale by (tex_width / screen_width, tex_height / screen_height).
                        //      The shader receives vertex positions in [-1,1]. If matrix scales by S, result is [-S, S].
                        //      So, effective size becomes 2S. We want effective size to be tex_width_ndc. So 2S = tex_width_ndc => S = tex_width_ndc / 2.
                        //      No, if shader does `matrix * pos`, and pos is [-1,1], then scale factor in matrix should be (width_ndc/2, height_ndc/2).
                        let screen_w = self.screen_size_physical.w as f32;
                        let screen_h = self.screen_size_physical.h as f32;

                        let quad_scale_x = (tex_width / screen_w);
                        let quad_scale_y = (tex_height / screen_h);

                        // 2. Translate: Position the quad's center.
                        //    The quad's top-left should be at `actual_top_left_x/y`.
                        //    So, its center should be at `actual_top_left_x + tex_width / 2`, `actual_top_left_y + tex_height / 2`.
                        let center_x_screen = actual_top_left_x + tex_width / 2.0;
                        let center_y_screen = actual_top_left_y + tex_height / 2.0;

                        // Convert center to NDC
                        let translate_x_ndc = (center_x_screen / screen_w) * 2.0 - 1.0;
                        let translate_y_ndc = (center_y_screen / screen_h) * -2.0 + 1.0; // Y is inverted

                        // Final matrix (column-major for WGSL)
                        // Scales the [-1,1] quad to the desired size, then translates its center.
                        let final_transform_col_major = [
                            quad_scale_x, 0.0, 0.0,          // Column 1: scales X
                            0.0, quad_scale_y, 0.0,          // Column 2: scales Y
                            translate_x_ndc, translate_y_ndc, 1.0, // Column 3: translates
                        ];

                        let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Cursor Transform Uniform Buffer"),
                            contents: bytemuck::cast_slice(&final_transform_col_major),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });

                        let transform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.transform_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: transform_buffer.as_entire_binding(),
                                }
                            ],
                            label: Some("Cursor Transform Bind Group"),
                        });
                        render_pass.set_bind_group(1, &transform_bind_group, &[]);

                        render_pass.draw_indexed(0..self.quad_num_indices, 0, 0..1);

                        // Reset pipeline to default for other elements (Wayland surfaces, etc.)
                        render_pass.set_pipeline(&self.render_pipeline);

                        tracing::trace!(
                            "Rendered Cursor: pos_logical=({},{}), hotspot=({},{}), actual_top_left=({},{}), tex_size=({}x{}), screen_size=({}x{}), quad_scale=({}, {}), ndc_center=({}, {})",
                            position_logical.x, position_logical.y, hotspot_logical.x, hotspot_logical.y,
                            actual_top_left_x, actual_top_left_y, tex_width, tex_height,
                            screen_w, screen_h, quad_scale_x, quad_scale_y, translate_x_ndc, translate_y_ndc
                        );
                    }
                }
            }
        } // render_pass is dropped here

        self.queue.submit(std::iter::once(encoder.finish()));

        drop(output_frame);

        Ok(())
    }

    // Update present_frame if needed. With implicit presentation on SurfaceTexture drop,
    // this might become a no-op or handle other vsync-related logic if any.
    fn present_frame(&mut self) -> Result<(), RendererError> {
        // As of wgpu 0.13+ (and 0.18), presentation happens when the SurfaceTexture is dropped.
        // If render_frame ensures the SurfaceTexture is dropped (e.g. by not storing it beyond its scope),
        // then this method might not need to do anything explicit for presentation.
        tracing::debug!("NovaWgpuRenderer::present_frame called (presentation is implicit on SurfaceTexture drop).");
        Ok(())
    }

    fn screen_size(&self) -> Size<i32, Physical> {
        self.screen_size_physical
    }

    fn create_texture_from_shm(
        &mut self,
        buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        let (shm_data, width, height, wl_shm_format) =
            match with_buffer_contents_data(buffer) {
                Ok(data) => data,
                Err(e) => {
                    let err_msg = format!("Failed to access SHM buffer contents: {}", e);
                    tracing::error!("{}", err_msg);
                    return Err(RendererError::InvalidBufferType(err_msg));
                }
            };

        tracing::debug!(
            "Creating WGPU texture from SHM buffer: format={:?}, dims={}x{}",
            wl_shm_format, width, height
        );

        let wgpu_texture_format = match wl_shm_format {
            WlShmFormat::Argb8888 => wgpu::TextureFormat::Bgra8UnormSrgb, // Often preferred for display
            WlShmFormat::Xrgb8888 => wgpu::TextureFormat::Bgra8UnormSrgb,
            WlShmFormat::Abgr8888 => wgpu::TextureFormat::Rgba8UnormSrgb,
            WlShmFormat::Xbgr8888 => wgpu::TextureFormat::Rgba8UnormSrgb,
            // Add other formats as needed, e.g., non-sRGB versions or different channel orders
            _ => {
                let err_msg = format!(
                    "Unsupported SHM format for WGPU texture creation: {:?}. Dimensions: {}x{}",
                    wl_shm_format, width, height
                );
                tracing::error!("{}", err_msg);
                return Err(RendererError::UnsupportedPixelFormat(err_msg));
            }
        };

        // Create a WGPU texture
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("NovaDE SHM WGPU Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[], // Added for wgpu 0.18+
        });
        tracing::debug!("Created WGPU texture {:?} for SHM data.", texture.global_id());


        // Write data to the texture
        // WGPU expects data in rows of `bytes_per_row` which must be a multiple of `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`.
        // For SHM buffers, data is typically tightly packed (stride = width * bytes_per_pixel).
        // We need to ensure this alignment or copy row by row if necessary.
        // For common formats like ARGB8888 (4 bytes/pixel), width * 4 is often aligned if width is reasonable.
        let bytes_per_pixel = match wl_shm_format {
            WlShmFormat::Argb8888 | WlShmFormat::Xrgb8888 | WlShmFormat::Abgr8888 | WlShmFormat::Xbgr8888 => 4,
            _ => return Err(RendererError::Generic("Unhandled bytes_per_pixel for SHM format".to_string())),
        };
        let bytes_per_row = width * bytes_per_pixel;

        // Ensure bytes_per_row is aligned to wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
        // This check is important. If not aligned, a staging buffer or row-by-row copy is needed.
        // For simplicity, this example assumes it's aligned or that `write_texture` handles it
        // if `bytes_per_row` can be specified and is not None.
        // Wgpu 0.18 `write_texture` takes `bytes_per_row: Option<u32>`. If None, assumes tightly packed.
        // If the SHM data is not aligned as WGPU expects for a direct copy, this will fail or be incorrect.
        // A robust solution might involve a staging buffer if alignment mismatches.

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All, // Or .Color if applicable
            },
            shm_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row), // If None, assumes tightly packed. Must be multiple of 256.
                rows_per_image: Some(height),    // If None, assumes tightly packed
            },
            texture_size,
        );
        tracing::debug!("SHM data written to WGPU texture {:?}.", texture.global_id());

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor { // Basic sampler
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        tracing::debug!("Created view and sampler for WGPU SHM texture {:?}.", texture.global_id());

        Ok(Box::new(WgpuRenderableTexture::new(
            self.device.clone(), // Arc<wgpu::Device>
            texture,
            view,
            sampler,
            width,
            height,
            wgpu_texture_format,
            None, // SHM buffers don't typically have a FourCC
        )))
    }

    fn create_texture_from_dmabuf(
        &mut self,
        dmabuf: &smithay::backend::allocator::dmabuf::Dmabuf,
    ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        tracing::warn!(
            "NovaWgpuRenderer::create_texture_from_dmabuf called for DMABUF with {} planes (modifier: {:?}, format: {:?}, size: {}x{}). Feature not yet implemented.",
            dmabuf.num_planes(),
            dmabuf.modifier(),
            dmabuf.format().code, // DrmFourcc code
            dmabuf.width(),
            dmabuf.height()
        ); // Note: Dmabuf part is mostly a stub that returns error due to wgpu 0.7 limitations.

        // --- EGL and GLES setup ---
        // This is highly dependent on how WGPU was initialized and what backend it's using.
        // We need access to the EGLDisplay and potentially EGLContext WGPU is using.

        // 1. Get EGLDisplay
        // Winit's RawDisplayHandle might give us this if it's an EGL display.
        // However, NovaWgpuRenderer::new takes a window_handle_target, but doesn't store the display handle.
        // This is a problem. We need to assume it's available globally or modify `new` to store it.
        // For now, let's try to get it dynamically. This is a common challenge.
        // A common way is eglGetCurrentDisplay(), but that requires an EGL context to be current.

        // Let's assume we need to initialize EGL here, which might conflict if WGPU already did.
        // A safer approach would be to get it from WGPU's HAL if possible.
        // Given wgpu 0.7, direct HAL access to EGLDisplay/Context is not straightforward.

        // Try to load EGL functions. We need EGL1_4 for KHR_image_base.
        // It's crucial that this EGL instance is compatible with the one WGPU/Winit might be using.
        // If WGPU is initialized with a different EGL display/context, operations might fail or be undefined.
        let egl_instance = match EglInstance::new(egl::DynamicLibrary::open_default()) {
            Ok(instance) => instance,
            Err(e) => {
                let err_msg = format!("Failed to load EGL library: {}", e);
                tracing::error!("{}", err_msg);
                return Err(RendererError::Generic(err_msg)); // Consider a more specific error like EglNotAvailable
            }
        };
        tracing::info!("EGL instance loaded dynamically via default library path.");

        // Get EGLDisplay:
        // Ideally, this should come from `raw_display_handle()` of the windowing system
        // that WGPU is also using, to ensure compatibility.
        // Using EGL_DEFAULT_DISPLAY is a fallback and might not be the correct display if multiple are present
        // or if WGPU was initialized on a non-default display.
        let egl_display = unsafe { egl_instance.get_display(egl_types::DEFAULT_DISPLAY) };
        if egl_display == egl_types::NO_DISPLAY {
            let egl_error = unsafe { egl_instance.get_error() };
            let err_msg = format!("Failed to get EGL default display. EGL Error: {:#x}", egl_error);
            tracing::error!("{}", err_msg);
            return Err(RendererError::Generic(err_msg)); // Consider EglDisplayNotAvailable
        }

        // Initialize EGL:
        // This is generally safe to call multiple times; it's idempotent.
        let mut major = 0i32;
        let mut minor = 0i32;
        if unsafe { egl_instance.initialize(egl_display, &mut major, &mut minor) } == egl_types::FALSE {
            let egl_error = unsafe { egl_instance.get_error() };
            let err_msg = format!("Failed to initialize EGL display. EGL Error: {:#x}", egl_error);
            tracing::error!("{}", err_msg);
            return Err(RendererError::Generic(err_msg)); // Consider EglInitializationFailed
        }
        tracing::info!("EGL initialized: version {}.{}. Display: {:?}", major, minor, egl_display);

        // Check for required EGL extensions:
        // These are crucial for DMABUF import.
        let extensions_str = unsafe {
            let ptr = egl_instance.query_string(egl_display, egl_types::EXTENSIONS);
            if ptr.is_null() {
                let egl_error = unsafe { egl_instance.get_error() };
                let err_msg = format!("Failed to query EGL extensions string. EGL Error: {:#x}", egl_error);
                tracing::error!("{}", err_msg);
                return Err(RendererError::Generic(err_msg)); // Consider EglExtensionQueryFailed
            }
            std.ffi::CStr::from_ptr(ptr).to_str().unwrap_or("") // unwrap_or: if CStr is not valid UTF-8
        };
        tracing::debug!("EGL Extensions: {}", extensions_str);

        if !extensions_str.contains("EGL_KHR_image_base") {
            return Err(RendererError::Unsupported("EGL extension EGL_KHR_image_base not supported".to_string()));
        }
        if !extensions_str.contains("EGL_EXT_image_dma_buf_import") {
            return Err(RendererError::Unsupported("EGL extension EGL_EXT_image_dma_buf_import not supported".to_string()));
        }
        let has_modifier_support = extensions_str.contains("EGL_EXT_image_dma_buf_import_modifiers");
        tracing::info!("EGL DMABUF import extensions found. Modifier support: {}", has_modifier_support);

        // --- EGLImage Creation ---
        let mut egl_attribs = Vec::<egl_types::Attrib>::new();
        egl_attribs.push(egl_types::WIDTH as egl_types::Attrib);
        egl_attribs.push(dmabuf.width() as egl_types::Attrib);
        egl_attribs.push(egl_types::HEIGHT as egl_types::Attrib);
        egl_attribs.push(dmabuf.height() as egl_types::Attrib);
        egl_attribs.push(egl_types::LINUX_DRM_FOURCC_EXT as egl_types::Attrib);
        egl_attribs.push(dmabuf.format().code as egl_types::Attrib);

        let num_planes = dmabuf.num_planes();
        if num_planes == 0 || num_planes > drm::control::MAX_PLANES { // drm::control::MAX_PLANES is usually 4
             let err_msg = format!("Invalid number of DMABUF planes: {}. Must be > 0 and <= {}.", num_planes, drm::control::MAX_PLANES);
             tracing::error!("{}", err_msg);
             return Err(RendererError::InvalidBufferFormat(err_msg));
        }

        const PLANE_ATTRIBS_CORE: [(i32, (i32, i32)); 3] = [
            (egl_types::DMA_BUF_PLANE0_FD_EXT, (egl_types::DMA_BUF_PLANE0_OFFSET_EXT, egl_types::DMA_BUF_PLANE0_PITCH_EXT)),
            (egl_types::DMA_BUF_PLANE1_FD_EXT, (egl_types::DMA_BUF_PLANE1_OFFSET_EXT, egl_types::DMA_BUF_PLANE1_PITCH_EXT)),
            (egl_types::DMA_BUF_PLANE2_FD_EXT, (egl_types::DMA_BUF_PLANE2_OFFSET_EXT, egl_types::DMA_BUF_PLANE2_PITCH_EXT)),
        ];
        const PLANE_ATTRIBS_MODIFIER: [(i32, i32); 3] = [
            (egl_types::DMA_BUF_PLANE0_MODIFIER_LO_EXT, egl_types::DMA_BUF_PLANE0_MODIFIER_HI_EXT),
            (egl_types::DMA_BUF_PLANE1_MODIFIER_LO_EXT, egl_types::DMA_BUF_PLANE1_MODIFIER_HI_EXT),
            (egl_types::DMA_BUF_PLANE2_MODIFIER_LO_EXT, egl_types::DMA_BUF_PLANE2_MODIFIER_HI_EXT),
        ];


        for i in 0..num_planes {
            let plane_fd = dmabuf.plane_fd(i).map_err(|e| RendererError::InvalidBufferFormat(format!("Failed to get DMABUF plane {} FD: {}", i, e)))?;
            let plane_offset = dmabuf.plane_offset(i).map_err(|e| RendererError::InvalidBufferFormat(format!("Failed to get DMABUF plane {} offset: {}", i, e)))?;
            let plane_stride = dmabuf.plane_stride(i).map_err(|e| RendererError::InvalidBufferFormat(format!("Failed to get DMABUF plane {} stride: {}", i, e)))?;

            egl_attribs.push(PLANE_ATTRIBS_CORE[i].0 as egl_types::Attrib);
            egl_attribs.push(plane_fd as egl_types::Attrib);
            egl_attribs.push(PLANE_ATTRIBS_CORE[i].1.0 as egl_types::Attrib);
            egl_attribs.push(plane_offset as egl_types::Attrib);
            egl_attribs.push(PLANE_ATTRIBS_CORE[i].1.1 as egl_types::Attrib);
            egl_attribs.push(plane_stride as egl_types::Attrib);

            if let Some(modifier) = dmabuf.modifier() {
                if has_modifier_support {
                    if i < PLANE_ATTRIBS_MODIFIER.len() { // Ensure we don't go out of bounds for modifier array
                        egl_attribs.push(PLANE_ATTRIBS_MODIFIER[i].0 as egl_types::Attrib);
                        egl_attribs.push((modifier.into_raw() & 0xFFFFFFFF) as egl_types::Attrib);
                        egl_attribs.push(PLANE_ATTRIBS_MODIFIER[i].1 as egl_types::Attrib);
                        egl_attribs.push((modifier.into_raw() >> 32) as egl_types::Attrib);
                    }
                } else if i == 0 { // Log warning only once for the first plane
                    tracing::warn!("DMABUF has modifier ({:?}) but EGL_EXT_image_dma_buf_import_modifiers not supported. Importing without modifier.", modifier);
                }
            }
        }
        egl_attribs.push(egl_types::NONE as egl_types::Attrib); // Terminate the list

        // Create EGLImageKHR:
        // For EGL_LINUX_DMA_BUF_EXT target, eglContext should be EGL_NO_CONTEXT.
        // clientBuffer is also typically NULL or not used for this target.
        let egl_image = unsafe {
            egl_instance.create_image_khr(
                egl_display,
                egl_types::NO_CONTEXT, // Must be EGL_NO_CONTEXT for EGL_LINUX_DMA_BUF_EXT
                egl_types::LINUX_DMA_BUF_EXT, // Target type
                std::ptr::null_mut(), // client_buffer (not used for this target)
                egl_attribs.as_ptr(), // Pointer to attribute list
            )
        };

        if egl_image == egl_types::NO_IMAGE_KHR {
            let egl_error = unsafe { egl_instance.get_error() };
            let err_msg = format!("Failed to create EGLImageKHR from DMABUF. EGL Error: {:#x}", egl_error);
            tracing::error!("{}", err_msg);
            // Consider more specific error mapping based on egl_error if possible
            return Err(RendererError::Generic(err_msg)); // E.g. EglImageCreationFailed
        }
        tracing::info!("EGLImageKHR created successfully from DMABUF attributes.");

        // --- Attempt to use EGLImage with GLES and WGPU (Known Limitation with wgpu 0.7) ---

        // At this point, an EGLImageKHR (egl_image) has been created.
        // The next conceptual steps would be:
        // 1. Ensure WGPU is using an OpenGL ES (GLES) backend.
        // 2. Obtain the `glow::Context` that WGPU's GLES backend is using.
        // 3. Create a GLES texture (`glow::Texture`).
        // 4. Bind this GLES texture to the `GL_TEXTURE_EXTERNAL_OES` target.
        // 5. Use `glEGLImageTargetTexture2DOES` to link the EGLImage data to the GLES texture.
        // 6. Wrap this GLES texture ID into a `wgpu::Texture` using WGPU's HAL (Hardware Abstraction Layer).

        // **Roadblock with wgpu 0.7:**
        // Steps 2 and 6 are problematic with wgpu 0.7's public API.
        // - Step 2: WGPU 0.7 does not provide a stable/public way to get its internal `glow::Context`.
        //           Creating a new `glow::Context` here would not be the same one WGPU uses, leading to issues.
        // - Step 6: WGPU 0.7's HAL, while present, does not offer straightforward public functions
        //           like `Device::from_hal_gles_texture` (found in later versions) to import an arbitrary
        //           GLES texture ID into WGPU's tracking.
        //
        // Therefore, even if the EGLImage is valid, integrating it into WGPU as a usable texture
        // is not feasible without internal HAL knowledge or unsupported operations for this WGPU version.

        // Check WGPU adapter backend to confirm if it's even GL.
        let adapter_info = self.adapter.get_info();
        if adapter_info.backend != wgpu::Backend::Gl {
            unsafe { egl_instance.destroy_image_khr(egl_display, egl_image); } // Clean up EGLImage
            let err_msg = format!(
                "WGPU backend is not OpenGL (it's {:?}). DMABUF import via EGL/GLES is not applicable.",
                adapter_info.backend
            );
            tracing::warn!("{}", err_msg); // Warn, as EGLImage was made, but backend mismatch.
            return Err(RendererError::Unsupported(err_msg));
        }

        // Even if the backend is GL, the interop issue remains.
        // We can load `glEGLImageTargetTexture2DOES` to show the EGL side is ready,
        // but we can't proceed with creating and binding a GLES texture *within WGPU's context*.
        let gl_egl_image_target_texture_2d_oes_ptr = unsafe {
            egl_instance.get_proc_address("glEGLImageTargetTexture2DOES")
        };

        if gl_egl_image_target_texture_2d_oes_ptr.is_null() {
            unsafe { egl_instance.destroy_image_khr(egl_display, egl_image); } // Clean up EGLImage
            let egl_error = unsafe { egl_instance.get_error() }; // Unlikely to be an EGL error, more like function not found.
            let err_msg = format!("EGL function glEGLImageTargetTexture2DOES not found. EGL GetProcAddress Error: {:#x}", egl_error);
            tracing::error!("{}", err_msg);
            return Err(RendererError::Unsupported(err_msg));
        }
        // Transmuting to function pointer is not needed here as we won't call it without a glow::Context.
        tracing::info!("EGL function glEGLImageTargetTexture2DOES loaded successfully.");

        // Cleanup the created EGLImage as we cannot proceed further.
        unsafe {
            if egl_instance.destroy_image_khr(egl_display, egl_image) == egl_types::FALSE {
                let egl_error = unsafe { egl_instance.get_error() };
                tracing::warn!("Failed to destroy EGLImageKHR. EGL Error: {:#x}", egl_error);
            } else {
                tracing::info!("EGLImageKHR destroyed as GLES/WGPU interop is not supported in this version.");
            }
        }

        let final_error_message = "DMABUF EGLImage created successfully, but GLES texture import and subsequent WGPU texture wrapping is not supported with this WGPU version (wgpu 0.7) due to HAL limitations.".to_string();
        tracing::error!("{}", final_error_message);
        Err(RendererError::Unsupported(final_error_message))
    }
}

// Required for WGPU surface creation from winit window
// unsafe impl<WHT: HasRawWindowHandle + HasRawDisplayHandle> HasRawWindowHandle for NovaWgpuRenderer {
//     fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
//         // This is problematic. NovaWgpuRenderer itself doesn't own the window.
//         // It needs to be constructed with a handle.
//         // This impl is likely incorrect or indicates a design flaw if the renderer
//         // itself is passed as the source of the handle.
//         // The surface is created using an external handle.
//         // Let's remove this impl for now, it's not needed on the renderer struct itself.
//         panic!("NovaWgpuRenderer does not directly provide a RawWindowHandle. It uses one at creation.");
//     }
// }
// unsafe impl<WHT: HasRawWindowHandle + HasRawDisplayHandle> HasRawDisplayHandle for NovaWgpuRenderer {
//     fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
//         panic!("NovaWgpuRenderer does not directly provide a RawDisplayHandle. It uses one at creation.");
//     }
// }
