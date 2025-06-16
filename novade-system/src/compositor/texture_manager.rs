// texture_manager.rs
use crate::compositor::renderer_interface::abstraction::{RenderableTexture, TextureFactory, RendererError};
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::reexports::drm_fourcc::DrmFourcc;

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
// Remove os::unix::io::AsRawFd if not used directly for fd_val in DmabufPlaneKey
// use std::os::unix::io::AsRawFd;

// Key for identifying planes within a DMABUF for the BufferKey
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DmabufPlaneKey {
    plane_index: usize,
    offset: u32,
    stride: u32,
    modifier: u64,
}

// Key for identifying client buffers
#[derive(Debug, Clone, Eq)]
pub enum BufferKey {
    Shm {
        id: u64 // Using a stable resource ID for wl_buffer
    },
    Dmabuf {
        width: u32,
        height: u32,
        format: DrmFourcc, // Store DrmFourcc directly
        planes: Vec<DmabufPlaneKey>,
    },
}

impl BufferKey {
    pub fn from_dmabuf(dmabuf: &Dmabuf) -> Self {
        let mut plane_keys = Vec::new();
        for i in 0..dmabuf.num_planes() {
            plane_keys.push(DmabufPlaneKey {
                plane_index: i,
                offset: dmabuf.offsets()[i],
                stride: dmabuf.strides()[i],
                modifier: dmabuf.modifiers()[i],
            });
        }

        BufferKey::Dmabuf {
            width: dmabuf.width(),
            height: dmabuf.height(),
            format: dmabuf.format(),
            planes: plane_keys,
        }
    }

    // Note: from_shm helper removed as direct construction `BufferKey::Shm { id }` is clear.
    // The `id` should be obtained by the caller (e.g., from `WlBuffer::id().protocol_id()`).

    /// Creates a BufferKey from a Smithay WlBuffer (SHM).
    /// The `id` should be a stable identifier for the buffer resource.
    pub fn from_shm(buffer: &WlBuffer) -> Self {
        // Smithay's `Resource::id()` returns `ObjectId`.
        // `ObjectId::as_ptr() as u64` provides a u64 value based on the object's address.
        // This is suitable for identity as long as the same logical buffer resource object isn't moved
        // or its ObjectId doesn't change unexpectedly during its lifetime relevant to caching.
        // For SHM buffers, which are often recreated or replaced, using this as a key means
        // different instances of a buffer (even if same content) might get different keys.
        // If the WlBuffer is a smithay Resource object, buffer.id() gives ObjectId.
        BufferKey::Shm { id: buffer.id().as_ptr() as u64 }
    }
}

impl PartialEq for BufferKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Shm { id: l_id }, Self::Shm { id: r_id }) => l_id == r_id,
            (
                Self::Dmabuf { width: lw, height: lh, format: lf, planes: lp },
                Self::Dmabuf { width: rw, height: rh, format: rf, planes: rp },
            ) => lw == rw && lh == rh && *lf == *rf && lp == rp, // Compare DrmFourcc directly
            _ => false,
        }
    }
}

impl Hash for BufferKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            BufferKey::Shm { id } => {
                id.hash(state);
            }
            BufferKey::Dmabuf { width, height, format, planes } => {
                width.hash(state);
                height.hash(state);
                format.as_u32().hash(state); // Hash DrmFourcc as u32
                planes.hash(state);
            }
        }
    }
}

pub struct TextureCacheEntry {
    texture: Arc<dyn RenderableTexture>, // Now a trait object
    timestamp: u64,
    memory_size: u64,
}

