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
    texture_bind_group_layout: wgpu::BindGroupLayout,
    default_sampler: Arc<wgpu::Sampler>, // Sampler for textures
}

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
                features: wgpu::Features::empty(), // Add features as needed, e.g., for DMABUF
                limits: wgpu::Limits::default(),
            },
            None, // Optional trace path
        ).await.map_err(|e| anyhow::anyhow!("Failed to request WGPU device: {}", e))?;
        tracing::info!("WGPU Device and Queue obtained.");

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

        // Render Pipeline Layout
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render Pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Textured Quad Render Pipeline"),
            layout: Some(&render_pipeline_layout),
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

        let final_device = Arc::new(device);
        let final_queue = Arc::new(queue);

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

                            // TODO: Apply geometry transformations
                            render_pass.draw_indexed(0..self.quad_num_indices, 0, 0..1);
                            tracing::trace!("Rendered WaylandSurface (texture id: {:?}) using WGPU.", wgpu_tex.id());

                        } else {
                            tracing::trace!("WaylandSurface element has no texture_handle, skipping render.");
                        }
                    }
                    RenderElement::SolidColor { color, geometry } => {
                        tracing::trace!("Skipping SolidColor element (not implemented for WGPU). Color: {:?}, Geo: {:?}", color, geometry);
                    }
                    RenderElement::Cursor { texture_arc, position_logical, .. } => {
                        tracing::trace!("Skipping Cursor element (not implemented for WGPU). Tex id: {:?}, Pos: {:?}", texture_arc.id(), position_logical);
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
    ) -> Result<Arc<WgpuRenderableTexture>, RendererError> { // Changed return type
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
        )))) // Wrapped in Arc::new
    }

    fn create_texture_from_dmabuf(
        &mut self,
        _dmabuf_attributes: &smithay::backend::allocator::dmabuf::Dmabuf,
    ) -> Result<Arc<WgpuRenderableTexture>, RendererError> { // Changed return type
        tracing::warn!("create_texture_from_dmabuf called but not yet implemented for WGPU.");
        Err(RendererError::Unsupported("DMABUF texture creation not yet implemented for WGPU".to_string()))
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
