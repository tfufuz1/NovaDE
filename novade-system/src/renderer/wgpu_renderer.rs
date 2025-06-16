// novade-system/src/renderer/wgpu_renderer.rs

use crate::compositor::renderer_interface::abstraction::{
    FrameRenderer, RenderElement, RenderableTexture, RendererError,
    ClientBuffer, BufferContent,
    BufferFormat as AbstractionBufferFormat, DmabufDescriptor, DmabufPlaneFormat
};
use novade_compositor_core::surface::SurfaceId;
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;
use smithay::utils::{Physical, Rectangle, Size};
use std::borrow::Cow;
use crate::renderer::wgpu_texture::WgpuRenderableTexture;
use smithay::reexports::wayland_server::protocol::wl_shm::Format as WlShmFormat;
use smithay::wayland::shm::with_buffer_contents_data;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            ],
        }
    }
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] }, Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 1.0] }, Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 1.0] }, Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 0.0] }, Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 0.0] },
];
const QUAD_INDICES: &[u16] = &[0, 1, 2, 3, 4, 5];

pub struct NovaWgpuRenderer {
    id: Uuid,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    surface: Option<wgpu::Surface>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    screen_size_physical: Size<i32, Physical>,

    render_pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    quad_num_indices: u32,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    transform_bind_group_layout: wgpu::BindGroupLayout,
    default_sampler: Arc<wgpu::Sampler>,

    solid_color_pipeline: wgpu::RenderPipeline,
    solid_color_bind_group_layout: wgpu::BindGroupLayout,
    dummy_white_texture: Arc<WgpuRenderableTexture>,

    current_encoder: Option<wgpu::CommandEncoder>,
    current_surface_texture: Option<wgpu::SurfaceTexture>,

    scene_render_target: Option<wgpu::Texture>,
    scene_render_target_view: Option<wgpu::TextureView>,

    post_processing_textures: [Option<wgpu::Texture>; 2],
    post_processing_texture_views: [Option<wgpu::TextureView>; 2],
    current_pp_input_idx: usize,
    post_processing_active: bool,

    post_processing_texture_bgl: Option<Arc<wgpu::BindGroupLayout>>, // Arc for cloning

    gamma_correction_pipeline: Option<wgpu::RenderPipeline>,
    gamma_correction_uniform_bgl: Option<Arc<wgpu::BindGroupLayout>>,
    gamma_value_buffer: Option<wgpu::Buffer>,

    tone_mapping_pipeline: Option<wgpu::RenderPipeline>,
    tone_mapping_uniform_bgl: Option<Arc<wgpu::BindGroupLayout>>,
    tone_mapping_params_buffer: Option<wgpu::Buffer>,

    blit_to_swapchain_pipeline: Option<wgpu::RenderPipeline>,

    // YUV-to-RGB conversion for multi-planar DMABUFs
    yuv_to_rgb_pipeline: Option<wgpu::RenderPipeline>,
    yuv_to_rgb_bind_group_layout_textures: Option<wgpu::BindGroupLayout>,
    // TODO: Consider BGL for uniforms like color matrix if needed
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ToneMapUniformsPod {
    exposure: f32,
    _padding: [f32; 3],
}

const SOLID_COLOR_VS_MAIN_WGSL: &str = include_str!("shaders/solid_color.vert.wgsl");
const SOLID_COLOR_FS_MAIN_WGSL: &str = include_str!("shaders/solid_color.frag.wgsl");
const TEXTURED_QUAD_WGSL: &str = include_str!("shaders/textured_quad.wgsl");
const FULLSCREEN_QUAD_VERT_WGSL: &str = include_str!("../../assets/shaders/fullscreen_quad.vert");
const GAMMA_CORRECTION_FRAG_WGSL: &str = include_str!("../../assets/shaders/gamma_correction.frag");
const TONEMAP_FRAG_WGSL: &str = include_str!("../../assets/shaders/tonemap.frag");
const COPY_TEXTURE_FRAG_WGSL: &str = include_str!("../../assets/shaders/copy_texture.frag");

// ANCHOR [YuvToRgbFragmentShaderPlaceholder]
// TODO [DmabufYuvShader]: This is a placeholder for novade-system/assets/shaders/yuv_to_rgb.frag
const YUV_TO_RGB_FRAG_WGSL: &str = r#"
// @group(0) @binding(0) var t_y: texture_2d<f32>;
// @group(0) @binding(1) var t_u: texture_2d<f32>; // Or t_uv for NV12 (texture_2d<vec2<f32>>)
// @group(0) @binding(2) var t_v: texture_2d<f32>; // Not needed for NV12 if t_uv is used
// @group(0) @binding(3) var s_source: sampler;
// // Uniforms for color matrix (e.g., BT.601, BT.709) might be needed
// // struct Colorimetry {
// //     y_offset: f32,
// //     uv_offset: f32,
// //     matrix: mat3x3<f32>,
// // };
// // @group(1) @binding(0) var<uniform> colorimetry: Colorimetry;

// @fragment
// fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
//     let y_raw = textureSample(t_y, s_source, tex_coords).r;

//     // Adjust UV sampling coordinates based on chroma subsampling (e.g., 4:2:0)
//     // This depends on how planes are mapped and if using one or two chroma textures.
//     // For I420/YU12 (Y, U, V planes separately):
//     let uv_coords = tex_coords; // Assuming U/V planes are already scaled if subsampled, or adjust here.
                                // If U/V planes are half-size: let uv_coords = tex_coords * 0.5; (incorrect, shader receives normalized coords for its texture)
                                // Correct sampling depends on how texture views are configured for subsampled planes.
                                // If U/V planes are full size but only contain chroma data for half image size, that's different.
                                // Assuming U/V planes are sampled correctly for their data content.

//     let u_raw = textureSample(t_u, s_source, uv_coords).r;
//     let v_raw = textureSample(t_v, s_source, uv_coords).r;

//     // Basic YCbCr to RGB conversion (BT.601 standard, adjust for others like BT.709)
//     // Assumes Y is in [0,1] range, U/V in [0,1] range with 0.5 as center.
//     // If Y is [16/255, 235/255] and U/V are [16/255, 240/255], adjustments are needed.
//     let y = y_raw; // Or apply offset: y_raw - colorimetry.y_offset;
//     let u = u_raw - 0.5; // Or apply offset: u_raw - colorimetry.uv_offset;
//     let v = v_raw - 0.5; // Or apply offset: v_raw - colorimetry.uv_offset;

//     // Using BT.601 coefficients (approximate)
//     let r = y + 1.402 * v;
//     let g = y - 0.344136 * u - 0.714136 * v;
//     let b = y + 1.772 * u;

//     // For NV12 (Y plane, UV interleaved plane):
//     // let y = textureSample(t_y, s_source, tex_coords).r;
//     // let uv = textureSample(t_u, s_source, tex_coords).rg; // t_u is actually t_uv (Rg88 format)
//     // let u = uv.r - 0.5;
//     // let v = uv.g - 0.5;
//     // ... same RGB calculation ...

//     return vec4<f32>(clamp(r,0.0,1.0), clamp(g,0.0,1.0), clamp(b,0.0,1.0), 1.0);
// }
"#;


impl NovaWgpuRenderer {
    pub async fn new<WHT>(window_handle_target: &WHT, initial_size: Size<u32, Physical>) -> Result<Self>
    where WHT: HasRawWindowHandle + HasRawDisplayHandle {
        let id = Uuid::new_v4();
        // ANCHOR [WgpuInstanceDmabufResearch]
        // TODO [WgpuDmabufInstanceFlags]: For DMABUF import, especially when relying on underlying
        // Vulkan extensions, specific instance extensions like VK_KHR_external_memory_capabilities
        // might be required. WGPU abstracts this, but if issues arise, ensure the WGPU backend
        // (e.g., Vulkan) is initialized with necessary capabilities.
        // Current wgpu instance creation:
        // let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        //     backends: wgpu::Backends::PRIMARY, // Or specific like VULKAN
        //     dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        //     flags: wgpu::InstanceFlags::default(), // Consider VALIDATION or DEBUG for development
        //     gles_minor_version: wgpu::Gles3MinorVersion::default(),
        // });
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(window_handle_target) }?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { /* ... */ compatible_surface: Some(&surface), ..Default::default()}).await.ok_or_else(|| anyhow::anyhow!("No suitable adapter"))?;

