use smithay::{
    reexports::wayland_server::{
        backend::ClientId,
        protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
        UserDataMap, Weak,
    },
    utils::{BufferCoords, Logical, Rectangle, Region, Size, Transform},
};
use std::sync::Mutex;
use uuid::Uuid;

use crate::compositor::renderer_interface::abstraction::RenderableTexture; // Assuming this path

#[derive(Debug)]
pub struct AttachedBufferInfo {
    pub buffer: WlBuffer,
    pub scale: i32,
    pub transform: Transform,
    pub dimensions: Size<i32, BufferCoords>,
}

#[derive(Debug)]
pub struct SurfaceData {
    pub id: Uuid,
    pub role: Mutex<Option<String>>,
    pub client_id: ClientId,
    pub current_buffer_info: Mutex<Option<AttachedBufferInfo>>,
    pub texture_handle: Mutex<Option<Box<dyn RenderableTexture>>>,
    pub damage_buffer_coords: Mutex<Vec<Rectangle<i32, smithay::utils::Buffer>>>,
    pub opaque_region_surface_local: Mutex<Option<Region<Logical>>>,
    pub input_region_surface_local: Mutex<Option<Region<Logical>>>,
    pub user_data_ext: UserDataMap,
    pub parent: Mutex<Option<Weak<WlSurface>>>,
    pub children: Mutex<Vec<Weak<WlSurface>>>,
}

impl SurfaceData {
    pub fn new(client_id: ClientId) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: Mutex::new(None),
            client_id,
            current_buffer_info: Mutex::new(None),
            texture_handle: Mutex::new(None),
            damage_buffer_coords: Mutex::new(Vec::new()),
            opaque_region_surface_local: Mutex::new(None),
            input_region_surface_local: Mutex::new(None),
            user_data_ext: UserDataMap::new(),
            parent: Mutex::new(None),
            children: Mutex::new(Vec::new()),
        }
    }
}
