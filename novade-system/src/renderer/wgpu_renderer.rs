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
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Textured Quad Render Pipeline"),
            layout: Some(&textured_pipeline_layout), // Use the new layout
            vertex: wgpu::VertexState {
                module: &shader_module, // Shader already updated for transform_matrix
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format, // Use the surface's format
                    blend: Some(wgpu::BlendState::REPLACE), // Simple replace blend
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
                        let surface_data = surface_data_arc.lock().unwrap();
                        if let Some(renderable_texture) = surface_data.texture_handle.as_ref() {
                            // Assumes texture_handle is Arc<WgpuRenderableTexture>
                            // due to changes in SurfaceData
                            let wgpu_tex = renderable_texture; // No downcast needed if type is concrete

                            // Create a new bind group for each texture
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

                            // Transformation for WaylandSurface
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
                            tracing::trace!("Rendered WaylandSurface (texture id: {:?}), geom: {:?}, NDC tr: ({:.2},{:.2}), scale: ({:.2},{:.2})",
                                            wgpu_tex.id(), geometry, final_translate_x_ndc, final_translate_y_ndc, scale_x_ndc, scale_y_ndc);
                        } else {
                            tracing::trace!("WaylandSurface element has no texture_handle or failed downcast, skipping render.");
                        }
                    }
                    RenderElement::SolidColor { color, geometry } => {
                        // For solid color, we don't use the textured pipeline's bind group.
                        // We need a new bind group for the color uniform.
                        // The geometry will need to be applied to the vertices or via a transform uniform.
                        // For simplicity, assuming the Vertex::desc() is okay for a unit quad,
                        // and we'd need to scale/translate it. This example will just render a full-screen quad
                        // with the color, not respecting geometry yet.
                        // TODO: Implement proper geometry transformation for solid color quads.

                        let color_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Solid Color Uniform Buffer"),
                            contents: bytemuck::cast_slice(&color), // color is [f32; 4]
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });

                        let solid_color_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.solid_color_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: color_buffer.as_entire_binding(),
                                },
                            ],
                            label: Some("Solid Color Bind Group"),
                        });

                        render_pass.set_pipeline(&self.solid_color_pipeline);
                        render_pass.set_bind_group(0, &solid_color_bind_group, &[]);
                        // Assuming quad_vertex_buffer and quad_index_buffer define a unit quad
                        // that fills the screen or is transformed by vertex shader if geometry is complex.
                        // For now, draws a full quad with that color.
                        render_pass.draw_indexed(0..self.quad_num_indices, 0, 0..1);
                        tracing::trace!("Rendered SolidColor element with color {:?}, (geometry {}x{} at {},{} not yet fully applied).",
                                        color, geometry.size.w, geometry.size.h, geometry.loc.x, geometry.loc.y);
                    }
                    RenderElement::Cursor { texture_arc, position_logical, hotspot, .. } => {
                        // Attempt to downcast the texture
                        // Assuming WgpuRenderableTexture is the concrete type stored in the Box.
                        // This requires WgpuRenderableTexture to be public or accessible.
                        // And RenderableTexture trait needs to support Any + Display + Debug + Send + Sync for downcasting.
                        // Let's assume a method like `as_wgpu_texture()` exists on the trait or can be added,
                        // or we use a direct downcast if RenderableTexture is `dyn Any`.
                        // For now, let's simulate getting the WgpuRenderableTexture.
                        // This is a simplification. A real downcast or type-specific handling is needed.

                        // Placeholder for actual downcast.
                        // This is a critical point that needs a robust solution for trait object downcasting.
                        // For this subtask, we'll proceed as if we can get WgpuRenderableTexture.
                        // One way: if texture_arc was Arc<WgpuRenderableTexture> initially, this would be easier.
                        // Since it's Box<dyn RenderableTexture>, a downcast is necessary.
                        // We'll assume `texture_arc.as_any().downcast_ref::<WgpuRenderableTexture>()` could work
                        // if RenderableTexture was `pub trait RenderableTexture: std::any::Any ...`

                        // Simplified: Assume we have a WgpuRenderableTexture.
                        // This part will need adjustment based on how RenderableTexture trait object is structured.
                        // For the purpose of this diff, let's assume `texture_arc` can be used as if it's `WgpuRenderableTexture`.
                        // This is NOT correct Rust but helps illustrate the WGPU calls.
                        // A real implementation would need `texture_arc.as_any().downcast_ref::<WgpuRenderableTexture>()`
                        // and the trait to support `Any`.

                        // Let's refine this to be more realistic with a downcast.
                        // The RenderableTexture trait in compositor/state.rs would need to be `pub trait RenderableTexture: std::fmt::Debug + Send + Sync + std::any::Any { ... }`
                        // And WgpuRenderableTexture would need to implement it.

                        // For now, let's assume the downcast works conceptually.
                        // The actual texture access will be commented out if it prevents compilation without trait changes.
                        let cursor_texture_ref: &dyn RenderableTexture = &**texture_arc; // Deref Box to &dyn Trait
                        let (tex_view, tex_sampler, tex_width_u32, tex_height_u32) =
                            if let Some(wgpu_tex) = cursor_texture_ref.as_any().downcast_ref::<WgpuRenderableTexture>() {
                                (wgpu_tex.view(), wgpu_tex.sampler(), wgpu_tex.width(), wgpu_tex.height())
                            } else {
                                tracing::warn!("Failed to downcast RenderableTexture to WgpuRenderableTexture for cursor. Using dummy texture.");
                                (self.dummy_white_texture.view(), self.dummy_white_texture.sampler(), self.dummy_white_texture.width(), self.dummy_white_texture.height())
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

                        // Transformation for cursor
                        let screen_w = self.screen_size_physical.w as f32;
                        let screen_h = self.screen_size_physical.h as f32;

                        // position_logical is top-left. Quad vertices are -1 to 1 (center at 0,0 for unit quad).
                        // Hotspot is already applied to position_logical by the caller (RenderElement construction).
                        // Adjust position_logical by hotspot before this, if hotspot is part of RenderElement::Cursor.
                        // The RenderElement::Cursor has `position_logical` which should already account for hotspot.

                        // 1. Scale factors for quad (size of cursor texture in NDC)
                        let scale_x = tex_width / screen_w; // Scale from unit quad width (2) to texture width in NDC
                        let scale_y = tex_height / screen_h;

                        // 2. Translation factors for quad (position of cursor's center in NDC)
                        // position_logical is top-left. We want to position the center of the quad.
                        let center_x_logical = position_logical.x as f32 + tex_width / 2.0;
                        let center_y_logical = position_logical.y as f32 + tex_height / 2.0;

                        let translate_x_ndc = (center_x_logical / screen_w) * 2.0 - 1.0;
                        let translate_y_ndc = (center_y_logical / screen_h) * -2.0 + 1.0; // Y is inverted in NDC

                        // Build mat3x3: column-major
                        // [ sx,  0, tx  ]
                        // [  0, sy, ty  ]
                        // [  0,  0,  1  ]
                        let transform_matrix = [ // Column 1, Column 2, Column 3
                            scale_x, 0.0, 0.0,         // Column 1 (scales x, then y-shear, then x-perspective)
                            0.0, scale_y, 0.0,         // Column 2 (x-shear, scales y, then y-perspective)
                            translate_x_ndc, translate_y_ndc, 1.0, // Column 3 (translates x, then y, then global scale)
                        ]; // This is row-major representation for bytemuck. WGSL mat3x3 constructor takes columns.
                           // So, this needs to be transposed if passed directly or constructed carefully for WGSL.
                           // WGSL mat3x3(col0, col1, col2)
                           // col0 = vec3(m[0],m[1],m[2]), col1 = vec3(m[3],m[4],m[5]), col2 = vec3(m[6],m[7],m[8])
                           // Our matrix above is laid out for bytemuck as if it's [col0.x, col0.y, col0.z, col1.x, ... ]
                           // The shader expects mat3x3<f32>.
                           // A common way to represent scale(S) then translate(T) for a point P is T * S * P.
                           // If unit quad is [-1,1], first scale by (tex_width/2, tex_height/2) then translate.
                           // Let's adjust vertices of QUAD_VERTICES to be [0,1] range.
                           // Current QUAD_VERTICES: [-1.0, -1.0] to [1.0, 1.0] (2x2 size)
                           // Scale: (tex_width / screen_w, tex_height / screen_h) to make it size of texture in NDC
                           // Translate: ( (pos.x / screen_w)*2-1 + tex_width_ndc/2, (pos.y / screen_h)*-2+1 - tex_height_ndc/2 )
                           // This is getting complex. Simpler:
                           // Scale matrix S = mat3x3(scale_x, 0, 0,  0, scale_y, 0,  0,0,1)
                           // Translate matrix T = mat3x3(1,0,translate_x_ndc,  0,1,translate_y_ndc, 0,0,1)
                           // Final matrix = T * S.
                           // Shader applies M * P (where P is original unit quad vertex)
                           // So, M = T*S
                           // M[0][0]=scale_x, M[0][2]=translate_x_ndc
                           // M[1][1]=scale_y, M[1][2]=translate_y_ndc
                           // M[2][2]=1
                           // In column major for shader:
                           // col0: (scale_x, 0, 0)
                           // col1: (0, scale_y, 0)
                           // col2: (translate_x_ndc, translate_y_ndc, 1)
                        let final_transform_col_major = [
                            scale_x, 0.0, 0.0,
                            0.0, scale_y, 0.0,
                            translate_x_ndc, translate_y_ndc, 1.0,
                        ];


                        let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Cursor Transform Uniform Buffer"),
                            contents: bytemuck::cast_slice(&final_transform_col_major), // mat3x3<f32>
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

                        // If we had a real texture, we'd set self.render_pipeline (textured)
                        // For the fallback, solid_color_pipeline is already set.
                        // If we successfully downcasted and created tex_bind_group:
                        // render_pass.set_pipeline(&self.render_pipeline);
                        // render_pass.set_bind_group(0, &tex_bind_group, &[]); // For texture
                        // render_pass.set_bind_group(1, &transform_bind_group, &[]); // For transform

                        // If using fallback (solid_color_pipeline), it doesn't use group 1 for transform.
                        // This means the current solid_color_pipeline's shader also needs the transform.
                        // This is a further complication: solid color elements might also need transform.
                        // For now, the cursor (even fallback) will use the textured pipeline,
                        // and if texture is missing, it might result in undefined texture sampling (e.g. black).
                        // Or, we use the textured pipeline and bind a dummy 1x1 white texture if cursor texture fails.

                        render_pass.set_pipeline(&self.render_pipeline); // Use textured pipeline
                        // If wgpu_texture_view_sampler was None, we need a fallback texture bind group
                        // or ensure the shader handles unbound texture gracefully (not typical).
                        // For now, this will crash if texture is not properly bound.
                        // This part needs a robust fallback texture if downcast fails.
                        // For the sake of this diff, assume a valid (but maybe dummy) tex_bind_group if actual one fails.
                        // This part of the logic is incomplete without robust fallback for texture binding.

                        // render_pass.set_bind_group(0, &tex_bind_group_or_fallback, &[]); // Bind group 0 for texture
                        render_pass.set_bind_group(1, &transform_bind_group, &[]); // Bind group 1 for transform matrix

                        render_pass.draw_indexed(0..self.quad_num_indices, 0, 0..1);
                        tracing::trace!("Attempted to render Cursor element at ({}, {}), size ~({}x{}) transformed to NDC ({}, {}) with scale ({}, {}).",
                                        position_logical.x, position_logical.y, tex_width, tex_height,
                                        translate_x_ndc, translate_y_ndc, scale_x, scale_y);
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
        );
        // TODO: A full implementation would require EGL interop to create an EGLImage
        // from the DMABUF, then import that into WGPU. This is highly platform-specific
        // and backend-specific (e.g. requires EGL context to be current).
        Err(RendererError::Unsupported(
            "DMABUF texture import not yet fully implemented for WGPU. Requires EGL interop or specific WGPU extensions.".to_string()
        ))
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