        // ANCHOR [WgpuDeviceDmabufResearch]
        // TODO [WgpuDmabufFeatures]: For DMABUF import, the underlying graphics API (e.g., Vulkan)
        // needs specific extensions (e.g., VK_EXT_external_memory_dma_buf, VK_KHR_external_memory_fd).
        // WGPU abstracts these. If WGPU provides explicit features for DMABUF import
        // (e.g., a variant in `wgpu::Features` like `EXTERNAL_MEMORY_DMA_BUF`),
        // they would need to be requested here in `DeviceDescriptor::features`.
        // As of current understanding, direct import of generic DMABUF FDs as `wgpu::Texture`
        // might not be a stable, high-level WGPU feature and could require either:
        // 1. Using `wgpu-hal` directly (breaks WGPU abstraction).
        // 2. Platform-specific surface integration if the DMABUF is for a displayable surface.
        // 3. Future WGPU APIs (e.g., `import_external_texture_from_dmabuf_fd` or similar).
        // The `TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES` feature might be relevant for formats
        // commonly used with DMABUFs if they are not standard WGPU formats.
        // let device_descriptor = wgpu::DeviceDescriptor {
        //     label: Some("NovaDE WGPU Device"),
        //     features: wgpu::Features::empty() // Add required features here
        //         // | wgpu::Features::TEXTURE_COMPRESSION_BC
        //         // | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        //         // | wgpu::Features::EXTERNAL_MEMORY_DMA_BUF (if such a feature exists)
        //         ,
        //     limits: wgpu::Limits::default(),
        // };
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await?;
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format: surface_format,
            width: initial_size.w, height: initial_size.h,
            present_mode: wgpu::PresentMode::Fifo, alpha_mode: wgpu::CompositeAlphaMode::Auto, view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let main_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("Main Shader"), source: wgpu::ShaderSource::Wgsl(TEXTURED_QUAD_WGSL.into()) });
        let default_sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor::default()));

        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Main Texture BGL"), entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ]});
        let transform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Main Transform BGL"), entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }]});
        let main_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Main Pipeline Layout"), bind_group_layouts: &[&texture_bgl, &transform_bgl], push_constant_ranges: &[] });
        let main_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Pipeline"), layout: Some(&main_pipeline_layout),
            vertex: wgpu::VertexState { module: &main_shader_module, entry_point: "vs_main", buffers: &[Vertex::desc()] },
            // TODO [AdvancedBlendingModes]: The current blend state is REPLACE. For transparency and other effects,
            // this could be configured to `wgpu::BlendState::ALPHA_BLENDING` or custom blend factors/operations.
            // This would likely involve:
            // 1. Storing `BlendState` as part of `RenderElement` or deriving it from surface properties.
            // 2. Potentially creating multiple pipelines for different blend states or dynamically setting blend state if supported.
            // ANCHOR [AdvancedBlendingModesOutline]
            fragment: Some(wgpu::FragmentState { module: &main_shader_module, entry_point: "fs_main", targets: &[Some(wgpu::ColorTargetState { format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None,
        });

        // TODO [ShaderHotReloading]: Implement shader hot reloading. This would involve:
        // 1. A mechanism to watch shader files for changes (e.g., `notify` crate).
        // 2. When a change is detected, re-compile the shader module (`device.create_shader_module`).
        // 3. Re-create any pipelines that use that shader. This can be complex if pipeline layouts also change.
        // 4. A strategy for error handling if the new shader fails to compile (e.g., keep using the old one).
        // 5. This might be managed by a dedicated "ShaderManager" struct.
        // ANCHOR [ShaderHotReloadingOutline]

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Quad VB"), contents: bytemuck::cast_slice(QUAD_VERTICES), usage: wgpu::BufferUsages::VERTEX });
        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Quad IB"), contents: bytemuck::cast_slice(QUAD_INDICES), usage: wgpu::BufferUsages::INDEX });

        let solid_color_vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("SolidVS"), source: wgpu::ShaderSource::Wgsl(SOLID_COLOR_VS_MAIN_WGSL.into())});
        let solid_color_fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("SolidFS"), source: wgpu::ShaderSource::Wgsl(SOLID_COLOR_FS_MAIN_WGSL.into())});
        let solid_color_bgl_uni = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{ label: Some("Solid Color BGL Uniform"), entries: &[wgpu::BindGroupLayoutEntry{binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer{ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None}, count: None}]});
        let solid_color_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label: Some("Solid Pipeline Layout"), bind_group_layouts: &[&solid_color_bgl_uni, &transform_bgl], push_constant_ranges: &[]});
        let solid_color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("Solid Pipeline"), layout: Some(&solid_color_pipeline_layout),
            vertex: wgpu::VertexState{module: &solid_color_vs_module, entry_point: "vs_main", buffers: &[Vertex::desc()]},
            fragment: Some(wgpu::FragmentState{module: &solid_color_fs_module, entry_point: "fs_main", targets: &[Some(wgpu::ColorTargetState{format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL})]}),
            primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None,
        });
        let dummy_texture_data = [255u8; 4];
        let dummy_wgpu_texture = device.create_texture(&wgpu::TextureDescriptor { label: Some("Dummy Texture"), size: wgpu::Extent3d{width:1,height:1,depth_or_array_layers:1}, mip_level_count:1, sample_count:1, dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8UnormSrgb, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[]});
        queue.write_texture(wgpu::ImageCopyTexture{texture: &dummy_wgpu_texture, mip_level:0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All}, &dummy_texture_data, wgpu::ImageDataLayout{offset:0, bytes_per_row:Some(4), rows_per_image:Some(1)}, wgpu::Extent3d{width:1,height:1,depth_or_array_layers:1});
        let dummy_white_texture = Arc::new(WgpuRenderableTexture::new(device.clone(), dummy_wgpu_texture, device.create_texture_view(&dummy_wgpu_texture, &wgpu::TextureViewDescriptor::default()), default_sampler.as_ref().clone(), 1,1,wgpu::TextureFormat::Rgba8UnormSrgb, None));

        // Post-Processing Common
        let fs_quad_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("Fullscreen Quad VS"), source: wgpu::ShaderSource::Wgsl(FULLSCREEN_QUAD_VERT_WGSL.into()) });
        let pp_texture_bgl = Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PP Texture BGL"), entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ]}));

        // Gamma Correction
        let gamma_frag_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("Gamma FS"), source: wgpu::ShaderSource::Wgsl(GAMMA_CORRECTION_FRAG_WGSL.into()) });
        let gamma_uniform_bgl = Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gamma Uniform BGL"), entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<f32>() as u64) }, count: None }],
        }));
        let gamma_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Gamma Pipeline Layout"), bind_group_layouts: &[&pp_texture_bgl, &gamma_uniform_bgl], push_constant_ranges: &[] });
        let gamma_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { label: Some("Gamma Pipeline"), layout: Some(&gamma_pipeline_layout), vertex: wgpu::VertexState { module: &fs_quad_shader_module, entry_point: "vs_main", buffers: &[] }, fragment: Some(wgpu::FragmentState { module: &gamma_frag_module, entry_point: "fs_main", targets: &[Some(wgpu::ColorTargetState { format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })] }), primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None });
        let gamma_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Gamma Uniform Buffer"), contents: bytemuck::cast_slice(&[2.2f32]), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST });
        // TODO [DynamicUniformBufferManagement]: For `gamma_buffer` and `tonemap_buffer` below, they are created once and updated.
        // For more dynamic UBOs (e.g. per-object data not fitting push constants, or needing updates every frame),
        // strategies include:
        // 1. Reusing a large buffer with dynamic offsets (`write_buffer` + `set_bind_group` with offset). Requires `has_dynamic_offset: true` in BGL.
        // 2. Multiple small buffers, potentially managed by a pool allocator.
        // 3. For frequently updated UBOs like transforms (if not using push constants or vertex attributes), `queue.write_buffer` is standard.
        //    The current transform buffer per-element (see `render_frame`) is one such example of dynamic UBO content.
        // Consider alignment requirements (`min_uniform_buffer_offset_alignment`).
        // ANCHOR [DynamicUniformBufferOutline]

        // Tone Mapping
        let tonemap_frag_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ToneMap FS"), source: wgpu::ShaderSource::Wgsl(TONEMAP_FRAG_WGSL.into()) });
        let tonemap_uniform_bgl = Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ToneMap Uniform BGL"), entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<ToneMapUniformsPod>() as u64) }, count: None }],
        }));
        let tonemap_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("ToneMap Pipeline Layout"), bind_group_layouts: &[&pp_texture_bgl, &tonemap_uniform_bgl], push_constant_ranges: &[] });
        let tonemap_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { label: Some("ToneMap Pipeline"), layout: Some(&tonemap_pipeline_layout), vertex: wgpu::VertexState { module: &fs_quad_shader_module, entry_point: "vs_main", buffers: &[] }, fragment: Some(wgpu::FragmentState { module: &tonemap_frag_module, entry_point: "fs_main", targets: &[Some(wgpu::ColorTargetState { format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })] }), primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None });
        let tonemap_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("ToneMap Uniform Buffer"), contents: bytemuck::cast_slice(&[ToneMapUniformsPod{exposure: 1.0, _padding: [0.0;3]}]), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST });

        // Blit Pipeline
        let copy_frag_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("Copy FS"), source: wgpu::ShaderSource::Wgsl(COPY_TEXTURE_FRAG_WGSL.into()) });
        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Blit Pipeline Layout"), bind_group_layouts: &[&pp_texture_bgl], push_constant_ranges: &[] });
        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"), layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState { module: &fs_quad_shader_module, entry_point: "vs_main", buffers: &[]},
            fragment: Some(wgpu::FragmentState { module: &copy_frag_module, entry_point: "fs_main", targets: &[Some(wgpu::ColorTargetState { format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None,
        });

        // Scene and Ping-Pong Textures
        let scene_render_target_desc = wgpu::TextureDescriptor {
            label: Some("Scene Render Target"), size: wgpu::Extent3d { width: initial_size.w, height: initial_size.h, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
        };
        let scene_rt = device.create_texture(&scene_render_target_desc);
        let scene_rt_view = scene_rt.create_view(&wgpu::TextureViewDescriptor::default());
        let pp_texture_desc_fn = |i:usize| wgpu::TextureDescriptor {
            label: Some(&format!("PP Texture {}", i)), size: wgpu::Extent3d { width: initial_size.w, height: initial_size.h, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
        };
        let pp_tex_0 = device.create_texture(&pp_texture_desc_fn(0));
        let pp_view_0 = pp_tex_0.create_view(&wgpu::TextureViewDescriptor::default());
        let pp_tex_1 = device.create_texture(&pp_texture_desc_fn(1));
        let pp_view_1 = pp_tex_1.create_view(&wgpu::TextureViewDescriptor::default());

        // TODO [DmabufYuvShader]: Load and compile the YUV_TO_RGB_FRAG_WGSL shader
        // And create the yuv_to_rgb_pipeline and its bind group layouts.
        // For now, initializing to None.
        let yuv_to_rgb_bgl_textures = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("YUV to RGB Textures BGL"),
            entries: &[
                // Y plane
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                // U plane (or UV plane)
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                // V plane (if separate)
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                // Sampler
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });

        Ok(Self {
            id, instance, adapter, device, queue, // Use the Arc'd device and queue
            surface: Some(surface), surface_config: Some(surface_config),
            screen_size_physical: Size::from((initial_size.w as i32, initial_size.h as i32)),
            quad_index_buffer, quad_num_indices: QUAD_INDICES.len() as u32, // ensure this is correctly initialized
            texture_bind_group_layout: texture_bgl, default_sampler, transform_bind_group_layout: transform_bgl,
            solid_color_pipeline, solid_color_bind_group_layout: solid_color_bgl, dummy_white_texture,
            current_encoder: None, current_surface_texture: None,
            scene_render_target: Some(scene_rt), scene_render_target_view: Some(scene_rt_view),
            post_processing_textures: [Some(pp_tex_0), Some(pp_tex_1)],
            post_processing_texture_views: [Some(pp_view_0), Some(pp_view_1)],
            current_pp_input_idx: 0, post_processing_active: false,
            post_processing_texture_bgl: Some(pp_texture_bgl.clone()), // Store common BGL
            gamma_correction_pipeline: Some(gamma_pipeline),
            gamma_correction_uniform_bgl: Some(gamma_uniform_bgl),
            gamma_value_buffer: Some(gamma_buffer),
            tone_mapping_pipeline: Some(tonemap_pipeline),
            tone_mapping_uniform_bgl: Some(tonemap_uniform_bgl),
            tone_mapping_params_buffer: Some(tonemap_buffer),
            blit_to_swapchain_pipeline: Some(blit_pipeline),
            yuv_to_rgb_pipeline: None, // Initialize as None, to be created with shader
            yuv_to_rgb_bind_group_layout_textures: Some(yuv_to_rgb_bgl_textures),
            // TODO [ErrorHandlingAndRecovery]: WGPU operations can fail (e.g., device lost).
            // Robust applications should handle `Device::poll(Maintain::Poll)` and listen for `DeviceLost` events.
            // This might involve recreating the device, surface, and all GPU resources.
            // Errors from `request_device`, `queue.submit`, `surface.get_current_texture` should be handled gracefully.
            // ANCHOR [ErrorHandlingAndRecoveryOutline]
        })
    }
    // TODO [ShaderHotReloading]: (see ANCHOR [ShaderHotReloadingOutline] in new())
    // TODO [DynamicUniformBufferManagement]: (see ANCHOR [DynamicUniformBufferOutline] in new() and render_frame())

    pub fn resize(&mut self, new_size: Size<u32, Physical>) {
        if new_size.w > 0 && new_size.h > 0 {
            self.screen_size_physical = Size::from((new_size.w as i32, new_size.h as i32));
            if let (Some(surface), Some(config)) = (self.surface.as_ref(), self.surface_config.as_mut()) {
                config.width = new_size.w;
                config.height = new_size.h;
                surface.configure(&self.device, config);
                tracing::info!("WGPU surface resized to: {}x{}", new_size.w, new_size.h);

                let scene_target_desc = wgpu::TextureDescriptor {
                    label: Some("Scene Render Target Texture"),
                    size: wgpu::Extent3d { width: new_size.w, height: new_size.h, depth_or_array_layers: 1 },
                    mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                    format: config.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                };
                self.scene_render_target = Some(self.device.create_texture(&scene_target_desc));
                self.scene_render_target_view = self.scene_render_target.as_ref().map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

                for i in 0..2 {
                    let pp_desc = wgpu::TextureDescriptor {
                        label: Some(&format!("Post-Processing Texture {}", i)),
                        size: wgpu::Extent3d { width: new_size.w, height: new_size.h, depth_or_array_layers: 1 },
                        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                        format: config.format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    };
                    self.post_processing_textures[i] = Some(self.device.create_texture(&pp_desc));
                    self.post_processing_texture_views[i] = self.post_processing_textures[i].as_ref().map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
                }
                tracing::info!("Resized offscreen targets to: {}x{}", new_size.w, new_size.h);
            }
        } else {
            tracing::warn!("WGPU surface resize requested with zero dimension: {}x{}", new_size.w, new_size.h);
        }
    }
}

impl FrameRenderer for NovaWgpuRenderer {
    fn id(&self) -> Uuid { self.id }

    fn render_frame<'iter_elements>(
        &mut self,
        elements: impl IntoIterator<Item = RenderElement<'iter_elements>>,
        _output_geometry_physical: Rectangle<i32, Physical>,
        _output_scale: f64,
    ) -> Result<(), RendererError> {
        if self.current_encoder.is_some() || self.current_surface_texture.is_some() {
            tracing::warn!("render_frame called while previous frame resources were not cleared.");
            self.current_encoder = None; self.current_surface_texture = None;
            // Consider this a recoverable error or internal state issue.
        }

        // TODO [ErrorHandlingAndRecovery]: Surface acquisition can fail. `Outdated` is handled, but other errors
        // like `Lost` or `Timeout` might require recreating the surface or even device.
        // ANCHOR [ErrorHandlingAndRecoveryOutlinePoint2]
        let surface = self.surface.as_ref().ok_or_else(|| RendererError::Generic("WGPU surface not available".to_string()))?;
        let acquired_surface_texture = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                // This is a recoverable error. We reconfigure the surface and try again.
                let config = self.surface_config.as_ref().unwrap();
                surface.configure(&self.device, config);
                surface.get_current_texture().map_err(|e| RendererError::BufferSwapFailed(format!("Get WGPU texture after reconfigure: {}", e)))?
            }
            Err(e) => return Err(RendererError::BufferSwapFailed(format!("Get WGPU texture: {}", e))),
        };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Main Encoder") });

        if let Some(target_view) = &self.scene_render_target_view {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Scene Pass (to scene_render_target)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store }, resolve_target: None,
                })],
                // TODO [MultiTargetRendering]: For effects requiring multiple outputs (e.g., deferred shading G-Buffer),
                // `color_attachments` would be an array of `Option<RenderPassColorAttachment>` targeting different texture views.
                // The fragment shader would then output multiple color values (`@location(0) out_color0: vec4<f32>, @location(1) out_color1: vec4<f32>`).
                // This would require separate `TextureView`s and likely different `TextureFormat`s for each target.
                // ANCHOR [MultiTargetRenderingOutline]
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            // TODO [AdvancedBlendingModes]: If different blend states are needed per element, this might involve
            // switching pipelines here or using a more advanced system if WGPU supports dynamic blend states without pipeline switches.
            // (see ANCHOR [AdvancedBlendingModesOutline] in new())
            rpass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            rpass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // TODO [InstancedRenderingImpl]: For scenarios with many identical elements (e.g., icons, particles,
            // or even identical window decorations if rendered as separate quads), instanced rendering
            // could significantly reduce draw calls. This would involve:
            // 1. Identifying batches of `RenderElement::TextureNode` that share the same texture, sampler,
            //    and basic geometry but differ in transform or other per-instance attributes (color tint, clip_rect etc.).
            // 2. Creating a new `RenderElement::TextureInstanceBatch { texture, instances: Vec<InstanceData> }` variant or similar.
            //    `InstanceData` would hold transform, color tint, source_rect, clip_rect_id (if clipping is complex).
            // 3. Modifying the vertex shader to accept per-instance attributes from an instance buffer (using `@builtin(instance_index)`
            //    and `wgpu::VertexStepMode::Instance`).
            // 4. Creating a new `wgpu::RenderPipeline` configured for instanced drawing (vertex_buffers would describe instance data layout).
            // 5. In `render_frame`, collecting instance data into a `wgpu::Buffer` (updated each frame or if data changes)
            //    and using `render_pass.set_vertex_buffer(slot, instance_buffer_slice)`
            //    before calling `render_pass.draw_indexed(indices, base_vertex, 0..instance_count)`.
            // ANCHOR [InstancedRenderingOutline]
            for element in elements {
                 match element {
                    RenderElement::TextureNode(params) => {
                        if let Some(wgpu_tex) = params.texture.as_any().downcast_ref::<WgpuRenderableTexture>() {
                            if wgpu_tex.is_multi_planar {
                                // TODO [DmabufYuvRendering]: Implement YUV rendering path
                                // 1. Ensure yuv_to_rgb_pipeline is compiled.
                                // 2. Get Y, U, V (or Y, UV) plane views from wgpu_tex.plane_views or specific accessors.
                                // 3. Create a bind group using yuv_to_rgb_bind_group_layout_textures with these plane views and a sampler.
                                // 4. Set the yuv_to_rgb_pipeline.
                                // 5. Set the YUV bind group.
                                // 6. Set transform bind group (as below).
                                // 7. Draw indexed.
                                tracing::warn!("Multi-planar DMABUF rendering for texture ID {} not yet implemented, skipping.", wgpu_tex.id());
                                // Fallthrough or skip rendering this element for now.
                                // For now, let's try to render the primary plane with the standard pipeline as a fallback.
                                // This will likely look wrong for YUV.
                                rpass.set_pipeline(&self.render_pipeline);
                                let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Element Texture BG (Primary Plane of Multi-Planar)"),
                                    layout: &self.texture_bind_group_layout,
                                    entries: &[
                                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(wgpu_tex.view()) }, // .view() gets primary_view
                                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(wgpu_tex.sampler()) },
                                    ],
                                });
                                rpass.set_bind_group(0, &texture_bind_group, &[]);
                            } else {
                                rpass.set_pipeline(&self.render_pipeline);
                                let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: Some("Element Texture BG (Single-Planar)"), layout: &self.texture_bind_group_layout,
                                    entries: &[
                                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(wgpu_tex.view()) },
                                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(wgpu_tex.sampler()) },
                                    ],
                                });
                                rpass.set_bind_group(0, &texture_bind_group, &[]);
                            }

                            let sg_matrix = params.transform.matrix;
                            let transform_uniform_data: [f32; 9] = [ sg_matrix[0][0], sg_matrix[1][0], 0.0, sg_matrix[0][1], sg_matrix[1][1], 0.0, sg_matrix[0][2], sg_matrix[1][2], 1.0 ];
                            // TODO [DynamicUniformBufferManagement]: This transform_buffer is created per-element, per-frame.
                            // For high element counts, this is inefficient (many small buffers, many bind group creations).
                            // Better approaches:
                            // 1. Instanced rendering (see ANCHOR [InstancedRenderingOutline]): Instance data (including transforms)
                            //    is uploaded once per frame to a larger buffer.
                            // 2. Uniform buffer with dynamic offsets: Upload all transforms to one large buffer, use `set_bind_group`
                            //    with dynamic offsets for each draw call. Requires `min_uniform_buffer_offset_alignment`.
                            // 3. Storage buffers: If transforms are numerous and complex, store them in a storage buffer accessible
                            //    by the vertex shader, indexed by `vertex_index` or `instance_index`.
                            // ANCHOR [DynamicUniformBufferOutlineInRenderFrame]
                            let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Element Transform Uniform Buffer"), contents: bytemuck::cast_slice(&transform_uniform_data), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            });
                            let transform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Element Transform BG"), layout: &self.transform_bind_group_layout,
                                entries: &[wgpu::BindGroupEntry { binding: 0, resource: transform_buffer.as_entire_binding() }],
                            });
                            rpass.set_bind_group(0, &texture_bind_group, &[]);
                            rpass.set_bind_group(1, &transform_bind_group, &[]);
                            let clip = params.clip_rect; // Assuming physical pixels
                            rpass.set_scissor_rect(clip.origin.x as u32, clip.origin.y as u32, clip.size.width as u32, clip.size.height as u32);
                            rpass.draw_indexed(0..self.quad_num_indices, 0, 0..1);
                        }
                    }
                    _ => {}
                }
            }
        } else {
            return Err(RendererError::Generic("Scene render target view not available".to_string()));
        }

        self.post_processing_active = false;
        self.current_pp_input_idx = 0;
        self.current_encoder = Some(encoder);
        self.current_surface_texture = Some(acquired_surface_texture);
        Ok(())
    }

    fn submit_and_present_frame(&mut self) -> Result<(), RendererError> {
        // TODO [FramePacingImpl]: Implement advanced frame pacing for smoother animations and power saving.
        // Current presentation is typically tied to VSync via FIFO or Mailbox present modes.
        // For more precise control (e.g., targeting specific frame times):
        // 1. Research platform-specific WGPU extensions or OS-level APIs for frame pacing.
        // 2. Consider using `wgpu::SurfaceConfiguration::present_mode = wgpu::PresentMode::Immediate`
        //    very carefully, combined with manual timing (e.g., `std::thread::sleep` or more precise timers)
        //    to control the presentation rate. This can lead to tearing if not handled correctly.
        // 3. Investigate integration with Wayland's presentation-time protocol (`wp_presentation_feedback`)
        //    to get feedback on when frames are actually shown, which can inform the next frame's timing.
        //    Smithay's `Output::presentation_feedback()` might be a starting point if using Smithay for Wayland parts.
        // ANCHOR [FramePacingOutline]

        // TODO [AdaptiveSyncImpl]: Support for Adaptive Sync (FreeSync/G-Sync/VRR).
        // This usually relies on the driver and display supporting Variable Refresh Rate (VRR).
        // 1. WGPU itself might not have direct API control beyond selecting appropriate present modes
        //    (e.g., `wgpu::PresentMode::Mailbox` or `wgpu::PresentMode::Fifo` are generally compatible).
        //    `wgpu::PresentMode::AutoNoVsync` or `wgpu::PresentMode::AutoVsync` might also be relevant if supported by backend.
        // 2. The primary mechanism is for the application (compositor) to present frames as fast as it can
        //    (or up to the desired rate within the VRR range of the display), and the display adjusts its refresh rate.
        // 3. Ensuring low input latency and consistent frame delivery from the compositor is key.
        // 4. May require querying display capabilities (e.g., via Wayland protocols like `wp_drm_lease` or OS APIs)
        //    to determine VRR range and enable it if necessary.
        // ANCHOR [AdaptiveSyncOutline]

        // TODO [PerformanceMonitoringIntegration]: Before submitting, GPU queries (occlusion, timestamp) would be resolved.
        // `wgpu-profiler` or manual `wgpu::QuerySet` can be used.
        // Timestamp queries can measure pass execution times:
        // `encoder.write_timestamp(query_set, start_query_index)`
        // `encoder.write_timestamp(query_set, end_query_index)`
        // Results are read back from the query set buffer on the CPU after submission, usually a few frames later.
        // ANCHOR [PerformanceMonitoringOutline]
        let mut encoder = self.current_encoder.take().ok_or_else(|| RendererError::Generic("Encoder missing in submit".to_string()))?;
        let swapchain_texture = self.current_surface_texture.take().ok_or_else(|| RendererError::Generic("Swapchain texture missing in submit".to_string()))?;
        let swapchain_view = swapchain_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let final_source_view = if self.post_processing_active {
            self.post_processing_texture_views[self.current_pp_input_idx].as_ref().unwrap()
        } else {
            self.scene_render_target_view.as_ref().unwrap()
        };

        let blit_pipeline = self.blit_to_swapchain_pipeline.as_ref().unwrap();
        let blit_bgl = self.post_processing_texture_bgl.as_ref().unwrap(); // Reusing common PP BGL

        let blit_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit to Swapchain BG"), layout: blit_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(final_source_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(self.default_sampler.as_ref()) },
            ],
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Blit to Swapchain Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &swapchain_view, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store }, resolve_target: None,
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        rpass.set_pipeline(blit_pipeline);
        rpass.set_bind_group(0, &blit_bind_group, &[]);
        rpass.draw(0..6, 0..1);
        drop(rpass);

        self.queue.submit(std::iter::once(encoder.finish()));
        swapchain_texture.present();
        self.post_processing_active = false;
        Ok(())
    }

    fn apply_gamma_correction(&mut self, gamma_value: f32) -> Result<(), RendererError> {
        let encoder = self.current_encoder.as_mut().ok_or_else(|| RendererError::Generic("Encoder missing for Gamma".to_string()))?;
        let input_view = if !self.post_processing_active { self.scene_render_target_view.as_ref() }
                         else { self.post_processing_texture_views[self.current_pp_input_idx].as_ref() }
                         .ok_or_else(|| RendererError::Generic("Gamma input view missing".to_string()))?;
        let output_idx = if self.post_processing_active { 1 - self.current_pp_input_idx } else { 0 };
        let output_view = self.post_processing_texture_views[output_idx].as_ref().unwrap();

        let pipeline = self.gamma_correction_pipeline.as_ref().unwrap();
        let texture_bgl = self.post_processing_texture_bgl.as_ref().unwrap();
        let uniform_bgl = self.gamma_correction_uniform_bgl.as_ref().unwrap();
        let gamma_buffer = self.gamma_value_buffer.as_ref().unwrap();

        self.queue.write_buffer(gamma_buffer, 0, bytemuck::cast_slice(&[gamma_value]));
        let tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Gamma Source BG"), layout: texture_bgl, entries: &[ wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(input_view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(self.default_sampler.as_ref()) } ]});
        let uni_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Gamma Uniform BG"), layout: uniform_bgl, entries: &[wgpu::BindGroupEntry { binding: 0, resource: gamma_buffer.as_entire_binding() }]});

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("Gamma Pass"), color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: output_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store }})], ..Default::default()});
        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &tex_bg, &[]);
        rpass.set_bind_group(1, &uni_bg, &[]);
        rpass.draw(0..6, 0..1);
        drop(rpass);
        self.current_pp_input_idx = output_idx;
        self.post_processing_active = true;
        Ok(())
    }

    fn apply_hdr_to_sdr_tone_mapping(&mut self, _max_luminance: f32, exposure: f32) -> Result<(), RendererError> {
        let encoder = self.current_encoder.as_mut().ok_or_else(|| RendererError::Generic("Encoder missing for ToneMap".to_string()))?;
        let input_view = if !self.post_processing_active { self.scene_render_target_view.as_ref() }
                         else { self.post_processing_texture_views[self.current_pp_input_idx].as_ref() }
                         .ok_or_else(|| RendererError::Generic("ToneMap input view missing".to_string()))?;
        let output_idx = if self.post_processing_active { 1 - self.current_pp_input_idx } else { 0 };
        let output_view = self.post_processing_texture_views[output_idx].as_ref().unwrap();

        let pipeline = self.tone_mapping_pipeline.as_ref().unwrap();
        let texture_bgl = self.post_processing_texture_bgl.as_ref().unwrap();
        let uniform_bgl = self.tone_mapping_uniform_bgl.as_ref().unwrap();
        let params_buffer = self.tone_mapping_params_buffer.as_ref().unwrap();

        self.queue.write_buffer(params_buffer, 0, bytemuck::cast_slice(&[ToneMapUniformsPod{exposure, _padding: [0.0;3]}]));
        let tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("ToneMap Source BG"), layout: texture_bgl, entries: &[ wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(input_view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(self.default_sampler.as_ref()) } ]});
        let uni_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("ToneMap Uniform BG"), layout: uniform_bgl, entries: &[wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() }]});

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("ToneMap Pass"), color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: output_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store }})], ..Default::default()});
        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &tex_bg, &[]);
        rpass.set_bind_group(1, &uni_bg, &[]);
        rpass.draw(0..6, 0..1);
        drop(rpass);
        self.current_pp_input_idx = output_idx;
        self.post_processing_active = true;
        Ok(())
    }

    // TODO [CustomPostProcessingEffects]: To add a new post-processing effect (e.g., "InvertColors"):
    // 1. Create a new fragment shader (e.g., `assets/shaders/invert_colors.frag`).
    //    It would sample an input texture and output inverted colors.
    // 2. In `NovaWgpuRenderer::new()`:
    //    a. Load the shader: `device.create_shader_module(...)`.
    //    b. Create a `wgpu::RenderPipeline` similar to `gamma_correction_pipeline` or `tone_mapping_pipeline`,
    //       using the fullscreen quad vertex shader and the new fragment shader.
    //    c. If the effect needs uniforms (e.g., an intensity factor), create a BGL and buffer for them.
    // 3. Add a new method to `NovaWgpuRenderer` and `FrameRenderer` trait:
    //    `fn apply_invert_colors(&mut self, intensity: f32) -> Result<(), RendererError>;`
    // 4. Implement this method in `NovaWgpuRenderer` following the pattern of `apply_gamma_correction`:
    //    a. Get the current encoder.
    //    b. Determine input_view and output_view from ping-pong textures.
    //    c. Update uniform buffer if any.
    //    d. Create bind groups for input texture and uniforms.
    //    e. Begin render pass, set pipeline, set bind groups, draw fullscreen quad.
    //    f. Update `current_pp_input_idx` and `post_processing_active`.
    // 5. The `CompositionEngine` can then call `renderer.apply_invert_colors(...)` in its chain.
    // ANCHOR [CustomPostProcessingEffectsOutline]

    // TODO [ComputeShaderIntegration]: WGPU supports compute shaders for general-purpose GPU calculations.
    // This could be used for:
    // - Advanced image processing effects not easily done with fragment shaders (e.g., complex blurs, simulations).
    // - Physics calculations.
    // - Data transformations.
    // Integration would involve:
    // 1. Writing a compute shader (`.wgsl` file with `@compute @workgroup_size(...) fn main(...)`).
    // 2. Creating a `wgpu::ComputePipeline`.
    // 3. Setting up `wgpu::BindGroup`s for input/output buffers/textures (storage buffers/textures).
    // 4. In a new method (e.g., `fn dispatch_compute_task(&mut self, ...)`):
    //    a. Get the current command encoder or create a new one.
    //    b. Begin a `wgpu::ComputePass`.
    //    c. Set the pipeline and bind groups.
    //    d. Call `pass.dispatch_workgroups(x, y, z)`.
    // e. Ensure proper synchronization with rendering passes if compute outputs are used in rendering (barriers).
    // ANCHOR [ComputeShaderIntegrationOutline]

    fn upload_surface_texture( &mut self, _surface_id: SurfaceId, client_buffer: &ClientBuffer<'_>, ) -> Result<Box<dyn RenderableTexture>, RendererError> {
        match &client_buffer.content {
            BufferContent::Shm { id, data, width, height, stride, format } => {
                let wgpu_texture_format = match format {
                    AbstractionBufferFormat::Argb8888 => wgpu::TextureFormat::Bgra8UnormSrgb,
                    AbstractionBufferFormat::Xrgb8888 => wgpu::TextureFormat::Bgra8UnormSrgb,
                };
                let texture_size = wgpu::Extent3d { width: *width, height: *height, depth_or_array_layers: 1 };
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("SHM Surface Texture (buffer_id: {})", id)), size: texture_size,
                    mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                    format: wgpu_texture_format, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
                });
                self.queue.write_texture( wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    data, wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(*stride), rows_per_image: Some(*height) }, texture_size);

                let view_arc = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
                let texture_arc = Arc::new(texture);
                let sampler_arc = self.default_sampler.clone();

                Ok(Box::new(WgpuRenderableTexture::new(
                    self.device.clone(),
                    Arc::try_unwrap(texture_arc).expect("Arc unwrap failed for SHM texture"), // WgpuRenderableTexture expects owned Texture
                    Arc::try_unwrap(view_arc).expect("Arc unwrap failed for SHM view"),       // WgpuRenderableTexture expects owned TextureView
                    Arc::try_unwrap(sampler_arc).expect("Arc unwrap failed for SHM sampler"), // WgpuRenderableTexture expects owned Sampler
                    *width, *height,
                    wgpu_texture_format,
                    None, // fourcc_format
                    false, // is_multi_planar
                    None, // initial_plane_textures
                    None, // initial_plane_views
                )))
            }
            BufferContent::Dmabuf { id, descriptors, width: buffer_overall_width, height: buffer_overall_height } => {
                tracing::info!( "Attempting DMABUF texture upload for surface_id: {}, buffer_id: {}, overall_size: {}x{}", _surface_id.id(), id, buffer_overall_width, buffer_overall_height);

                // ANCHOR [DmabufImportMultiPlane]
                // This section attempts to import multiple DMABUF planes.
                // WARNING: Direct DMABUF import into a wgpu::Texture is highly advanced.
                // This implementation outlines conceptual steps and uses placeholders.

                let mut plane_wgpu_textures: Vec<Arc<wgpu::Texture>> = Vec::new();
                let mut plane_wgpu_views: Vec<Arc<wgpu::TextureView>> = Vec::new();
                let mut primary_plane_format_wgpu: Option<wgpu::TextureFormat> = None;
                let mut num_valid_planes = 0;

                for (idx, desc_opt) in descriptors.iter().enumerate() {
                    if let Some(desc) = desc_opt {
                        num_valid_planes += 1;
                        tracing::info!("Processing DMABUF plane {}: desc: {:?}", idx, desc);

                        let plane_texture_format_wgpu = match desc.format {
                            DmabufPlaneFormat::R8 => wgpu::TextureFormat::R8Unorm,
                            DmabufPlaneFormat::Rg88 => wgpu::TextureFormat::Rg8Unorm,
                            DmabufPlaneFormat::Argb8888 => wgpu::TextureFormat::Bgra8Unorm, // Assuming display/overlay usage
                            DmabufPlaneFormat::Xrgb8888 => wgpu::TextureFormat::Bgra8Unorm,
                        };
                        if idx == 0 { primary_plane_format_wgpu = Some(plane_texture_format_wgpu); }

                        let plane_texture_descriptor = wgpu::TextureDescriptor {
                            label: Some(&format!("dmabuf_surface_{}_b{}_p{}", _surface_id.id(), id, desc.plane_index)),
                            size: wgpu::Extent3d { width: desc.width, height: desc.height, depth_or_array_layers: 1 },
                            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                            format: plane_texture_format_wgpu,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                            view_formats: &[],
                        };

                        // Conceptual import - for now, create placeholder
                        let plane_texture = self.device.create_texture(&plane_texture_descriptor);
                        let magenta_pixel_r8: [u8; 1] = [128]; // Mid-gray for R8 planes
                        let magenta_pixel_rg88: [u8; 2] = [128, 128]; // Mid-gray for RG88 planes
                        let magenta_pixel_bgra8: [u8; 4] = [255, 0, 255, 255]; // BGRA Magenta

                        let (pixel_data, bytes_per_pixel_plane) = match plane_texture_format_wgpu {
                            wgpu::TextureFormat::R8Unorm => (vec![magenta_pixel_r8[0]; (desc.width * desc.height) as usize], 1),
                            wgpu::TextureFormat::Rg8Unorm => (vec![magenta_pixel_rg88[0]; (desc.width * desc.height * 2) as usize].chunks_mut(2).flat_map(|chunk| { chunk.copy_from_slice(&magenta_pixel_rg88); chunk }).collect(), 2),
                            wgpu::TextureFormat::Bgra8Unorm => (vec![magenta_pixel_bgra8[0]; (desc.width * desc.height * 4) as usize].chunks_mut(4).flat_map(|chunk| { chunk.copy_from_slice(&magenta_pixel_bgra8); chunk }).collect(), 4),
                            _ => return Err(RendererError::Generic(format!("Unsupported wgpu format for DMABUF plane placeholder: {:?}", plane_texture_format_wgpu))),
                        };

                        if desc.stride < desc.width * bytes_per_pixel_plane {
                             tracing::error!("DMABUF plane {} stride {} is less than expected minimum {} for placeholder upload.", idx, desc.stride, desc.width * bytes_per_pixel_plane);
                             return Err(RendererError::Generic(format!("DMABUF plane {} stride too small for placeholder", idx)));
                        }

                        self.queue.write_texture(
                            wgpu::ImageCopyTexture { texture: &plane_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                            &pixel_data,
                            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(desc.width * bytes_per_pixel_plane), rows_per_image: Some(desc.height) },
                            plane_texture_descriptor.size,
                        );

                        plane_wgpu_textures.push(Arc::new(plane_texture));
                        plane_wgpu_views.push(Arc::new(plane_wgpu_textures.last().unwrap().create_view(&wgpu::TextureViewDescriptor::default())));
                    }
                }

                if plane_wgpu_textures.is_empty() {
                    tracing::error!("No valid DMABUF planes found for surface {}, buffer {}", _surface_id.id(), id);
                    return Err(RendererError::Generic("No valid DMABUF planes processed".to_string()));
                }

                let is_multi_planar_bool = num_valid_planes > 1;
                // Or determine based on known YUV formats (e.g. if first plane is R8 and second is RG88 for NV12)
                // For now, simple count > 1 implies multi-planar for rendering purposes.

                // The primary texture/view for WgpuRenderableTexture will be the first plane (e.g. Y plane)
                let primary_plane_texture_owned = Arc::try_unwrap(plane_wgpu_textures[0].clone()).expect("Arc unwrap failed for DMABUF primary plane texture");
                let primary_plane_view_owned = Arc::try_unwrap(plane_wgpu_views[0].clone()).expect("Arc unwrap failed for DMABUF primary plane view");

                Ok(Box::new(WgpuRenderableTexture::new(
                    self.device.clone(),
                    primary_plane_texture_owned,
                    primary_plane_view_owned,
                    Arc::try_unwrap(self.default_sampler.clone()).expect("Arc unwrap failed for DMABUF sampler"),
                    *buffer_overall_width, // Overall image width
                    *buffer_overall_height, // Overall image height
                    primary_plane_format_wgpu.unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb), // Fallback, should always be Some
                    None, // TODO: Determine combined FourCC for multi-planar (e.g., Fourcc::NV12)
                    is_multi_planar_bool,
                    Some(plane_wgpu_textures),
                    Some(plane_wgpu_views),
                )))
            }
        }
    }
                // ANCHOR [DmabufImportSinglePlane]
                // This section attempts to import a single-plane DMABUF.
                // WARNING: Direct DMABUF import into a wgpu::Texture is a highly advanced
                // and platform/backend-specific feature. The high-level wgpu API may not
                // expose this directly in a stable, cross-platform way. This implementation
                // outlines the conceptual steps and potential unsafe code required.
                // Full implementation likely requires wgpu-hal or specific WGPU features.

                let primary_descriptor = match descriptors[0] {
                    Some(ref desc) => desc,
                    None => {
                        eprintln!("DMABUF import error for surface {}: No primary plane descriptor provided.", _surface_id.id());
                        return Err(RendererError::Generic("Missing primary DMABUF descriptor".to_string()));
                    }
                };

                // TODO: Map DmabufPlaneFormat to wgpu::TextureFormat more robustly.
                // For now, assume Argb8888/Xrgb8888 maps to a Bgra8Unorm variant.
                // The actual surface_config.format might be sRGB, so using Bgra8Unorm (non-sRGB) for data upload
                // and letting the view handle sRGB conversion if necessary, or ensuring format consistency.
                let texture_format = match primary_descriptor.format {
                    DmabufPlaneFormat::Argb8888 => wgpu::TextureFormat::Bgra8Unorm,
                    DmabufPlaneFormat::Xrgb8888 => wgpu::TextureFormat::Bgra8Unorm,
                    // Add other single-plane formats if necessary, e.g. R8, Rg8 etc.
                    _ => {
                        eprintln!("DMABUF import error for surface {}: Unsupported single-plane format: {:?}", _surface_id.id(), primary_descriptor.format);
                        return Err(RendererError::Generic(format!("Unsupported DMABUF single-plane format: {:?}", primary_descriptor.format)));
                    }
                };

                let texture_descriptor = wgpu::TextureDescriptor {
                    label: Some(&format!("dmabuf_surface_{}_b{}", _surface_id.id(), id)),
                    size: wgpu::Extent3d {
                        width: primary_descriptor.width,
                        height: primary_descriptor.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: texture_format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, // COPY_DST is essential for write_texture fallback.
                                                                                                // For direct import, usage might be just TEXTURE_BINDING if sampleable.
                    view_formats: &[], // TODO: Investigate if view_formats are needed for DMABUF, esp. if main format is sRGB.
                };

                let mut imported_texture: Option<wgpu::Texture> = None;

                // --- Begin Hypothetical/Unsafe WGPU DMABUF Import ---
                // This is where the platform-specific magic would happen.
                // The exact API depends heavily on WGPU version and enabled features/backends.
                // TODO [WgpuDmabufImportActualApi]: Implement actual DMABUF import using appropriate WGPU API
                // when available/chosen (e.g. device.import_external_texture_from_fd or TextureDescriptor with external_memory_handle)
                // This will involve unsafe code and careful handling of FDs, strides, and modifiers.
                // Ensure all necessary WGPU features and instance/device extensions are enabled.
                // Example conceptual path using a hypothetical future wgpu API or direct HAL access:
                //
                // #[cfg(target_os = "linux")] // Or specific backend feature
                // {
                //     // This is pseudo-code, actual API will differ.
                //     // Option 1: If wgpu::TextureDescriptor supports external memory handles:
                //     // let external_memory_info = wgpu::ExternalMemoryDescriptor::DmaBuf {
                //     //     fd: primary_descriptor.fd,
                //     //     stride: primary_descriptor.stride,
                //     //     modifier: primary_descriptor.modifier, // If supported
                //     //     // Plane info might be part of this or implied by format
                //     // };
                //     // let texture_descriptor_with_import = wgpu::TextureDescriptor {
                //     //     // ... regular fields ...
                //     //     external_memory_handle: Some(external_memory_info), // Hypothetical field
                //     // };
                //     // unsafe {
                //     //     match self.device.create_texture_with_external_memory(&texture_descriptor_with_import) {
                //     //         Ok(tex) => imported_texture = Some(tex),
                //     //         Err(e) => eprintln!("DMABUF import via create_texture_with_external_memory failed: {:?}", e),
                //     //     }
                //     // }
                //
                //     // Option 2: Using a device method like `import_external_texture_from_fd`
                //     // unsafe {
                //     //     match self.device.import_external_texture_from_fd(
                //     //         &texture_descriptor, // Base descriptor
                //     //         primary_descriptor.fd,
                //     //         primary_descriptor.stride, // And other plane/modifier info
                //     //     ) {
                //     //         Ok(tex) => imported_texture = Some(tex),
                //     //         Err(e) => eprintln!("DMABUF import via import_external_texture_from_fd failed: {:?}", e),
                //     //     }
                //     // }
                // }
                //
                // If direct import is not available or fails, one might need to:
                // 1. Create a CPU-accessible staging buffer from the DMABUF FD (mmap).
                // 2. Create a regular GPU texture.
                // 3. Copy from the staging buffer to the GPU texture. This is NOT zero-copy.
                //    This would be similar to the SHM path but with mmap first.
                // --- End Hypothetical/Unsafe WGPU DMABUF Import ---

                // Fallback to magenta placeholder if import_texture is None
                let final_texture = if let Some(tex) = imported_texture {
                    tracing::info!("DMABUF for surface {} buffer {} successfully imported (conceptually).", _surface_id.id(), id);
                    tex
                } else {
                    // ANCHOR [DmabufUploadPlaceholderActive]
                    tracing::warn!("DMABUF import for surface {} buffer {} failed or not implemented, using placeholder.", _surface_id.id(), id);

                    let placeholder_texture = self.device.create_texture(&texture_descriptor); // Use the descriptor already defined

                    // Create magenta placeholder data matching the descriptor's format and dimensions
                    // Assuming 4 bytes per pixel for Bgra8Unorm
                    let bytes_per_pixel = 4; // For Bgra8Unorm
                    let magenta_pixel: [u8; 4] = [255, 0, 255, 255]; // BGRA: Blue, Green, Red, Alpha (Magenta for BGRA)
                                                                    // If format was Rgba8Unorm, it would be [255,0,255,255] for RGBA magenta

                    let mut pixel_data = vec![0u8; (primary_descriptor.width * primary_descriptor.height * bytes_per_pixel) as usize];
                    for i in (0..pixel_data.len()).step_by(bytes_per_pixel) {
                        pixel_data[i..i+bytes_per_pixel].copy_from_slice(&magenta_pixel);
                    }

                    // Check if primary_descriptor.stride matches theoretical stride for placeholder
                    if primary_descriptor.stride < primary_descriptor.width * bytes_per_pixel {
                        tracing::error!("DMABUF primary plane stride {} is less than expected minimum {} for placeholder upload.", primary_descriptor.stride, primary_descriptor.width * bytes_per_pixel);
                        return Err(RendererError::Generic("DMABUF stride too small for placeholder".to_string()));
                    }

                    self.queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &placeholder_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &pixel_data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            // For the placeholder, we use its own packed stride.
                            // If we were to try and use primary_descriptor.stride, it must be validated.
                            bytes_per_row: Some(primary_descriptor.width * bytes_per_pixel),
                            rows_per_image: Some(primary_descriptor.height),
                        },
                        texture_descriptor.size,
                    );
                    placeholder_texture
                };

                let view = final_texture.create_view(&wgpu::TextureViewDescriptor::default());
                // Sampler can be reused or created per texture as needed.
                // Ensure default_sampler is available, otherwise this will panic.
                let sampler = self.default_sampler.clone(); // Clone Arc<Sampler>

                Ok(Box::new(WgpuRenderableTexture::new(
                    self.device.clone(),
                    final_texture,
                    view,
                    sampler,
                    primary_descriptor.width,
                    primary_descriptor.height,
                    texture_descriptor.format,
                    None, // TODO: Pass actual DmabufFdInfo if WgpuRenderableTexture needs it for release or other ops
                )))
            }
        }
    }
     fn create_texture_from_shm(&mut self, _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer) -> Result<Box<dyn RenderableTexture>, RendererError> {
        Err(RendererError::Unsupported("Direct SHM WlBuffer import not used in this flow".to_string()))
    }
    fn create_texture_from_dmabuf( &mut self, _dmabuf: &smithay::backend::allocator::dmabuf::Dmabuf) -> Result<Box<dyn RenderableTexture>, RendererError> {
        Err(RendererError::Unsupported("Direct DMABUF import not used in this flow".to_string()))
    }
}
// TODO [ErrorHandlingAndRecovery]: Ensure all public methods of the renderer handle potential WGPU errors
// and propagate them as `RendererError`. Consider specific error variants for common issues like
// texture creation failure, buffer creation failure, shader compilation failure, etc.
// (see also ANCHOR [ErrorHandlingAndRecoveryOutline] in new() and ANCHOR [ErrorHandlingAndRecoveryOutlinePoint2] in render_frame())
// ANCHOR [ErrorHandlingAndRecoveryOutlineEnd]