// Enum to pass buffer source to get_or_create_texture
pub enum BufferSource<'a> {
    Shm(&'a WlBuffer),
    Dmabuf(&'a Dmabuf),
}


pub struct TextureManager {
    cache: RwLock<HashMap<BufferKey, TextureCacheEntry>>,
    lru_order: RwLock<VecDeque<BufferKey>>,
    max_memory_usage: u64,
    current_memory_usage: RwLock<u64>,
}

impl TextureManager {
    pub fn new(max_memory_usage: u64) -> Self {
        TextureManager {
            cache: RwLock::new(HashMap::new()),
            lru_order: RwLock::new(VecDeque::new()),
            max_memory_usage,
            current_memory_usage: RwLock::new(0),
        }
    }

    fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    pub fn get_or_create_texture(
        &self,
        key: BufferKey,
        buffer_source: BufferSource<'_>, // Use the new enum
        factory: &mut dyn TextureFactory,
    ) -> Result<Arc<dyn RenderableTexture>, RendererError> {
        // Check cache first (read lock)
        {
            let cache_read = self.cache.read().unwrap();
            if let Some(entry) = cache_read.get(&key) {
                drop(cache_read);
                self.promote_in_lru(&key);
                let cache_read_again = self.cache.read().unwrap(); // Re-acquire lock
                return Ok(cache_read_again.get(&key).unwrap().texture.clone());
            }
        }

        let new_texture_result = match buffer_source {
            BufferSource::Shm(wl_buffer) => factory.create_texture_from_shm(wl_buffer),
            BufferSource::Dmabuf(dmabuf) => factory.create_texture_from_dmabuf(dmabuf),
        };

        let new_texture = match new_texture_result {
            Ok(tex) => tex,
            Err(e) => {
                log::error!("TextureFactory failed to create texture for key {:?}: {:?}", key, e);
                return Err(e);
            }
        };

        let memory_size = new_texture.estimated_gpu_memory_size();
        self.evict_if_needed(memory_size);

        let mut cache_write = self.cache.write().unwrap();
        let mut lru_write = self.lru_order.write().unwrap();
        let mut current_memory_write = self.current_memory_usage.write().unwrap();

        if let Some(entry) = cache_write.get(&key) {
            drop(lru_write);
            drop(current_memory_write);
            drop(cache_write);
            self.promote_in_lru(&key);
            let cache_read_final = self.cache.read().unwrap();
            return Ok(cache_read_final.get(&key).unwrap().texture.clone());
        }

        let entry = TextureCacheEntry {
            texture: new_texture.clone(),
            timestamp: Self::get_current_timestamp(),
            memory_size,
        };

        cache_write.insert(key.clone(), entry);
        lru_write.push_front(key);
        *current_memory_write += memory_size;

        log::debug!(
            "Texture cached. ID: {:?}, Size: {}, Current Total Memory: {}",
            new_texture.id(), memory_size, *current_memory_write
        );

        Ok(new_texture)
    }

    fn promote_in_lru(&self, key: &BufferKey) {
        let mut lru_write = self.lru_order.write().unwrap();
        if let Some(pos) = lru_write.iter().position(|k| k == key) {
            if let Some(k) = lru_write.remove(pos) { // Check Some from remove
                lru_write.push_front(k);
                // Update timestamp
                drop(lru_write); // Release LRU lock before acquiring cache lock
                let mut cache_write_guard = self.cache.write().unwrap();
                if let Some(entry) = cache_write_guard.get_mut(key) {
                    entry.timestamp = Self::get_current_timestamp();
                }
            }
        }
    }

    fn evict_if_needed(&self, needed_space: u64) {
        // This loop needs to acquire locks carefully.
        loop {
            let current_mem = *self.current_memory_usage.read().unwrap();
            if current_mem + needed_space <= self.max_memory_usage {
                break;
            }

            let mut lru_write = self.lru_order.write().unwrap();
            if lru_write.is_empty() {
                if needed_space > self.max_memory_usage && current_mem == 0 {
                    log::warn!("Attempting to cache texture larger ({}) than max cache size ({}).", needed_space, self.max_memory_usage);
                }
                break;
            }

            let key_to_evict = lru_write.pop_back().unwrap();
            drop(lru_write); // Release LRU lock

            let mut cache_write = self.cache.write().unwrap();
            if let Some(evicted_entry) = cache_write.remove(&key_to_evict) {
                drop(cache_write); // Release cache lock before current_memory lock
                let mut current_memory_write_lock = self.current_memory_usage.write().unwrap();
                *current_memory_write_lock -= evicted_entry.memory_size;
                log::debug!(
                    "Evicted texture. Key: {:?}, Size: {}, Current Total Memory: {}",
                    key_to_evict, evicted_entry.memory_size, *current_memory_write_lock
                );
            } else {
                 log::warn!("LRU key {:?} not found in cache for eviction.", key_to_evict);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::renderer_interface::abstraction::{RenderableTexture, TextureFactory, RendererError};
    use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer; // For type in factory
    use smithay::backend::allocator::dmabuf::Dmabuf; // For type in factory
    use smithay::reexports::drm_fourcc::DrmFourcc;
    use uuid::Uuid;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::cell::RefCell; // For interior mutability in mock factory config

    // Simplified BufferSource descriptor for tests to guide key creation and factory behavior
    #[derive(Debug, Clone)]
    enum TestBufferSourceDescriptor {
        Shm { id: u64, width: u32, height: u32, bpp: u32 },
        Dmabuf { width: u32, height: u32, format: DrmFourcc, planes: Vec<DmabufPlaneKey>, bpp: u32 },
    }

    impl TestBufferSourceDescriptor {
        fn to_buffer_key(&self) -> BufferKey {
            match self {
                TestBufferSourceDescriptor::Shm { id, .. } => BufferKey::Shm { id: *id },
                TestBufferSourceDescriptor::Dmabuf { width, height, format, planes, .. } => BufferKey::Dmabuf {
                    width: *width,
                    height: *height,
                    format: *format,
                    planes: planes.clone(),
                },
            }
        }
        fn get_w_h_bpp(&self) -> (u32, u32, u32) {
            match self {
                TestBufferSourceDescriptor::Shm { width, height, bpp, .. } => (*width, *height, *bpp),
                TestBufferSourceDescriptor::Dmabuf { width, height, bpp, .. } => (*width, *height, *bpp),
            }
        }
    }


    #[derive(Debug)]
    struct MockRenderableTexture {
        uuid: Uuid,
        memory_size: u64,
        width: u32,
        height: u32,
    }

    impl RenderableTexture for MockRenderableTexture {
        fn id(&self) -> Uuid { self.uuid }
        fn bind(&self, _slot: u32) -> Result<(), RendererError> { Ok(()) }
        fn width_px(&self) -> u32 { self.width }
        fn height_px(&self) -> u32 { self.height }
        fn format(&self) -> Option<DrmFourcc> { Some(DrmFourcc::Argb8888) } // Mock
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn estimated_gpu_memory_size(&self) -> u64 { self.memory_size }
    }

    struct MockTextureFactory {
        shm_creations: AtomicUsize,
        dmabuf_creations: AtomicUsize,
        // Configurable behavior
        force_shm_error: RefCell<bool>,
        force_dmabuf_error: RefCell<bool>,
        next_texture_props: RefCell<Option<(u32, u32, u32)>>, // width, height, bpp for next texture
    }

    impl MockTextureFactory {
        fn new() -> Self {
            Self {
                shm_creations: AtomicUsize::new(0),
                dmabuf_creations: AtomicUsize::new(0),
                force_shm_error: RefCell::new(false),
                force_dmabuf_error: RefCell::new(false),
                next_texture_props: RefCell::new(None),
            }
        }

        fn set_force_shm_error(&self, val: bool) { *self.force_shm_error.borrow_mut() = val; }
        fn set_force_dmabuf_error(&self, val: bool) { *self.force_dmabuf_error.borrow_mut() = val; }
        fn set_next_texture_props(&self, w: u32, h: u32, bpp: u32) {
            *self.next_texture_props.borrow_mut() = Some((w,h,bpp));
        }
    }

    impl TextureFactory for MockTextureFactory {
        fn create_texture_from_shm(&mut self, _buffer: &WlBuffer) -> Result<Arc<dyn RenderableTexture>, RendererError> {
            if *self.force_shm_error.borrow() {
                return Err(RendererError::Generic("Mock SHM creation error".to_string()));
            }
            self.shm_creations.fetch_add(1, Ordering::SeqCst);
            let (w,h,bpp) = self.next_texture_props.borrow_mut().take().unwrap_or((100,100,4)); // Default if not set
            Ok(Arc::new(MockRenderableTexture {
                uuid: Uuid::new_v4(),
                memory_size: (w * h * bpp) as u64,
                width: w,
                height: h,
            }))
        }

        fn create_texture_from_dmabuf(&mut self, _dmabuf: &Dmabuf) -> Result<Arc<dyn RenderableTexture>, RendererError> {
            if *self.force_dmabuf_error.borrow() {
                return Err(RendererError::DmabufImportFailed("Mock DMABUF creation error".to_string()));
            }
            self.dmabuf_creations.fetch_add(1, Ordering::SeqCst);
            let (w,h,bpp) = self.next_texture_props.borrow_mut().take().unwrap_or((200,200,4)); // Default if not set
            Ok(Arc::new(MockRenderableTexture {
                uuid: Uuid::new_v4(),
                memory_size: (w * h * bpp) as u64,
                width: w,
                height: h,
            }))
        }
    }

    // Helper to create a BufferSource for tests. Since WlBuffer and Dmabuf are hard to mock directly,
    // the mock factory will rely on TestBufferSourceDescriptor's data carried via BufferKey.
    // The actual WlBuffer/Dmabuf passed to get_or_create_texture can be dummy/unsafe references
    // as the mock factory won't use their internal data. This is a common pattern for testing
    // components that interact with complex external types.

    fn get_buffer_source_for_test<'a>(desc: &TestBufferSourceDescriptor) -> (BufferKey, BufferSource<'a>) {
        let key = desc.to_buffer_key();
        let buffer_source = match desc {
            TestBufferSourceDescriptor::Shm { .. } => {
                // Unsafe because we are providing a null pointer as WlBuffer.
                // This is ONLY okay because our MockTextureFactory for SHM does not dereference the WlBuffer.
                // A real test with a real factory would need a proper WlBuffer.
                let dummy_wl_buffer: &WlBuffer = unsafe { std::mem::transmute(std::ptr::null::<()>()) };
                BufferSource::Shm(dummy_wl_buffer)
            }
            TestBufferSourceDescriptor::Dmabuf { .. } => {
                // Similar to WlBuffer, creating a real Dmabuf is complex.
                // The MockTextureFactory for Dmabuf uses the key's properties (width, height, etc.)
                // which we extract from TestBufferSourceDescriptor.
                let dummy_dmabuf: &Dmabuf = unsafe { std::mem::transmute(std::ptr::null::<()>()) };
                BufferSource::Dmabuf(dummy_dmabuf)
            }
        };
        (key, buffer_source)
    }


    #[test]
    fn test_texture_manager_new() {
        let manager = TextureManager::new(1000);
        assert_eq!(*manager.current_memory_usage.read().unwrap(), 0);
        assert_eq!(manager.max_memory_usage, 1000);
        assert!(manager.cache.read().unwrap().is_empty());
        assert!(manager.lru_order.read().unwrap().is_empty());
    }

    #[test]
    fn test_get_or_create_texture_simple_insert_shm() {
        let manager = TextureManager::new(20_000_000);
        let mut factory = MockTextureFactory::new();
        let desc = TestBufferSourceDescriptor::Shm { id: 1, width: 1280, height: 720, bpp: 4 };
        factory.set_next_texture_props(1280, 720, 4);

        let (key, buffer_source_arg) = get_buffer_source_for_test(&desc);
        let texture_res = manager.get_or_create_texture(key.clone(), buffer_source_arg, &mut factory);
        assert!(texture_res.is_ok());
        let texture = texture_res.unwrap();

        assert_eq!(texture.width_px(), 1280);
        assert_eq!(texture.height_px(), 720);
        let expected_size = (1280 * 720 * 4) as u64;
        assert_eq!(texture.estimated_gpu_memory_size(), expected_size);
        assert_eq!(*manager.current_memory_usage.read().unwrap(), expected_size);
        assert_eq!(manager.cache.read().unwrap().len(), 1);
        assert_eq!(manager.lru_order.read().unwrap().len(), 1);
        assert_eq!(manager.lru_order.read().unwrap().front().unwrap(), &key);
        assert_eq!(factory.shm_creations.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_get_or_create_updates_lru_order() {
        let manager = TextureManager::new(50_000_000); // Enough for all
        let mut factory = MockTextureFactory::new();

        let desc1 = TestBufferSourceDescriptor::Shm { id: 1, width: 640, height: 480, bpp: 4 }; // ~1.2MB
        let desc2 = TestBufferSourceDescriptor::Shm { id: 2, width: 1280, height: 720, bpp: 4 }; // ~3.7MB
        let desc3 = TestBufferSourceDescriptor::Dmabuf { // ~8.3MB
            width: 1920, height: 1080, format: DrmFourcc::Argb8888,
            planes: vec![DmabufPlaneKey{plane_index:0, offset:0, stride: 1920*4, modifier:0}], bpp: 4
        };

        factory.set_next_texture_props(640,480,4);
        let (key1, bs1) = get_buffer_source_for_test(&desc1);
        manager.get_or_create_texture(key1.clone(), bs1, &mut factory).unwrap(); // LRU: key1

        factory.set_next_texture_props(1280,720,4);
        let (key2, bs2) = get_buffer_source_for_test(&desc2);
        manager.get_or_create_texture(key2.clone(), bs2, &mut factory).unwrap(); // LRU: key2, key1

        factory.set_next_texture_props(1920,1080,4);
        let (key3, bs3) = get_buffer_source_for_test(&desc3);
        manager.get_or_create_texture(key3.clone(), bs3, &mut factory).unwrap(); // LRU: key3, key2, key1

        {
            let lru = manager.lru_order.read().unwrap();
            assert_eq!(lru.front().unwrap(), &key3);
            assert_eq!(lru.get(1).unwrap(), &key2);
            assert_eq!(lru.back().unwrap(), &key1);
        }

        // Access key1 again, should promote it
        let (_, bs1_again) = get_buffer_source_for_test(&desc1); // BufferSource itself is not stored
        manager.get_or_create_texture(key1.clone(), bs1_again, &mut factory).unwrap(); // LRU: key1, key3, key2

        {
            let lru = manager.lru_order.read().unwrap();
            assert_eq!(lru.front().unwrap(), &key1);
            assert_eq!(lru.get(1).unwrap(), &key3);
            assert_eq!(lru.back().unwrap(), &key2);
        }
        assert_eq!(factory.shm_creations.load(Ordering::SeqCst), 2); // desc1, desc2
        assert_eq!(factory.dmabuf_creations.load(Ordering::SeqCst), 1); // desc3
                                                                        // key1 was accessed again, but it was a cache hit.
    }

    #[test]
    fn test_eviction_logic() {
        let s1_w=64, s1_h=64, s1_bpp=4; // size = 16384
        let s2_w=128, s2_h=128, s2_bpp=4; // size = 65536
        let s3_w=256, s3_h=256, s3_bpp=4; // size = 262144

        let size1 = (s1_w*s1_h*s1_bpp) as u64;
        let size2 = (s2_w*s2_h*s2_bpp) as u64;
        let size3 = (s3_w*s3_h*s3_bpp) as u64;

        let max_mem = size1 + size2; // Enough for 1 and 2, but not 1, 2, and 3. (16384 + 65536 = 81920)
        let manager = TextureManager::new(max_mem);
        let mut factory = MockTextureFactory::new();

        let desc1 = TestBufferSourceDescriptor::Shm { id: 1, width: s1_w, height: s1_h, bpp: s1_bpp };
        let desc2 = TestBufferSourceDescriptor::Dmabuf {
            width: s2_w, height: s2_h, format: DrmFourcc::Argb8888,
            planes: vec![DmabufPlaneKey{plane_index:0, offset:0, stride:s2_w*s2_bpp, modifier:0}], bpp: s2_bpp
        };
        let desc3 = TestBufferSourceDescriptor::Shm { id: 3, width: s3_w, height: s3_h, bpp: s3_bpp };

        // Add 1
        factory.set_next_texture_props(s1_w,s1_h,s1_bpp);
        let (key1, bs1) = get_buffer_source_for_test(&desc1);
        manager.get_or_create_texture(key1.clone(), bs1, &mut factory).unwrap(); // LRU: key1, Mem: size1
        assert_eq!(*manager.current_memory_usage.read().unwrap(), size1);

        // Add 2
        factory.set_next_texture_props(s2_w,s2_h,s2_bpp);
        let (key2, bs2) = get_buffer_source_for_test(&desc2);
        manager.get_or_create_texture(key2.clone(), bs2, &mut factory).unwrap(); // LRU: key2, key1, Mem: size1+size2
        assert_eq!(*manager.current_memory_usage.read().unwrap(), size1 + size2);
        assert_eq!(manager.cache.read().unwrap().len(), 2);

        // Add 3 (size3 = 262144). current_mem (size1+size2=81920) + needed (size3=262144) = 344064. max_mem = 81920.
        // Evict key1 (size1). current_mem = size2 (65536).
        // current_mem (size2) + needed (size3) = 65536 + 262144 = 327680. Still > max_mem.
        // Evict key2 (size2). current_mem = 0.
        // current_mem (0) + needed (size3) = 262144. Still > max_mem.
        // Oh, wait, max_mem is size1+size2. So 262144 is > 81920.
        // So key1 and key2 will be evicted. Then key3 will be added.
        factory.set_next_texture_props(s3_w,s3_h,s3_bpp);
        let (key3, bs3) = get_buffer_source_for_test(&desc3);
        manager.get_or_create_texture(key3.clone(), bs3, &mut factory).unwrap();

        assert_eq!(*manager.current_memory_usage.read().unwrap(), size3);
        let cache = manager.cache.read().unwrap();
        assert_eq!(cache.len(), 1);
        assert!(cache.contains_key(&key3));
        assert!(!cache.contains_key(&key1));
        assert!(!cache.contains_key(&key2));
        let lru = manager.lru_order.read().unwrap();
        assert_eq!(lru.front().unwrap(), &key3);
    }

    #[test]
    fn test_texture_creation_failure() {
        let manager = TextureManager::new(100_000);
        let mut factory = MockTextureFactory::new();
        factory.set_force_shm_error(true);

        let desc = TestBufferSourceDescriptor::Shm { id: 1, width: 10, height: 10, bpp: 4 };
        let (key, buffer_source_arg) = get_buffer_source_for_test(&desc);

        let result = manager.get_or_create_texture(key, buffer_source_arg, &mut factory);
        assert!(result.is_err());
        match result.err().unwrap() {
            RendererError::Generic(msg) => assert_eq!(msg, "Mock SHM creation error"),
            _ => panic!("Unexpected error type"),
        }
        assert_eq!(*manager.current_memory_usage.read().unwrap(), 0);
        assert!(manager.cache.read().unwrap().is_empty());
    }

    #[test]
    fn test_evict_texture_larger_than_max_cache_size() {
        let manager = TextureManager::new(1000); // Max cache size 1000
        let mut factory = MockTextureFactory::new();

        // Texture 1: small, fits
        let desc1 = TestBufferSourceDescriptor::Shm { id: 1, width: 10, height: 10, bpp: 4 }; // Size 400
        factory.set_next_texture_props(10,10,4);
        let (key1, bs1) = get_buffer_source_for_test(&desc1);
        manager.get_or_create_texture(key1.clone(), bs1, &mut factory).unwrap();
        assert_eq!(*manager.current_memory_usage.read().unwrap(), 400);

        // Texture 2: larger than max cache size
        let desc2 = TestBufferSourceDescriptor::Shm { id: 2, width: 20, height: 20, bpp: 4 }; // Size 1600
        factory.set_next_texture_props(20,20,4);
        let (key2, bs2) = get_buffer_source_for_test(&desc2);
        // evict_if_needed will be called with needed_space = 1600.
        // current_mem (400) + needed_space (1600) = 2000 > max_mem (1000).
        // Evict key1 (400). current_mem becomes 0.
        // Now, current_mem (0) + needed_space (1600) = 1600 > max_mem (1000).
        // LRU is empty. Warning will be logged. Texture will be added.
        manager.get_or_create_texture(key2.clone(), bs2, &mut factory).unwrap();

        assert_eq!(*manager.current_memory_usage.read().unwrap(), 1600); // Texture was added
        let cache = manager.cache.read().unwrap();
        assert_eq!(cache.len(), 1); // key1 should be evicted
        assert!(cache.contains_key(&key2));
        assert!(!cache.contains_key(&key1));
    }
}
