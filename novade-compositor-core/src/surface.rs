use novade_buffer_manager::{BufferManager, BufferDetails, BufferId, ClientId, BufferFormat};
use std::sync::{Arc, Mutex};

// Placeholder for Wayland output transform
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WlOutputTransform {
    Normal,
    // Add other transform types as needed
}

// Placeholder for Mat3x3
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3x3 {
    pub m: [f32; 9],
}

impl Default for Mat3x3 {
    fn default() -> Self {
        Self {
            m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], // Identity matrix
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::surface_registry::{SurfaceRegistry, SurfaceRegistryAccessor};
    use crate::buffer_manager::{BufferManager, BufferDetails, BufferType, BufferFormat, ClientId};
    use crate::subcompositor::SubsurfaceSyncMode;
    use std::sync::atomic::Ordering;

    // Helper to create a simple BufferManager for tests
    fn test_buffer_manager() -> BufferManager {
        BufferManager::new()
    }

    // Helper to create a simple SurfaceRegistry for tests
    fn test_surface_registry() -> SurfaceRegistry {
        SurfaceRegistry::new()
    }

    #[test]
    fn test_unique_surface_ids() {
        let id1 = SurfaceId::new_unique();
        let id2 = SurfaceId::new_unique();
        assert_ne!(id1, id2, "SurfaceId::new_unique should generate unique IDs.");
    }

    #[test]
    fn test_surface_creation_defaults() {
        let surface = Surface::new();
        assert_eq!(surface.current_state, SurfaceState::Created, "Initial state should be Created.");
        assert_eq!(surface.pending_attributes, SurfaceAttributes::default(), "Pending attributes should be default.");
        assert_eq!(surface.current_attributes, SurfaceAttributes::default(), "Current attributes should be default.");
        assert!(surface.pending_buffer.is_none(), "Pending buffer should be None initially.");
        assert!(surface.current_buffer.is_none(), "Current buffer should be None initially.");
        assert!(surface.cached_pending_buffer.is_none(), "Cached pending buffer should be None initially.");
        assert!(surface.cached_pending_attributes.is_none(), "Cached pending attributes should be None initially.");
        assert!(surface.children.is_empty(), "Children list should be empty initially.");
        assert!(surface.parent.is_none(), "Parent should be None initially.");
        assert!(surface.frame_callbacks.is_empty(), "Frame callbacks should be empty initially.");

        let default_opaque_region = Region::new();
        let default_input_region = Region::new();
        assert_eq!(surface.opaque_region.as_ref().unwrap().rectangles, default_opaque_region.rectangles, "Opaque region should be default/empty.");
        assert_eq!(surface.input_region.as_ref().unwrap().rectangles, default_input_region.rectangles, "Input region should be default/empty.");
    }

    #[test]
    fn test_register_new_surface_in_registry() {
        let mut registry = test_surface_registry();
        let (id, surface_arc) = registry.register_new_surface();

        assert_eq!(surface_arc.lock().unwrap().id, id, "Registered surface ID should match returned ID.");
        assert!(registry.get_surface(id).is_some(), "Surface should be retrievable from registry.");
        let retrieved_arc = registry.get_surface(id).unwrap();
        assert!(Arc::ptr_eq(&surface_arc, &retrieved_arc), "Retrieved Arc should be the same as registered Arc.");
    }

    #[test]
    fn test_get_surface_from_registry() {
        let mut registry = test_surface_registry();
        let (id, _) = registry.register_new_surface();

        let retrieved_arc = registry.get_surface(id);
        assert!(retrieved_arc.is_some(), "Should retrieve registered surface.");
        assert_eq!(retrieved_arc.unwrap().lock().unwrap().id, id);

        let non_existent_id = SurfaceId::new_unique();
        assert!(registry.get_surface(non_existent_id).is_none(), "Should return None for non-existent surface.");
    }

    #[test]
    fn test_unregister_surface_simple() {
        let mut registry = test_surface_registry();
        let mut buffer_manager = test_buffer_manager();
        let (id, surface_arc) = registry.register_new_surface();

        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);
        let buffer_details_arc = buffer_manager.register_buffer(BufferType::Shm, 10, 10, 40, BufferFormat::Argb8888, client_id);

        {
            let mut surface_guard = surface_arc.lock().unwrap();
            surface_guard.attach_buffer(&mut buffer_manager, Some(buffer_details_arc.clone()), client_id_val, 0, 0).unwrap();
            surface_guard.commit(&mut buffer_manager).unwrap();
        }

        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2);

