use super::object::{WaylandObject, ObjectId, ProtocolError, RequestContext}; // Changed
use super::wire::WlArgument; // Changed
use super::surface::Surface; // Changed
use super::subsurface::WlSubsurface; // Changed
use std::sync::Arc;

#[derive(Debug)]
pub struct WlSubcompositor {
    id: ObjectId,
    version: u32,
}

impl WlSubcompositor {
    pub fn new(id: ObjectId, version: u32) -> Self {
        Self { id, version }
    }
}

impl WaylandObject for WlSubcompositor {
    fn id(&self) -> ObjectId { self.id }
    fn version(&self) -> u32 { self.version }
    fn interface_name(&self) -> &'static str { "wl_subcompositor" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> { self }

    fn handle_request(
        &self,
        opcode: u16,
        args: Vec<WlArgument>,
        context: &mut RequestContext,
    ) -> Result<(), ProtocolError> {
        match opcode {
            0 => { // destroy
                // Standard object destruction. WlSubcompositor is usually global,
                // but clients can destroy their handle.
                context.object_manager.destroy_object(self.id);
                Ok(())
            }
            1 => { // get_subsurface(id: new_id, surface: object, parent: object)
                if args.len() < 3 { return Err(ProtocolError::InvalidArguments); }

                let new_subsurface_id = match args[0] {
                    WlArgument::NewId(id) => id,
                    _ => return Err(ProtocolError::InvalidArguments),
                };
                let surface_id = match args[1] {
                    WlArgument::Object(id) => id,
                    _ => return Err(ProtocolError::InvalidArguments),
                };
                let parent_surface_id = match args[2] {
                    WlArgument::Object(id) => id,
                    _ => return Err(ProtocolError::InvalidArguments),
                };

                if surface_id == parent_surface_id {
                    // TODO: Wayland protocol error SURFACE_SAME_AS_PARENT (custom error)
                    return Err(ProtocolError::InvalidArguments);
                }

                // Get the surface and parent surface objects
                let surface_obj: Arc<Surface> = context.object_manager.get_typed_object(surface_id)?;
                let parent_surface_obj: Arc<Surface> = context.object_manager.get_typed_object(parent_surface_id)?;

                // Create the WlSubsurface
                let subsurface = WlSubsurface::new(
                    new_subsurface_id,
                    self.version, // Subsurface inherits version from subcompositor
                    surface_obj.clone(),
                    parent_surface_obj.clone(),
                )?; // new can fail if role assignment fails

                // Register the new WlSubsurface object
                context.object_manager.register_new_object(new_subsurface_id, subsurface)?;

                // Link subsurface to parent (parent needs to store a reference/ID)
                parent_surface_obj.add_subsurface(new_subsurface_id)?; // Assuming new_subsurface_id is the ID of the WlSubsurface object

                Ok(())
            }
            _ => Err(ProtocolError::InvalidOpcode(opcode)),
        }
    }
}