        let unregistered_surface_arc_opt = registry.unregister_surface(id, &mut buffer_manager);
        assert!(unregistered_surface_arc_opt.is_some());
        let unregistered_surface_arc = unregistered_surface_arc_opt.unwrap();
        assert_eq!(unregistered_surface_arc.lock().unwrap().id, id);
        assert!(registry.get_surface(id).is_none());

        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1);
        assert_eq!(unregistered_surface_arc.lock().unwrap().current_state, SurfaceState::Destroyed);
    }

    #[test]
    fn test_attach_null_buffer() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id = ClientId::new(1);

        assert_eq!(surface.pending_attributes.size, (0,0));

        let result = surface.attach_buffer(&mut buffer_manager, None, client_id, 0, 0);
        assert!(result.is_ok());
        assert!(surface.pending_buffer.is_none());
        assert_eq!(surface.current_state, SurfaceState::PendingBuffer);
        assert_eq!(surface.pending_attributes.size, (0,0));

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_ok());
        assert!(surface.current_buffer.is_none());
        assert_eq!(surface.current_state, SurfaceState::Committed);
        assert_eq!(surface.current_attributes.size, (0,0));
    }

    #[test]
    fn test_attach_buffer_valid() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        let buffer_details_arc = buffer_manager.register_buffer(
            BufferType::Shm, 64, 64, 256, BufferFormat::Argb8888, client_id
        );
        let buffer_id = buffer_details_arc.lock().unwrap().id;

        let attach_result = surface.attach_buffer(&mut buffer_manager, Some(buffer_details_arc.clone()), client_id_val, 0, 0);
        assert!(attach_result.is_ok());

        assert!(surface.pending_buffer.is_some());
        assert_eq!(surface.pending_buffer.as_ref().unwrap().lock().unwrap().id, buffer_id);
        assert_eq!(surface.pending_attributes.size, (64,64));
        assert_eq!(surface.pending_attributes.buffer_offset, (0,0));
        assert_eq!(surface.current_state, SurfaceState::PendingBuffer);
        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_attach_buffer_invalid_offset() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        let buffer_details_arc = buffer_manager.register_buffer(
            BufferType::Shm, 64, 64, 256, BufferFormat::Argb8888, client_id
        );

        let attach_result = surface.attach_buffer(&mut buffer_manager, Some(buffer_details_arc.clone()), client_id_val, 10, 5);
        assert!(attach_result.is_err());
        match attach_result.err().unwrap() {
            BufferAttachError::InvalidBufferOffset => {},
            e => panic!("Expected InvalidBufferOffset, got {:?}", e),
        }
        assert!(surface.pending_buffer.is_none());
        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_commit_simple_attributes() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();

        surface.pending_attributes.alpha = 0.5;
        surface.pending_attributes.buffer_scale = 2;

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_ok());

        assert_eq!(surface.current_attributes.alpha, 0.5);
        assert_eq!(surface.current_attributes.buffer_scale, 2);
        assert_eq!(surface.current_attributes.size, (0,0));
        assert_eq!(surface.current_state, SurfaceState::Committed);
    }

    #[test]
    fn test_commit_buffer_attach() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        let buffer1_arc = buffer_manager.register_buffer(
            BufferType::Shm, 32, 32, 128, BufferFormat::Argb8888, client_id
        );
        surface.attach_buffer(&mut buffer_manager, Some(buffer1_arc.clone()), client_id_val, 0, 0).unwrap();
        surface.commit(&mut buffer_manager).unwrap();

        assert!(surface.current_buffer.is_some());
        assert_eq!(surface.current_buffer.as_ref().unwrap().lock().unwrap().id, buffer1_arc.lock().unwrap().id);
        assert_eq!(surface.current_attributes.size, (32,32));
        assert_eq!(buffer1_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2);
        assert_eq!(surface.current_state, SurfaceState::Committed);

        let buffer2_arc = buffer_manager.register_buffer(
            BufferType::Shm, 64, 64, 256, BufferFormat::Xrgb8888, client_id
        );
        surface.attach_buffer(&mut buffer_manager, Some(buffer2_arc.clone()), client_id_val, 0, 0).unwrap();
        assert!(surface.pending_buffer.is_some());

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_ok());

        assert!(surface.current_buffer.is_some());
        assert_eq!(surface.current_buffer.as_ref().unwrap().lock().unwrap().id, buffer2_arc.lock().unwrap().id);
        assert!(surface.pending_buffer.is_none());
        assert_eq!(surface.current_attributes.size, (64,64));
        assert_eq!(buffer1_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1);
        assert_eq!(buffer2_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2);
        assert_eq!(surface.current_state, SurfaceState::Committed);
    }

    #[test]
    fn test_commit_validation_invalid_size_due_to_scale() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        let buffer_arc = buffer_manager.register_buffer(
            BufferType::Shm, 63, 63, 63*4, BufferFormat::Argb8888, client_id
        );
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc.clone()), client_id_val, 0, 0).unwrap();

        surface.pending_attributes.buffer_scale = 2;

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_err());
        match commit_result.err().unwrap() {
            CommitError::InvalidBufferSize => {},
            e => panic!("Expected InvalidBufferSize, got {:?}", e),
        }

        assert!(surface.current_buffer.is_none());
        assert_ne!(surface.current_attributes.buffer_scale, 2);
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_damage_buffer_simple() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let buffer_arc = buffer_manager.register_buffer(BufferType::Shm, 100, 100, 400, BufferFormat::Argb8888, Some(client_id_val));
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc.clone()), client_id_val, 0, 0).unwrap();

        surface.damage_buffer(10, 10, 20, 20);
        assert_eq!(surface.damage_tracker.pending_damage_buffer.len(), 1);
        assert_eq!(surface.damage_tracker.pending_damage_buffer[0], Rectangle::new(10,10,20,20));

        surface.commit(&mut buffer_manager).unwrap();
        assert_eq!(surface.damage_tracker.current_damage.len(), 1);
        let expected_damage_after_commit = Rectangle::new(10,10,20,20).clipped_to(&Rectangle::new(0,0,100,100));
        assert_eq!(surface.damage_tracker.current_damage[0], expected_damage_after_commit);
    }

    #[test]
    fn test_damage_surface_simple() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        surface.pending_attributes.size = (100,100);
        surface.commit(&mut buffer_manager).unwrap();

        surface.damage_surface(15, 15, 25, 25);
        assert_eq!(surface.damage_tracker.pending_damage_surface.len(), 1);
        assert_eq!(surface.damage_tracker.pending_damage_surface[0], Rectangle::new(15,15,25,25));

        surface.commit(&mut buffer_manager).unwrap();
        assert_eq!(surface.damage_tracker.current_damage.len(), 1);
        let expected_damage_after_commit = Rectangle::new(15,15,25,25).clipped_to(&Rectangle::new(0,0,100,100));
        assert_eq!(surface.damage_tracker.current_damage[0], expected_damage_after_commit);
    }

    #[test]
    fn test_damage_commit_clears_pending() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        surface.pending_attributes.size = (100,100);
        surface.commit(&mut buffer_manager).unwrap();

        surface.damage_surface(10,10,20,20);
        assert!(!surface.damage_tracker.pending_damage_surface.is_empty());

        surface.commit(&mut buffer_manager).unwrap();
        assert!(surface.damage_tracker.pending_damage_surface.is_empty());
        assert!(surface.damage_tracker.pending_damage_buffer.is_empty());
    }

    #[test]
    fn test_damage_buffer_transform_and_clip() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);

        let buffer_arc = buffer_manager.register_buffer(BufferType::Shm, 200, 100, 800, BufferFormat::Argb8888, Some(client_id_val));
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc.clone()), client_id_val, 0, 0).unwrap();

        surface.pending_attributes.buffer_scale = 2;

        surface.damage_buffer(40, 20, 80, 40);

        surface.commit(&mut buffer_manager).unwrap();

        assert_eq!(surface.current_attributes.size, (100,50));
        assert_eq!(surface.damage_tracker.current_damage.len(), 1);

        let expected_rect = Rectangle::new(20, 10, 40, 20);
        let clipped_expected = expected_rect.clipped_to(&Rectangle::new(0,0,100,50));
        assert_eq!(surface.damage_tracker.current_damage[0], clipped_expected);

        surface.damage_buffer(180, 80, 40, 30);
        surface.commit(&mut buffer_manager).unwrap();
        assert_eq!(surface.damage_tracker.current_damage.len(), 1);
        let expected_rect2 = Rectangle::new(90,40,20,15);
        let clipped_expected2 = expected_rect2.clipped_to(&Rectangle::new(0,0,100,50));
        assert_eq!(clipped_expected2, Rectangle::new(90,40,10,10));
        assert_eq!(surface.damage_tracker.current_damage[0], clipped_expected2);
    }

    #[test]
    fn test_damage_overflow_fallback() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        surface.pending_attributes.size = (100,100);
        surface.commit(&mut buffer_manager).unwrap();

        surface.damage_surface(0,0,100,76);
        surface.commit(&mut buffer_manager).unwrap();

        assert_eq!(surface.damage_tracker.current_damage.len(), 1);
        assert_eq!(surface.damage_tracker.current_damage[0], Rectangle::new(0,0,100,100));
    }

    #[test]
    fn test_damage_age_reset_on_new_content_commit() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);

        surface.pending_attributes.size = (50,50);
        surface.commit(&mut buffer_manager).unwrap();

        surface.damage_surface(0,0,10,10);
        assert_eq!(surface.damage_tracker.damage_age, 1);
        surface.commit(&mut buffer_manager).unwrap();
        assert_eq!(surface.damage_tracker.damage_age, 1);

        surface.damage_surface(0,0,10,10);
        assert_eq!(surface.damage_tracker.damage_age, 2);

        let buffer_arc = buffer_manager.register_buffer(BufferType::Shm, 50, 50, 200, BufferFormat::Argb8888, Some(client_id_val));
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc), client_id_val, 0, 0).unwrap();
        surface.commit(&mut buffer_manager).unwrap();

        assert_eq!(surface.damage_tracker.damage_age, 0);
    }

    #[test]
    fn test_frame_callback_registration() {
        let mut surface = Surface::new();
        assert!(surface.frame_callbacks.is_empty());
        surface.frame(123);
        assert_eq!(surface.frame_callbacks.len(), 1);
        assert_eq!(surface.frame_callbacks[0].id, 123);
        surface.frame(456);
        assert_eq!(surface.frame_callbacks.len(), 2);
        assert_eq!(surface.frame_callbacks[1].id, 456);
    }

    #[test]
    fn test_take_frame_callbacks() {
        let mut surface = Surface::new();
        surface.frame(1);
        surface.frame(2);

        let taken_callbacks = surface.take_frame_callbacks();
        assert_eq!(taken_callbacks.len(), 2);
        assert_eq!(taken_callbacks[0].id, 1);
        assert_eq!(taken_callbacks[1].id, 2);
        assert!(surface.frame_callbacks.is_empty());

        let taken_again = surface.take_frame_callbacks();
        assert!(taken_again.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::surface_registry::{SurfaceRegistry, SurfaceRegistryAccessor};
    use crate::buffer_manager::{BufferManager, BufferDetails, BufferType, BufferFormat, ClientId};
    use crate::subcompositor::SubsurfaceSyncMode; // For later tests
    use std::sync::atomic::Ordering;

    // Helper to create a simple BufferManager for tests
    fn test_buffer_manager() -> BufferManager {
        BufferManager::new()
    }

    // Helper to create a simple SurfaceRegistry for tests
    fn test_surface_registry() -> SurfaceRegistry {
        SurfaceRegistry::new()
    }

    #[test]
    fn test_unique_surface_ids() {
        let id1 = SurfaceId::new_unique();
        let id2 = SurfaceId::new_unique();
        assert_ne!(id1, id2, "SurfaceId::new_unique should generate unique IDs.");
    }

    #[test]
    fn test_surface_creation_defaults() {
        let surface = Surface::new();
        assert_eq!(surface.current_state, SurfaceState::Created, "Initial state should be Created.");
        assert_eq!(surface.pending_attributes, SurfaceAttributes::default(), "Pending attributes should be default.");
        assert_eq!(surface.current_attributes, SurfaceAttributes::default(), "Current attributes should be default.");
        assert!(surface.pending_buffer.is_none(), "Pending buffer should be None initially.");
        assert!(surface.current_buffer.is_none(), "Current buffer should be None initially.");
        assert!(surface.cached_pending_buffer.is_none(), "Cached pending buffer should be None initially.");
        assert!(surface.cached_pending_attributes.is_none(), "Cached pending attributes should be None initially.");
        assert!(surface.children.is_empty(), "Children list should be empty initially.");
        assert!(surface.parent.is_none(), "Parent should be None initially.");
        assert!(surface.frame_callbacks.is_empty(), "Frame callbacks should be empty initially.");

        let default_opaque_region = Region::new(); // Assuming default region is empty or defined
        let default_input_region = Region::new();
        assert_eq!(surface.opaque_region.as_ref().unwrap().rectangles, default_opaque_region.rectangles, "Opaque region should be default/empty.");
        assert_eq!(surface.input_region.as_ref().unwrap().rectangles, default_input_region.rectangles, "Input region should be default/empty.");
    }

    #[test]
    fn test_register_new_surface_in_registry() {
        let mut registry = test_surface_registry();
        let (id, surface_arc) = registry.register_new_surface();

        assert_eq!(surface_arc.lock().unwrap().id, id, "Registered surface ID should match returned ID.");
        assert!(registry.get_surface(id).is_some(), "Surface should be retrievable from registry.");
        let retrieved_arc = registry.get_surface(id).unwrap();
        assert!(Arc::ptr_eq(&surface_arc, &retrieved_arc), "Retrieved Arc should be the same as registered Arc.");
    }

    #[test]
    fn test_get_surface_from_registry() {
        let mut registry = test_surface_registry();
        let (id, _) = registry.register_new_surface();

        let retrieved_arc = registry.get_surface(id);
        assert!(retrieved_arc.is_some(), "Should retrieve registered surface.");
        assert_eq!(retrieved_arc.unwrap().lock().unwrap().id, id);

        let non_existent_id = SurfaceId::new_unique();
        assert!(registry.get_surface(non_existent_id).is_none(), "Should return None for non-existent surface.");
    }

    #[test]
    fn test_unregister_surface_simple() {
        let mut registry = test_surface_registry();
        let mut buffer_manager = test_buffer_manager();
        let (id, surface_arc) = registry.register_new_surface();

        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);
        let buffer_details_arc = buffer_manager.register_buffer(BufferType::Shm, 10, 10, 40, BufferFormat::Argb8888, client_id);

        // Attach and commit buffer to surface
        {
            let mut surface_guard = surface_arc.lock().unwrap();
            surface_guard.attach_buffer(&mut buffer_manager, Some(buffer_details_arc.clone()), client_id_val, 0, 0).unwrap();
            surface_guard.commit(&mut buffer_manager).unwrap();
        }

        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2, "Buffer ref count should be 2 (manager + surface).");

        let unregistered_surface_arc_opt = registry.unregister_surface(id, &mut buffer_manager);
        assert!(unregistered_surface_arc_opt.is_some(), "Unregister should return the surface.");
        let unregistered_surface_arc = unregistered_surface_arc_opt.unwrap();
        assert_eq!(unregistered_surface_arc.lock().unwrap().id, id);
        assert!(registry.get_surface(id).is_none(), "Surface should be removed from registry after unregister.");

        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1, "Buffer ref count should be 1 (manager only) after surface unregister.");

        assert_eq!(unregistered_surface_arc.lock().unwrap().current_state, SurfaceState::Destroyed, "Surface state should be Destroyed.");
    }

    #[test]
    fn test_attach_null_buffer() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id = ClientId::new(1);

        // Initial size check
        assert_eq!(surface.pending_attributes.size, (0,0), "Initial pending size should be (0,0).");

        let result = surface.attach_buffer(&mut buffer_manager, None, client_id, 0, 0);
        assert!(result.is_ok(), "Attaching NULL buffer should be Ok.");
        assert!(surface.pending_buffer.is_none(), "Pending buffer should be None after attaching NULL.");
        assert_eq!(surface.current_state, SurfaceState::PendingBuffer, "Surface state should be PendingBuffer after attach(NULL).");

        // Check that pending_attributes.size is not changed by attach(NULL)
        assert_eq!(surface.pending_attributes.size, (0,0), "Pending size should remain (0,0) after attach(NULL) if not previously set by a real buffer.");

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_ok(), "Commit after attach(NULL) should be Ok.");
        assert!(surface.current_buffer.is_none(), "Current buffer should be None after commit of NULL buffer.");
        assert_eq!(surface.current_state, SurfaceState::Committed, "Surface state should be Committed.");
        assert_eq!(surface.current_attributes.size, (0,0), "Current size should be (0,0) after commit of attach(NULL).");
    }

    #[test]
    fn test_attach_buffer_valid() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        let buffer_details_arc = buffer_manager.register_buffer(
            BufferType::Shm, 64, 64, 256, BufferFormat::Argb8888, client_id
        );
        let buffer_id = buffer_details_arc.lock().unwrap().id;

        let attach_result = surface.attach_buffer(&mut buffer_manager, Some(buffer_details_arc.clone()), client_id_val, 0, 0);
        assert!(attach_result.is_ok());

        assert!(surface.pending_buffer.is_some(), "Pending buffer should be Some.");
        assert_eq!(surface.pending_buffer.as_ref().unwrap().lock().unwrap().id, buffer_id, "Correct buffer should be pending.");
        assert_eq!(surface.pending_attributes.size, (64,64), "Pending attributes size should be updated to buffer size.");
        assert_eq!(surface.pending_attributes.buffer_offset, (0,0), "Pending buffer offset should be set.");
        assert_eq!(surface.current_state, SurfaceState::PendingBuffer, "Surface state should be PendingBuffer.");
        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2, "Buffer ref count should be incremented to 2.");
    }

    #[test]
    fn test_attach_buffer_invalid_offset() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        let buffer_details_arc = buffer_manager.register_buffer(
            BufferType::Shm, 64, 64, 256, BufferFormat::Argb8888, client_id
        );

        let attach_result = surface.attach_buffer(&mut buffer_manager, Some(buffer_details_arc.clone()), client_id_val, 10, 5); // Non-zero offset
        assert!(attach_result.is_err(), "Attaching with non-zero offset should fail.");
        match attach_result.err().unwrap() {
            BufferAttachError::InvalidBufferOffset => { /* Correct error */ },
            e => panic!("Expected InvalidBufferOffset, got {:?}", e),
        }
        assert!(surface.pending_buffer.is_none(), "Pending buffer should remain None after failed attach.");
        assert_eq!(buffer_details_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1, "Buffer ref count should not change on failed attach.");
    }

    #[test]
    fn test_commit_simple_attributes() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();

        surface.pending_attributes.alpha = 0.5;
        surface.pending_attributes.buffer_scale = 2;
        // Note: Size is (0,0) by default and no buffer is attached.

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_ok());

        assert_eq!(surface.current_attributes.alpha, 0.5);
        assert_eq!(surface.current_attributes.buffer_scale, 2);
        assert_eq!(surface.current_attributes.size, (0,0)); // Size doesn't change without buffer
        assert_eq!(surface.current_state, SurfaceState::Committed);
    }

    #[test]
    fn test_commit_buffer_attach() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        // First buffer
        let buffer1_arc = buffer_manager.register_buffer(
            BufferType::Shm, 32, 32, 128, BufferFormat::Argb8888, client_id
        );
        surface.attach_buffer(&mut buffer_manager, Some(buffer1_arc.clone()), client_id_val, 0, 0).unwrap();
        surface.commit(&mut buffer_manager).unwrap();

        assert!(surface.current_buffer.is_some(), "Current buffer should be set after first commit.");
        assert_eq!(surface.current_buffer.as_ref().unwrap().lock().unwrap().id, buffer1_arc.lock().unwrap().id);
        assert_eq!(surface.current_attributes.size, (32,32));
        assert_eq!(buffer1_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2, "Buffer1 ref count should be 2.");
        assert_eq!(surface.current_state, SurfaceState::Committed);

        // Attach a second buffer
        let buffer2_arc = buffer_manager.register_buffer(
            BufferType::Shm, 64, 64, 256, BufferFormat::Xrgb8888, client_id
        );
        surface.attach_buffer(&mut buffer_manager, Some(buffer2_arc.clone()), client_id_val, 0, 0).unwrap();
        assert!(surface.pending_buffer.is_some());

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_ok());

        assert!(surface.current_buffer.is_some(), "Current buffer should be updated to buffer2.");
        assert_eq!(surface.current_buffer.as_ref().unwrap().lock().unwrap().id, buffer2_arc.lock().unwrap().id);
        assert!(surface.pending_buffer.is_none(), "Pending buffer should be None after commit.");
        assert_eq!(surface.current_attributes.size, (64,64), "Current attributes size should reflect buffer2.");
        assert_eq!(buffer1_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 1, "Buffer1 ref count should be 1 (released by surface).");
        assert_eq!(buffer2_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2, "Buffer2 ref count should be 2.");
        assert_eq!(surface.current_state, SurfaceState::Committed);
    }

    #[test]
    fn test_commit_validation_invalid_size_due_to_scale() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        let client_id = Some(client_id_val);

        let buffer_arc = buffer_manager.register_buffer(
            BufferType::Shm, 63, 63, 63*4, BufferFormat::Argb8888, client_id // Dimensions not divisible by 2
        );
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc.clone()), client_id_val, 0, 0).unwrap();

        // Set a scale that makes the buffer size invalid
        surface.pending_attributes.buffer_scale = 2;
        // Buffer width 63 % scale 2 != 0

        let commit_result = surface.commit(&mut buffer_manager);
        assert!(commit_result.is_err(), "Commit should fail due to invalid size with scale.");
        match commit_result.err().unwrap() {
            CommitError::InvalidBufferSize => { /* Correct error */ },
            e => panic!("Expected InvalidBufferSize, got {:?}", e),
        }

        // Ensure state was not applied
        assert!(surface.current_buffer.is_none(), "Current buffer should not be set on failed commit.");
        assert_ne!(surface.current_attributes.buffer_scale, 2, "Buffer scale should not be applied on failed commit.");
        assert_eq!(buffer_arc.lock().unwrap().ref_count.load(Ordering::SeqCst), 2, "Buffer ref count should still be 2 (pending on surface).");
    }

    // --- Damage Tracking Tests ---
    #[test]
    fn test_damage_buffer_simple() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);
        // Attach a buffer so damage can be applied relative to it
        let buffer_arc = buffer_manager.register_buffer(BufferType::Shm, 100, 100, 400, BufferFormat::Argb8888, Some(client_id_val));
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc.clone()), client_id_val, 0, 0).unwrap();

        surface.damage_buffer(10, 10, 20, 20);
        assert_eq!(surface.damage_tracker.pending_damage_buffer.len(), 1);
        assert_eq!(surface.damage_tracker.pending_damage_buffer[0], Rectangle::new(10,10,20,20));

        surface.commit(&mut buffer_manager).unwrap();
        assert_eq!(surface.damage_tracker.current_damage.len(), 1);
        // Assuming scale=1, transform=Normal for simplicity here.
        // Transformed and clipped rect will be tested separately.
        let expected_damage_after_commit = Rectangle::new(10,10,20,20).clipped_to(&Rectangle::new(0,0,100,100));
        assert_eq!(surface.damage_tracker.current_damage[0], expected_damage_after_commit);
    }

    #[test]
    fn test_damage_surface_simple() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        // Set a surface size for surface damage to be relative to
        surface.pending_attributes.size = (100,100);
        surface.commit(&mut buffer_manager).unwrap(); // Commit size

        surface.damage_surface(15, 15, 25, 25);
        assert_eq!(surface.damage_tracker.pending_damage_surface.len(), 1);
        assert_eq!(surface.damage_tracker.pending_damage_surface[0], Rectangle::new(15,15,25,25));

        surface.commit(&mut buffer_manager).unwrap();
        assert_eq!(surface.damage_tracker.current_damage.len(), 1);
        let expected_damage_after_commit = Rectangle::new(15,15,25,25).clipped_to(&Rectangle::new(0,0,100,100));
        assert_eq!(surface.damage_tracker.current_damage[0], expected_damage_after_commit);
    }

    #[test]
    fn test_damage_commit_clears_pending() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        surface.pending_attributes.size = (100,100); // Give surface a size
        surface.commit(&mut buffer_manager).unwrap();


        surface.damage_surface(10,10,20,20);
        assert!(!surface.damage_tracker.pending_damage_surface.is_empty());

        surface.commit(&mut buffer_manager).unwrap();
        assert!(surface.damage_tracker.pending_damage_surface.is_empty(), "Pending surface damage should be cleared after commit.");
        assert!(surface.damage_tracker.pending_damage_buffer.is_empty(), "Pending buffer damage should be cleared after commit (even if none was added).");
    }

    #[test]
    fn test_damage_buffer_transform_and_clip() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);

        let buffer_arc = buffer_manager.register_buffer(BufferType::Shm, 200, 100, 800, BufferFormat::Argb8888, Some(client_id_val));
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc.clone()), client_id_val, 0, 0).unwrap();

        // Surface attributes for transform: scale = 2
        // Buffer is 200x100. Surface will be 100x50.
        surface.pending_attributes.buffer_scale = 2;
        // No wl_output_transform for this test, just scale.

        // Damage in buffer coordinates
        surface.damage_buffer(40, 20, 80, 40); // This corresponds to (20,10,40,20) in surface coords

        surface.commit(&mut buffer_manager).unwrap();

        assert_eq!(surface.current_attributes.size, (100,50)); // Verifying surface size after scale
        assert_eq!(surface.damage_tracker.current_damage.len(), 1, "Expected 1 damage rect after commit.");

        // Expected damage in surface coordinates:
        // x = 40/2 = 20
        // y = 20/2 = 10
        // width = 80/2 = 40
        // height = 40/2 = 20
        let expected_rect = Rectangle::new(20, 10, 40, 20);
        // Clipped to surface size 100x50 (which it is within)
        let clipped_expected = expected_rect.clipped_to(&Rectangle::new(0,0,100,50));

        assert_eq!(surface.damage_tracker.current_damage[0], clipped_expected);

        // Test clipping: damage partially outside scaled buffer dimensions
        surface.damage_buffer(180, 80, 40, 30); // In buffer: (180,80,40,30). Scaled: (90,40,20,15)
                                               // Surface size: 100x50.
        surface.commit(&mut buffer_manager).unwrap();
        assert_eq!(surface.damage_tracker.current_damage.len(), 1);
        let expected_rect2 = Rectangle::new(90,40,20,15);
        let clipped_expected2 = expected_rect2.clipped_to(&Rectangle::new(0,0,100,50));
        // Expected: x=90, y=40, w=10 (clipped from 20), h=10 (clipped from 15)
        assert_eq!(clipped_expected2, Rectangle::new(90,40,10,10));
        assert_eq!(surface.damage_tracker.current_damage[0], clipped_expected2);
    }

    #[test]
    fn test_damage_overflow_fallback() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        surface.pending_attributes.size = (100,100);
        surface.commit(&mut buffer_manager).unwrap();

        // Add more damage rects than MAX_DAMAGE_RECTS (assuming MAX_DAMAGE_RECTS is small for test, e.g. 3)
        // Or, check if DamageTracker::MAX_DAMAGE_RECTS can be set for testing.
        // For now, simulate by adding rects that cover > 75% area.
        surface.damage_surface(0,0,100,76); // 76% area
        surface.commit(&mut buffer_manager).unwrap();

        assert_eq!(surface.damage_tracker.current_damage.len(), 1, "Damage should fallback to 1 rect.");
        assert_eq!(surface.damage_tracker.current_damage[0], Rectangle::new(0,0,100,100), "Damage should be full surface.");
    }

    #[test]
    fn test_damage_age_reset_on_new_content_commit() {
        let mut surface = Surface::new();
        let mut buffer_manager = test_buffer_manager();
        let client_id_val = ClientId::new(1);

        surface.pending_attributes.size = (50,50);
        surface.commit(&mut buffer_manager).unwrap(); // Initial commit, sets size

        surface.damage_surface(0,0,10,10); // damage_age becomes 1
        assert_eq!(surface.damage_tracker.damage_age, 1);
        surface.commit(&mut buffer_manager).unwrap(); // Commit damage, no new buffer, age should persist (or handled by renderer)
                                                      // Current logic in DamageTracker::commit_pending_damage resets age if new_buffer_committed.
                                                      // new_content_committed in Surface::commit is false here. So age should NOT reset.
        assert_eq!(surface.damage_tracker.damage_age, 1); // DamageTracker.commit called with new_buffer_committed=false

        surface.damage_surface(0,0,10,10); // damage_age becomes 2
        assert_eq!(surface.damage_tracker.damage_age, 2);

        // Now attach a new buffer (new content)
        let buffer_arc = buffer_manager.register_buffer(BufferType::Shm, 50, 50, 200, BufferFormat::Argb8888, Some(client_id_val));
        surface.attach_buffer(&mut buffer_manager, Some(buffer_arc), client_id_val, 0, 0).unwrap();
        surface.commit(&mut buffer_manager).unwrap(); // new_content_committed will be true

        assert_eq!(surface.damage_tracker.damage_age, 0, "Damage age should reset on new buffer commit.");
    }

    // --- Frame Callback Tests ---
    #[test]
    fn test_frame_callback_registration() {
        let mut surface = Surface::new();
        assert!(surface.frame_callbacks.is_empty());
        surface.frame(123);
        assert_eq!(surface.frame_callbacks.len(), 1);
        assert_eq!(surface.frame_callbacks[0].id, 123);
        surface.frame(456);
        assert_eq!(surface.frame_callbacks.len(), 2);
        assert_eq!(surface.frame_callbacks[1].id, 456);
    }

    #[test]
    fn test_take_frame_callbacks() {
        let mut surface = Surface::new();
        surface.frame(1);
        surface.frame(2);

        let taken_callbacks = surface.take_frame_callbacks();
        assert_eq!(taken_callbacks.len(), 2);
        assert_eq!(taken_callbacks[0].id, 1);
        assert_eq!(taken_callbacks[1].id, 2);
        assert!(surface.frame_callbacks.is_empty(), "Internal frame_callbacks list should be empty after take.");

        let taken_again = surface.take_frame_callbacks();
        assert!(taken_again.is_empty(), "Taking callbacks again should yield an empty list.");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceId(u64);

impl SurfaceId {
    // Basic ID generation, can be improved later
    pub fn new_unique() -> Self { // Made public for potential external use if needed
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        SurfaceId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceState {
    Created,
    PendingBuffer,
    Committed,
    Rendering,
    Presented,
    Destroyed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceAttributes {
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub transform: Mat3x3,
    pub alpha: f32,
    pub buffer_scale: i32,
    pub buffer_transform: WlOutputTransform,
    pub buffer_offset: (i32, i32), // For wl_surface.attach dx, dy
}

impl Default for SurfaceAttributes {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] // Added Eq, Hash for potential use in sets
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    pub fn area(&self) -> i32 {
        if self.is_empty() {
            0
        } else {
            self.width * self.height
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

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

    pub fn intersection(&self, other: &Self) -> Self {
        if !self.intersects(other) {
            return Self { x: 0, y: 0, width: 0, height: 0 }; // Empty rectangle
        }

        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        Self { x: x1, y: y1, width: x2 - x1, height: y2 - y1 }
    }

    // Clip this rectangle against another (clipping_rect).
    // Returns a new rectangle that is the part of this rectangle within clipping_rect.
    pub fn clipped_to(&self, clipping_rect: &Self) -> Self {
        self.intersection(clipping_rect)
    }

    // Translates the rectangle by (dx, dy)
    pub fn translate(&self, dx: i32, dy: i32) -> Self {
        Self { x: self.x + dx, y: self.y + dy, width: self.width, height: self.height }
    }

    // Scales the rectangle.
    pub fn scale(&self, factor: i32) -> Self {
        if factor == 0 { return Self::new(0,0,0,0); }
        if factor == 1 { return *self; } // No change
        Self {
            x: self.x / factor, // Integer division, as per wl_surface rules for buffer_scale
            y: self.y / factor,
            width: self.width / factor,
            height: self.height / factor,
        }
    }

    // Applies wl_output transform to a point.
    // `surface_width` and `surface_height` are the dimensions of the surface *before* this transform is applied,
    // but *after* scaling.
    fn transform_point(x: i32, y: i32, transform: WlOutputTransform, s_width: i32, s_height: i32) -> (i32, i32) {
        match transform {
            WlOutputTransform::Normal => (x, y),
            WlOutputTransform::Rotated90 => (s_height - y - 1, x), // (y, surface_width - x -1) but map to new coordinate space origin
                                                                // More standard: (y, old_sw - x -1) then map to new origin.
                                                                // Let's use a simpler mapping: (old_y, new_sw - old_x -1) -> (y, s_width - x -1)
                                                                // No, Wayland spec: (surface_height - y_buffer - 1, x_buffer) for 90 deg
                                                                // This implies coordinates are flipped then translated.
                                                                // For rects, it's more complex.
                                                                // Simpler: x' = y_buffer, y' = surface_width_untransformed - (x_buffer + width_buffer)
                                                                // For points: x_surf = y_buff, y_surf = s_w - x_buff -1 (if s_w is original width)
                                                                // This is complex. Sticking to rect transform for now.
            // TODO: Implement other transforms for points if necessary for advanced rect transformation.
            _ => (x,y) // Fallback for unimplemented transforms
        }
    }


    // Transforms the rectangle according to wl_output::Transform.
    // `s_width` and `s_height` are the surface dimensions *before* this transform is applied (i.e. buffer_w/scale, buffer_h/scale)
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
            WlOutputTransform::Flipped => { // Flipped horizontally
                new_x = s_width - (self.x + self.width);
            }
            WlOutputTransform::FlippedRotated90 => { // Flipped horizontally, then Rotated90
                // Original: (x, y, w, h)
                // Flipped: (s_w - (x+w), y, w, h)
                // Rotated90 from Flipped:
                // new_x = y_flipped = y
                // new_y = s_width_of_flipped_surface - (x_flipped + w_flipped)
                //       = s_width - ( (s_width - (x+w)) + w) = s_width - (s_width - x) = x
                new_x = self.y; // y from original
                new_y = s_width - ( (s_width - (self.x + self.width)) + self.width); // This is s_width - (s_width - self.x) = self.x
                new_width = self.height;
                new_height = self.width;
            }
            WlOutputTransform::FlippedRotated180 => { // Flipped horizontally, then Rotated180 (Vertical Flip)
                new_x = self.x; // x is flipped, then flipped back by 180 rot.
                new_y = s_height - (self.y + self.height); // y is not changed by horiz_flip, then flipped by 180 rot.
            }
            WlOutputTransform::FlippedRotated270 => { // Flipped horizontally, then Rotated270
                // Flipped: (s_w - (x+w), y, w, h)
                // Rotated270 from Flipped:
                // new_x = s_height_of_flipped_surface - (y_flipped + h_flipped)
                //       = s_height - (y + h)
                // new_y = x_flipped = s_w - (x+w)
                new_x = s_height - (self.y + self.height);
                new_y = s_width - (self.x + self.width);
                new_width = self.height;
                new_height = self.width;
            }
        }
        Self::new(new_x, new_y, new_width, new_height)
    }
}

const MAX_DAMAGE_RECTS: usize = 100; // Example limit

#[derive(Debug, Clone)]
pub struct DamageTracker {
    pending_damage_buffer: Vec<Rectangle>,  // Damage in buffer coordinates
    pending_damage_surface: Vec<Rectangle>, // Damage in surface-local coordinates (legacy)
    current_damage: Vec<Rectangle>,         // Committed damage in surface-local coordinates, clipped to surface
    damage_age: u32,                        // Counter for buffer age optimization
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DamageTracker {
    pub fn new() -> Self {
        Self {
            pending_damage_buffer: Vec::new(),
            pending_damage_surface: Vec::new(),
            current_damage: Vec::new(),
            damage_age: 0,
        }
    }

    // Helper to merge overlapping or adjacent rectangles
    fn merge_rectangles(rects: &mut Vec<Rectangle>) {
        if rects.len() <= 1 {
            return;
        }
        let mut i = 0;
        while i < rects.len() {
            let mut j = i + 1;
            let mut merged_current = false;
            while j < rects.len() {
                // A more robust check: if their union has the same area as sum of areas minus intersection area
                // or simply, if union is not much larger than individual ones.
                // For now, simple intersection or exact adjacency.
                let r1 = rects[i];
                let r2 = rects[j];
                if r1.intersects(&r2) ||
                   (r1.x == r2.x + r2.width && r1.y == r2.y && r1.height == r2.height && r1.width > 0 && r2.width > 0) || // r2 is left of r1
                   (r1.x + r1.width == r2.x && r1.y == r2.y && r1.height == r2.height && r1.width > 0 && r2.width > 0) || // r2 is right of r1
                   (r1.y == r2.y + r2.height && r1.x == r2.x && r1.width == r2.width && r1.height > 0 && r2.height > 0) || // r2 is above r1
                   (r1.y + r1.height == r2.y && r1.x == r2.x && r1.width == r2.width && r1.height > 0 && r2.height > 0)    // r2 is below r1
                {
                    rects[i] = r1.union(&r2);
                    rects.remove(j);
                    merged_current = true;
                    // No j increment here, as rects[j] is now a new rect
                } else {
                    j += 1;
                }
            }
            if merged_current {
                // Restart scan for rects[i] as it has changed and might merge with earlier rects if list was unsorted
                // For simplicity assuming this pass is enough or list is somewhat sorted.
                // To be fully robust, this merge might need multiple passes or a more complex algorithm.
                // Or, simply restart i if a merge happened:
                // i = 0; continue; // but this could be slow if many merges.
                // Let's stick to one pass for now.
            }
            i += 1;
        }
    }

    pub fn add_damage_buffer(&mut self, rect: Rectangle) {
        if rect.is_empty() { return; }
        self.pending_damage_buffer.push(rect);
        //DamageTracker::merge_rectangles(&mut self.pending_damage_buffer); // Merge during commit_pending_damage
        self.damage_age += 1;
    }

    pub fn add_damage_surface(&mut self, rect: Rectangle) {
        if rect.is_empty() { return; }
        self.pending_damage_surface.push(rect);
        //DamageTracker::merge_rectangles(&mut self.pending_damage_surface); // Merge during commit_pending_damage
        self.damage_age += 1;
    }

    fn transform_and_clip_damage_list(
        damage_list: &mut Vec<Rectangle>,
        attributes: &SurfaceAttributes,
        is_buffer_damage: bool,
        surface_boundary: &Rectangle,
    ) {
        let mut transformed_damage: Vec<Rectangle> = Vec::new();
        let (buffer_width, buffer_height) = if let Some(buffer) = attributes.size /* This is wrong, needs actual buffer dimensions for transform */ {
            // This 'size' in attributes is already scaled surface size.
            // We need original buffer dimensions for correct transform origin.
            // This part of the logic is tricky. For now, assume attributes.size IS the buffer dimension / scale.
            // This assumption holds if surface size is always derived from buffer.
            (attributes.size.0 as i32, attributes.size.1 as i32)
        } else {
            (0,0) // Should not happen if buffer is attached
        };


        for rect in damage_list.iter() {
            if rect.is_empty() { continue; }

            let transformed_rect = if is_buffer_damage {
                if attributes.buffer_scale <= 0 { Rectangle::new(0,0,0,0) } else {
                    // 1. Scale
                    let scaled_rect = rect.scale(attributes.buffer_scale);
                    // 2. Transform
                    // The `s_width` and `s_height` for `rect.transform` should be the dimensions of the
                    // surface in its "untransformed" state, i.e., after scaling but before wl_output_transform.
                    // This is attributes.size.
                    scaled_rect.transform(attributes.buffer_transform, surface_boundary.width, surface_boundary.height)
                }
            } else {
                *rect // Already in surface coordinates
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


    pub fn commit_pending_damage(&mut self, attributes: &SurfaceAttributes, new_buffer_committed: bool) {
        let surface_boundary = Rectangle::new(0, 0, attributes.size.0 as i32, attributes.size.1 as i32);
        if surface_boundary.is_empty() && !(self.pending_damage_buffer.is_empty() && self.pending_damage_surface.is_empty()) {
            // If surface has no size, but damage is being committed, it usually means full damage to the future size.
            // Or, it's an error state. For now, clear damage if surface is sizeless.
            self.current_damage.clear();
            self.pending_damage_buffer.clear();
            self.pending_damage_surface.clear();
            self.damage_age = 0;
            return;
        }


        // Transform pending_damage_buffer to surface coordinates
        DamageTracker::transform_and_clip_damage_list(&mut self.pending_damage_buffer, attributes, true, &surface_boundary);

        // Clip pending_damage_surface (already in surface coordinates)
        DamageTracker::transform_and_clip_damage_list(&mut self.pending_damage_surface, attributes, false, &surface_boundary);

        // Combine damage
        let mut combined_damage: Vec<Rectangle> = self.pending_damage_buffer.drain(..).collect();
        combined_damage.extend(self.pending_damage_surface.drain(..));

        // Merge all collected damage rectangles
        DamageTracker::merge_rectangles(&mut combined_damage);

        // Handle damage region overflow
        let total_surface_area = surface_boundary.area();
        let mut current_total_damage_area = 0;
        for rect in &combined_damage {
            current_total_damage_area += rect.area();
        }

        if combined_damage.len() > MAX_DAMAGE_RECTS ||
           (total_surface_area > 0 && combined_damage.len() > 1 && current_total_damage_area as f32 / total_surface_area as f32 > 0.75) { // Avoid full damage if only one rect
            self.current_damage = vec![surface_boundary];
        } else {
            self.current_damage = combined_damage;
        }

        if new_buffer_committed {
            self.damage_age = 0;
        }
        // If no new buffer, damage_age continues from add_damage calls.
    }

    pub fn get_current_damage_clipped(&self, surface_size: (u32, u32)) -> Vec<Rectangle> {
        let surface_boundary = Rectangle::new(0, 0, surface_size.0 as i32, surface_size.1 as i32);
        if surface_boundary.is_empty() { return Vec::new(); }
        self.current_damage.iter()
            .map(|rect| rect.clipped_to(&surface_boundary))
            .filter(|rect| !rect.is_empty())
            .collect()
    }

    pub fn clear_all_damage(&mut self) {
        self.pending_damage_buffer.clear();
        self.pending_damage_surface.clear();
        self.current_damage.clear();
        self.damage_age = 0;
    }
}

use crate::subcompositor::SubsurfaceState; // Import SubsurfaceState

// Placeholder for SurfaceRole
#[derive(Debug, Clone)] // Removed Copy, PartialEq, Eq as SubsurfaceState is not Copy/Eq by default
pub enum SurfaceRole {
    Toplevel, // Placeholder for actual Toplevel state struct
    Subsurface(SubsurfaceState),
    Cursor,   // Placeholder for actual Cursor state struct
    DragIcon, // Placeholder for actual DragIcon state struct
}

// BufferObject is now replaced by Arc<Mutex<BufferDetails>> from novade-buffer-manager

// Placeholder for WlCallback
#[derive(Debug, Clone)]
pub struct WlCallback {
    // Placeholder, actual implementation will involve Wayland objects
    pub id: u64,
}

// Placeholder for Region (similar to DamageTracker)
#[derive(Debug, Clone, Default)]
pub struct Region {
    pub rectangles: Vec<Rectangle>,
}

impl Region {
    pub fn new() -> Self {
        Self { rectangles: Vec::new() }
    }
}

pub struct Surface {
    pub id: SurfaceId,
    pub current_state: SurfaceState,
    pub pending_attributes: SurfaceAttributes,
    pub current_attributes: SurfaceAttributes,
    pub damage_tracker: DamageTracker,
    pub role: Option<SurfaceRole>,
    // pub attached_buffer: Option<BufferObject>, // Replaced
    pub pending_buffer: Option<Arc<Mutex<BufferDetails>>>,
    pub current_buffer: Option<Arc<Mutex<BufferDetails>>>,
    pub frame_callbacks: Vec<WlCallback>,
    pub opaque_region: Option<Region>,
    pub input_region: Option<Region>,

    // Subsurface fields
    pub children: Vec<SurfaceId>,   // List of direct child subsurfaces
    pub parent: Option<SurfaceId>,  // If this surface is a subsurface

    // Cache for synchronized subsurface state
    // These are populated when a synchronized subsurface commits.
    // They are applied when its parent commits (or when transitioning to desynchronized).
    cached_pending_buffer: Option<Arc<Mutex<BufferDetails>>>,
    cached_pending_attributes: Option<SurfaceAttributes>, // Using Option to clearly indicate if cache is populated
    // Damage is trickier to cache directly if DamageTracker processes it immediately.
    // For now, let's assume commit for sync subsurface populates these, and damage is part of attributes or handled separately.
    // The DamageTracker itself has pending_damage_buffer/surface.
    // When a sync subsurface commits, its pending damage is NOT yet moved to current_damage.
    // It's processed when apply_cached_state_from_sync is called.
}

impl Surface {
    pub fn new() -> Self { // ID is now generated internally
        Self {
            id: SurfaceId::new_unique(),
            current_state: SurfaceState::Created,
            pending_attributes: SurfaceAttributes::default(),
            current_attributes: SurfaceAttributes::default(),
            damage_tracker: DamageTracker::new(),
            role: None,
            // attached_buffer: None, // Replaced
            pending_buffer: None,
            current_buffer: None,
            frame_callbacks: Vec::new(),
            opaque_region: Some(Region::new()), // Wayland spec: initially the whole surface
            input_region: Some(Region::new()),  // Wayland spec: initially the whole surface

            children: Vec::new(),
            parent: None,

            cached_pending_buffer: None,
            cached_pending_attributes: None,
        }
    }
}

#[derive(Debug)]
pub enum BufferAttachError {
    BufferNotFound,
    ClientMismatch, // If buffer is owned by a different client
    InvalidBufferSize, // If buffer dimensions are zero or otherwise invalid
    InvalidState, // If surface is in a state that doesn't allow attach (e.g., Destroyed)
    InvalidBufferOffset, // Added here for attach validation consistency
}

#[derive(Debug)]
pub enum CommitError {
    InvalidState, // Surface is in a state that doesn't allow commit (e.g., Destroyed)
    InvalidBufferSize, // e.g. zero dimension, or not divisible by scale
    // InvalidBufferOffset is primarily an attach error, but commit might re-validate if needed.
    // For now, primary validation in attach.
}

// WlCallback struct is defined above (near SurfaceRole)

#[derive(Debug)]
pub enum SurfaceTransitionError {
    InvalidTransition,
    // Add other error types as needed
}

impl Surface {
    // Helper to release the current buffer, called on commit of new buffer or surface destruction
    fn release_current_buffer(&mut self, buffer_manager: &mut BufferManager) {
        if let Some(buffer_arc) = self.current_buffer.take() {
            let buffer_id = buffer_arc.lock().unwrap().id;
            buffer_manager.release_buffer(buffer_id);
        }
    }

    // Helper to release a pending buffer if a new one is attached or surface is cleared/destroyed
    fn release_pending_buffer(&mut self, buffer_manager: &mut BufferManager) {
        if let Some(buffer_arc) = self.pending_buffer.take() {
            let buffer_id = buffer_arc.lock().unwrap().id;
            buffer_manager.release_buffer(buffer_id);
        }
    }

    pub fn damage_buffer(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.current_state == SurfaceState::Destroyed { return; }
        // Wayland spec: width and height must be positive.
        // TODO: Consider sending a wl_surface protocol error if width/height are non-positive.
        let rect = Rectangle::new(x, y, width, height);
        if rect.is_empty() { return; }

        let mut opt_buffer_dims: Option<(i32,i32)> = None;
        if let Some(pending_ref) = &self.pending_buffer {
             let details = pending_ref.lock().unwrap();
             opt_buffer_dims = Some((details.width as i32, details.height as i32));
        } else if let Some(current_ref) = &self.current_buffer {
            let details = current_ref.lock().unwrap();
            opt_buffer_dims = Some((details.width as i32, details.height as i32));
        }

        if let Some(buffer_dims) = opt_buffer_dims {
            if buffer_dims.0 > 0 && buffer_dims.1 > 0 {
                let buffer_bounds = Rectangle::new(0, 0, buffer_dims.0, buffer_dims.1);
                let clipped_rect = rect.clipped_to(&buffer_bounds);
                if !clipped_rect.is_empty() {
                    self.damage_tracker.add_damage_buffer(clipped_rect);
                }
            }
        }
    }

    pub fn damage_surface(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.current_state == SurfaceState::Destroyed { return; }
        // Wayland spec: width and height must be positive.
        // TODO: Consider sending a wl_surface protocol error if width/height are non-positive.
        let rect = Rectangle::new(x, y, width, height);
        if rect.is_empty() { return; }

        let surface_bounds = Rectangle::new(0, 0, self.current_attributes.size.0 as i32, self.current_attributes.size.1 as i32);
         if surface_bounds.is_empty() && (self.current_attributes.size.0 !=0 || self.current_attributes.size.1 !=0) {
            return;
        }
        let clipped_rect = rect.clipped_to(&surface_bounds);
        if !clipped_rect.is_empty() {
            self.damage_tracker.add_damage_surface(clipped_rect);
        }
    }

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
        if x_offset != 0 || y_offset != 0 { // Assuming version >= 5
            return Err(BufferAttachError::InvalidBufferOffset);
        }
        self.release_pending_buffer(buffer_manager);
        self.pending_buffer = None;
        if let Some(buffer_arc) = buffer_arc_opt {
            {
                let buffer_details = buffer_arc.lock().unwrap();
                if let Some(owner_id) = buffer_details.client_owner_id {
                    if owner_id != client_id { return Err(BufferAttachError::ClientMismatch); }
                }
                if buffer_details.width == 0 || buffer_details.height == 0 {
                    return Err(BufferAttachError::InvalidBufferSize);
                }
            }
            buffer_arc.lock().unwrap().increment_ref_count();
            self.pending_buffer = Some(buffer_arc.clone());
            let (buffer_width, buffer_height) = {
                let details = buffer_arc.lock().unwrap();
                (details.width, details.height)
            };
            self.pending_attributes.size = (buffer_width, buffer_height);
        } else {
            self.pending_buffer = None;
        }
        self.pending_attributes.buffer_offset = (x_offset, y_offset);
        if self.current_state != SurfaceState::PendingBuffer {
            let _ = self.transition_to(SurfaceState::PendingBuffer);
        }
        Ok(())
    }

    pub fn commit(&mut self, buffer_manager: &mut BufferManager) -> Result<(), CommitError> {
        if self.current_state == SurfaceState::Destroyed {
            return Err(CommitError::InvalidState);
        }
        if !matches!(self.current_state, SurfaceState::Created | SurfaceState::PendingBuffer | SurfaceState::Committed) {
            return Err(CommitError::InvalidState);
        }

        // --- Validation Phase ---
        if let Some(pending_buffer_arc) = &self.pending_buffer {
            let pending_buffer_details = pending_buffer_arc.lock().unwrap();
            if pending_buffer_details.width == 0 || pending_buffer_details.height == 0 {
                return Err(CommitError::InvalidBufferSize);
            }
            let scale = self.pending_attributes.buffer_scale;
            if scale <= 0 {
                return Err(CommitError::InvalidBufferSize);
            }
            if pending_buffer_details.width % (scale as u32) != 0 ||
               pending_buffer_details.height % (scale as u32) != 0 {
                return Err(CommitError::InvalidBufferSize);
            }
        }

        // --- Synchronized Subsurface Handling ---
        let mut is_currently_synchronized_subsurface = false;
        if let Some(SurfaceRole::Subsurface(ref mut subsurface_state)) = self.role {
            if subsurface_state.sync_mode == SubsurfaceSyncMode::Synchronized {
                is_currently_synchronized_subsurface = true;
                subsurface_state.needs_apply_on_parent_commit = true;

                if let Some(pending_buffer_arc_real) = &self.pending_buffer {
                    pending_buffer_arc_real.lock().unwrap().increment_ref_count();
                    self.cached_pending_buffer = Some(pending_buffer_arc_real.clone());
                } else {
                    self.cached_pending_buffer = None;
                }
                self.cached_pending_attributes = Some(self.pending_attributes);
                self.pending_buffer.take();

                let _ = self.transition_to(SurfaceState::Committed);
                return Ok(());
            }
        }

        // --- Regular Commit Logic (for Toplevels or Desynchronized Subsurfaces) ---
        let mut new_content_committed = false;
        if self.current_state == SurfaceState::PendingBuffer {
            new_content_committed = true;
        }

        self.current_attributes = self.pending_attributes;

        if new_content_committed {
            self.release_current_buffer(buffer_manager);
            self.current_buffer = self.pending_buffer.take();
        }

        let mut applied_cache_for_desync = false;
        if !is_currently_synchronized_subsurface {
            if let Some(SurfaceRole::Subsurface(ref mut subsurface_state)) = self.role {
                 if subsurface_state.sync_mode == SubsurfaceSyncMode::Desynchronized &&
                    subsurface_state.needs_apply_on_parent_commit {
                     self.apply_cached_state_from_sync(buffer_manager)?;
                     applied_cache_for_desync = true;
                 }
            }
        }

        if !applied_cache_for_desync {
            self.damage_tracker.commit_pending_damage(&self.current_attributes, new_content_committed);
        }

        let _ = self.transition_to(SurfaceState::Committed);
        Ok(())
    }

    pub fn apply_cached_state_from_sync(&mut self, buffer_manager: &mut BufferManager) -> Result<(), CommitError> {
        if self.current_state == SurfaceState::Destroyed {
            return Err(CommitError::InvalidState);
        }

        let mut needs_apply_flag_was_set = false;
        if let Some(SurfaceRole::Subsurface(ref mut subsurface_state)) = self.role {
            if subsurface_state.needs_apply_on_parent_commit {
                needs_apply_flag_was_set = true;
                subsurface_state.needs_apply_on_parent_commit = false;
            } else {
                return Ok(());
            }
        } else {
            return Err(CommitError::InvalidState);
        }

        if !needs_apply_flag_was_set {
            return Ok(());
        }

        if let Some(cached_attrs) = self.cached_pending_attributes.take() {
            self.current_attributes = cached_attrs;
        }

        self.release_current_buffer(buffer_manager);
        self.current_buffer = self.cached_pending_buffer.take();

        let new_content_was_cached = self.cached_pending_attributes.is_some();

        self.damage_tracker.commit_pending_damage(&self.current_attributes, new_content_was_cached);
        Ok(())
    }

    pub fn prepare_for_destruction(
        &mut self,
        buffer_manager: &mut BufferManager,
        surface_registry_accessor: &impl surface_registry::SurfaceRegistryAccessor,
    ) {
        if let Some(buffer_arc) = self.pending_buffer.take() {
            buffer_manager.release_buffer(buffer_arc.lock().unwrap().id);
        }
        if let Some(buffer_arc) = self.current_buffer.take() {
            buffer_manager.release_buffer(buffer_arc.lock().unwrap().id);
        }
        if let Some(cached_buffer_arc) = self.cached_pending_buffer.take() {
             buffer_manager.release_buffer(cached_buffer_arc.lock().unwrap().id);
        }


        if let Some(parent_id) = self.parent.take() {
            if let Some(parent_surface_arc) = surface_registry_accessor.get_surface(parent_id) {
                if let Ok(mut parent_surface) = parent_surface_arc.lock() {
                    parent_surface.children.retain(|child_id| *child_id != self.id);
                }
            }
        }

        let children_to_unmap = std::mem::take(&mut self.children);
        for child_id in children_to_unmap {
            if let Some(child_surface_arc) = surface_registry_accessor.get_surface(child_id) {
                if let Ok(mut child_surface) = child_surface_arc.lock() {
                    if let Some(SurfaceRole::Subsurface(ref mut sub_state)) = child_surface.role {
                        // Mark as orphaned: parent_id could be set to a sentinel, or role cleared.
                        // For now, just clearing parent link. The child is now effectively unmapped.
                        child_surface.parent = None;
                        sub_state.parent_id = SurfaceId::new_unique(); // Or some other invalid ID
                                                                       // To prevent it from thinking it's still attached.
                        // Actual recursive destruction/unregistration will be handled by the main loop
                        // in SurfaceRegistry::unregister_surface.
                    }
                }
            }
        }

        self.frame_callbacks.clear();
        self.damage_tracker.clear_all_damage();
        self.current_state = SurfaceState::Destroyed;
    }


    pub fn frame(&mut self, callback_id: u64) {

pub mod surface_registry {
    use super::{Surface, SurfaceId, SurfaceState, SurfaceRole};
    use crate::buffer_manager::BufferManager; // Needed for unregister_surface
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Trait for accessing surfaces from the registry, used to decouple
    /// `Surface::prepare_for_destruction` from needing a direct `&mut SurfaceRegistry`,
    /// which can cause borrowing issues if `SurfaceRegistry::unregister_surface` calls it.
    pub trait SurfaceRegistryAccessor {
        fn get_surface(&self, id: SurfaceId) -> Option<Arc<Mutex<Surface>>>;
        // fn unregister_child_from_map(&mut self, id: SurfaceId) -> Option<Arc<Mutex<Surface>>>;
    }

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
        pub fn new() -> Self {
            Self {
                surfaces: HashMap::new(),
            }
        }

        pub fn register_new_surface(&mut self) -> (SurfaceId, Arc<Mutex<Surface>>) {
            // Note on no_memory for Surface allocation:
            // Rust's direct allocation (like Arc::new(Mutex::new(surface))) panics on OOM.
            // A robust Wayland compositor would use fallible allocation (e.g., Box::try_new)
            // and if it fails, propagate the error upwards to ultimately send wl_display.error
            // with the `no_memory` code to the client.
            let surface = Surface::new();
            let id = surface.id;
            let arc_surface = Arc::new(Mutex::new(surface));
            self.surfaces.insert(id, arc_surface.clone());
            (id, arc_surface)
        }

        pub fn register_surface_arc(&mut self, surface_arc: Arc<Mutex<Surface>>) -> SurfaceId {
            let id = surface_arc.lock().unwrap().id;
            self.surfaces.insert(id, surface_arc);
            id
        }

        pub fn get_surface(&self, id: SurfaceId) -> Option<Arc<Mutex<Surface>>> {
            self.surfaces.get(&id).cloned()
        }

        pub fn unregister_surface(
            &mut self,
            surface_id: SurfaceId,
            buffer_manager: &mut BufferManager,
        ) -> Option<Arc<Mutex<Surface>>> {
            if let Some(surface_arc) = self.surfaces.get(&surface_id).cloned() {
                // Collect children IDs before locking the current surface for destruction.
                let children_ids: Vec<SurfaceId> = { // Explicit scope for lock
                    let surface_guard = surface_arc.lock().unwrap();
                    surface_guard.children.clone()
                };

                // Call prepare_for_destruction on the current surface.
                // This will release its buffers, disassociate from its parent,
                // and update its children's parent links.
                { // Explicit scope for lock
                    let mut surface_guard = surface_arc.lock().unwrap();
                    if surface_guard.current_state != SurfaceState::Destroyed {
                        surface_guard.prepare_for_destruction(buffer_manager, self);
                    }
                }

                // Now, recursively unregister all children identified before destruction.
                // This ensures that `prepare_for_destruction` for each child is called,
                // which handles buffer releases and further parent/child list modifications.
                for child_id in children_ids {
                    self.unregister_surface(child_id, buffer_manager);
                }

                // Finally, remove the target surface from the registry.
                self.surfaces.remove(&surface_id)
            } else {
                None // Surface not found
            }
        }
    }
}
