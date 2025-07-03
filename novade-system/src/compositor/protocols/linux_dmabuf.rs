// novade-system/src/compositor/protocols/linux_dmabuf.rs
// Implementation of the zwp_linux_dmabuf_v1 Wayland protocol

use smithay::{
    delegate_dmabuf,
    reexports::{
        wayland_protocols::wp::linux_dmabuf::zv1::server::{
            zwp_linux_dmabuf_feedback_v1::{self, ZwpLinuxDmabufFeedbackV1},
            zwp_linux_dmabuf_params_v1::{self, ZwpLinuxDmabufParamsV1, Request as ParamsRequest, Error as ParamsError},
            zwp_linux_dmabuf_v1::{self, ZwpLinuxDmabufV1, Request as DmabufRequest, Event as DmabufEvent},
        },
        wayland_server::{
            protocol::wl_buffer, // The result of a successful import is a wl_buffer
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData, Backend,
        },
        calloop::LoopHandle,
        drm::control::Device as DrmDevice, // For interacting with DRM device if needed for main device
    },
    utils::{Buffer as SmithayBuffer, Physical, Size, Serial}, // SmithayBuffer can wrap DMABufs
    backend::{
        allocator::{
            dmabuf::{Dmabuf, DmabufAllocator, DmabufFlags, DmabufHandle, MAX_PLANES},
            Allocator, Buffer, Format, Fourcc, Modifier,
        },
        drm::DrmNode, // For selecting the primary DRM node
        renderer::{
            gles::GlesRenderer, // Example renderer that can import DMABufs
            // ImportDma, Renderer, // Generic traits
            // Bind, ExportMem, // Traits related to buffer handling in renderers
            // ImportMem, // For importing into renderer
        },
    },
    wayland::dmabuf::{
        DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier, DmabufFeedback, DmabufFeedbackBuilder,
        DmabufParamsData, // UserData for zwp_linux_dmabuf_params_v1
    },
    // If using GlesRenderer directly for format checking:
    // backend::renderer::gles::ffi,
};
use std::{
    sync::{Arc, Mutex},
    collections::HashSet,
    os::unix::io::OwnedFd,
};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `DmabufState` and provide access to the renderer and allocator.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For DMABUF, it would need to manage or access:
    // - DmabufState
    // - Renderer (to check supported formats/modifiers and import buffers)
    // - Primary DRM node (for feedback)
    // - DmabufAllocator (if used for compositor's own allocations)
}

#[derive(Debug, Error)]
pub enum DmabufError {
    #[error("Failed to create DMABUF buffer: {0}")]
    BufferCreationFailed(String),
    #[error("Unsupported DMABUF format or modifier: FourCC {fourcc:?}, Modifier {modifier:?}")]
    UnsupportedFormatModifier { fourcc: Fourcc, modifier: Modifier },
    #[error("Invalid DMABUF parameters: {0}")]
    InvalidParameters(String),
    #[error("Renderer failed to import DMABUF: {0}")]
    ImportFailed(String),
    #[error("DRM node error: {0}")]
    DrmNodeError(String),
}

// The main compositor state (e.g., NovaCompositorState) would implement DmabufHandler
// and store DmabufState, DmabufGlobal, and provide access to the GlesRenderer (or other).
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub dmabuf_state: DmabufState,
//     pub dmabuf_global: DmabufGlobal, // Stores supported formats/modifiers
//     pub renderer: GlesRenderer, // Or other renderer implementing ImportDma
//     pub primary_drm_node: DrmNode, // Main DRM node for feedback
//     ...
// }
//
// impl DmabufHandler for NovaCompositorState {
//     fn dmabuf_state(&mut self) -> &mut DmabufState {
//         &mut self.dmabuf_state
//     }
//
//     fn dmabuf_imported(&mut self, global: &DmabufGlobal, dmabuf: Dmabuf, notifier: ImportNotifier) {
//         // This is the crucial callback after a buffer is created by params.create_immed or params.create.
//         // We need to import it into the renderer.
//         let result = self.renderer.import_dmabuf(&dmabuf, None); // Or with damage tracking
//         match result {
//             Ok(render_buffer) => {
//                 // `render_buffer` is now a type usable by the renderer (e.g., GlesTexture).
//                 // We need to wrap it in a `wl_buffer`.
//                 // Smithay provides `ExternalDmaBuffer` or similar mechanisms, or we might need
//                 // to create a custom `Buffer` impl that holds `render_buffer`.
//                 // For now, let's assume `Dmabuf` itself can be made into a `SmithayBuffer`.
//                 let smithay_buffer = SmithayBuffer::from(dmabuf); // This might need custom wrapping if renderer owns it.
//                 notifier.successful(smithay_buffer.into()); // Convert to wl_buffer
//             }
//             Err(e) => {
//                 error!("Failed to import DMABUF into renderer: {}", e);
//                 notifier.failed();
//             }
//         }
//     }
// }
// delegate_dmabuf!(NovaCompositorState);


impl DmabufHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        // TODO: Properly integrate DmabufState with DesktopState or NovaCompositorState.
        panic!("DmabufHandler::dmabuf_state() needs proper integration. DesktopState must own DmabufState.");
        // Example: &mut self.nova_compositor_state.dmabuf_state
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal, // The DmabufGlobal that handled this import
        dmabuf: Dmabuf,         // The Dmabuf containing the FDs and layout
        notifier: ImportNotifier, // Used to signal success (with wl_buffer) or failure
    ) {
        info!("DMABUF imported by client, attempting to import into renderer. Dmabuf: {:?}", dmabuf);
        // This is the core of the DMABUF import process on the compositor side.
        // A client has called `zwp_linux_buffer_params_v1.create` or `create_immed`.
        // Smithay has validated the parameters against the `DmabufGlobal`'s advertised formats/modifiers
        // and created a `Dmabuf` object.
        // Now, we (the compositor) must try to import this `Dmabuf` into our renderer.

        // TODO: Access the actual renderer from `self` (DesktopState or NovaCompositorState)
        // let renderer = &mut self.renderer; // e.g., self.nova_compositor_state.renderer

        // --- Placeholder for renderer import ---
        // This section needs to be replaced with actual renderer interaction.
        // For example, if using Smithay's GlesRenderer:
        /*
        match renderer.import_dmabuf(&dmabuf, None) { // None for damage, provide if available
            Ok(gles_texture_or_similar_render_buffer) => {
                info!("Successfully imported DMABUF into GlesRenderer: {:?}", gles_texture_or_similar_render_buffer);

                // The `gles_texture_or_similar_render_buffer` is now a GPU resource.
                // We need to create a `wl_buffer` resource that represents this.
                // Smithay's `Dmabuf` itself can be wrapped into a `SmithayBuffer` if the renderer
                // doesn't take ownership or provides a way to associate it.
                // More commonly, the renderer's buffer type (GlesTexture) would be wrapped.

                // Smithay requires the `ImportNotifier::successful()` to be called with a `wl_buffer`.
                // This `wl_buffer` needs to be backed by our imported DMABUF.
                //
                // One way is to have a custom `Buffer` implementation that wraps the renderer's buffer type
                // and also holds onto the original `Dmabuf` for metadata if needed.
                //
                // Smithay's `Dmabuf` can be turned into a `wl_buffer` if it's properly
                // associated with the renderer's imported resource.
                // The `Dmabuf` object itself is mostly a descriptor of FDs and layout.
                // The actual import happens in the renderer.

                // Let's assume the renderer gives us back something that can be turned into a SmithayBuffer
                // which then can be converted to a wl_buffer for the notifier.
                // This is a complex part depending on renderer specifics.
                //
                // A common pattern:
                // 1. Renderer imports Dmabuf, returns a handle/texture.
                // 2. Create a struct that implements `smithay::backend::allocator::Buffer`
                //    (or uses an existing one like `smithay::backend::renderer::gles::GlesBuffer`)
                //    that wraps this handle/texture and the Dmabuf metadata.
                // 3. Pass this struct to `notifier.successful()`. Smithay will create the `wl_buffer`
                //    and associate our Buffer impl with it.

                // For this skeleton, we'll simulate success but acknowledge the complexity.
                // This is a critical integration point.
                // If `Dmabuf` itself is to be the `SmithayBuffer`:
                // This implies that when the renderer uses this `wl_buffer`, it can retrieve
                // the underlying `Dmabuf` (e.g., from UserData of wl_buffer) and then find its
                // imported GPU resource.

                // A more direct approach if the renderer doesn't need a complex wrapper:
                // The `Dmabuf` object itself might be sufficient if the renderer can re-import
                // it on demand using its FDs, or if the import process registers the Dmabuf globally.
                // However, `ImportNotifier::successful` takes a `wl_buffer::WlBuffer`.
                // Smithay's `Dmabuf::create_buffer` can create this `wl_buffer` *after* we confirm import.

                // The `ImportNotifier` is for the `zwp_linux_buffer_params_v1.create_immed` case.
                // For `create`, the wl_buffer is returned directly from the request.
                // Smithay's `DmabufHandler::dmabuf_imported` is for `create_immed`.
                // For `create` (non-immed), the wl_buffer is made by `params.create(...)` if successful.

                // Let's refine: `dmabuf_imported` is called by Smithay *after* it has constructed
                // the `Dmabuf` object from client parameters. Our job is to try and make it usable
                // by our renderer and then tell Smithay if it's a go.
                // If successful, Smithay will then create the `wl_buffer` and send 'created' or 'success'.

                // The `notifier` is of type `ImportNotifier`.
                // `notifier.successful()` does NOT take a wl_buffer. It signals success.
                // `notifier.failed()` signals failure.
                // If `create_immed` was used, Smithay sends `zwp_linux_buffer_params_v1.created` or `failed`.
                // The `wl_buffer` is the one associated with `zwp_linux_buffer_params_v1.created(new_id)`.
                // Smithay handles associating the `Dmabuf` object with that `new_id` (`wl_buffer`)
                // if we call `notifier.successful()`.

                // So, the flow is:
                // 1. Client calls create_immed(params_id, new_buffer_id, ...).
                // 2. Smithay validates, creates `Dmabuf` object.
                // 3. Smithay calls `dmabuf_imported(..., dmabuf, notifier)`.
                // 4. We try to import `dmabuf` into our renderer.
                //    - If OK: call `notifier.successful()`. Smithay then:
                //        - Associates `dmabuf` with `new_buffer_id`.
                //        - Sends `zwp_linux_buffer_params_v1.created(new_buffer_id)` to client.
                //    - If Fail: call `notifier.failed()`. Smithay then:
                //        - Sends `zwp_linux_buffer_params_v1.failed()` to client.

                // Placeholder for successful import:
                let import_success = true; // Simulate renderer import success

                if import_success {
                    info!("DMABUF successfully prepared for renderer use.");
                    notifier.successful();

                    // After this, the client will have a wl_buffer (new_buffer_id).
                    // When the client commits this wl_buffer to a wl_surface,
                    // `CompositorHandler::commit` will be called.
                    // Inside `commit`, we can get the `wl_buffer`, check if it's a DMABUF
                    // (e.g., `with_states(&surface, |states| { BufferDamageTracker::get_buffer(states) })`
                    // then `buffer.as_dmabuf()` or checking UserData).
                    // If it is, we then use our renderer to draw it.
                    // The renderer needs to be able to find its imported GPU resource from the Dmabuf.
                    // This usually means the renderer has its own map: Dmabuf (or its FDs) -> GPU resource.
                } else {
                    error!("Failed to import DMABUF into renderer (simulated failure).");
                    notifier.failed();
                }

        */
        // --- End Placeholder ---
        warn!("DmabufHandler::dmabuf_imported: Actual renderer import logic is needed here.");
        // Simulate failure until renderer is integrated to avoid issues with unusable buffers.
        notifier.failed();
    }
}

// Delegate DMABUF handling to DesktopState (or NovaCompositorState)
// delegate_dmabuf!(DesktopState); // Needs to be NovaCompositorState

/// Initializes and registers the Linux DMABUF globals.
/// `D` is your main compositor state type.
pub fn init_linux_dmabuf<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
    renderer: &mut GlesRenderer, // Example: pass renderer to get supported formats/modifiers
                                 // A more generic approach would be to pass an object that can list them.
    primary_drm_node: DrmNode,   // For default feedback
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<ZwpLinuxDmabufV1, ()> + Dispatch<ZwpLinuxDmabufV1, UserData, D> +
       Dispatch<ZwpLinuxDmabufParamsV1, DmabufParamsData, D> +
       Dispatch<ZwpLinuxDmabufFeedbackV1, UserData, D> + // UserData for feedback is often ()
       DmabufHandler + 'static,
       // D must also own DmabufState.
{
    info!("Initializing ZwpLinuxDmabufV1 global");

    // 1. Create DmabufState. This state needs to be managed by your compositor (in D).
    //    Example: state.dmabuf_state = DmabufState::new();

    // 2. Determine supported DMABUF formats and modifiers.
    //    This is highly dependent on your renderer and graphics hardware.
    //    Smithay's GlesRenderer can provide this information.
    let mut supported_formats = HashSet::<Format>::new();

    // Example for GlesRenderer:
    // Query EGL for supported formats and modifiers for external buffers.
    // This often involves checking EGL extensions like EGL_EXT_image_dma_buf_import and EGL_EXT_image_dma_buf_import_modifiers.
    // Smithay's GlesRenderer might have helper functions for this, or it might need direct EGL calls.
    //
    // For simplicity, let's add some common formats. This list MUST be accurate for your hardware/renderer.
    // Incorrectly advertising formats will lead to client errors or rendering issues.
    supported_formats.insert(Format { code: Fourcc::Abgr8888, modifier: Modifier::None });
    supported_formats.insert(Format { code: Fourcc::Xbgr8888, modifier: Modifier::None });
    supported_formats.insert(Format { code: Fourcc::Argb8888, modifier: Modifier::None });
    supported_formats.insert(Format { code: Fourcc::Xrgb8888, modifier: Modifier::None });
    // Add formats with explicit modifiers if supported (e.g., Modifier::Linear, vendor-specific modifiers)
    // Example:
    // if renderer.supports_modifier(Fourcc::Argb8888, Modifier::I915Linear) {
    //     supported_formats.insert(Format { code: Fourcc::Argb8888, modifier: Modifier::I915Linear });
    // }
    warn!("DMABUF supported formats are hardcoded placeholders. These MUST be queried from the renderer/driver.");


    // 3. Create DmabufGlobal: This stores the list of supported formats/modifiers.
    //    Clients will query this via zwp_linux_dmabuf_feedback_v1.
    let dmabuf_global = DmabufGlobal::new(supported_formats.clone()); // Clone if needed elsewhere

    // 4. Create the main zwp_linux_dmabuf_v1 global.
    //    This global is what clients bind to initiate DMABUF operations.
    display.create_global::<D, ZwpLinuxDmabufV1, _>(
        4, // protocol version of zwp-linux-dmabuf-unstable-v1 is 4
        dmabuf_global.clone() // Pass the DmabufGlobal as UserData for the ZwpLinuxDmabufV1 global itself
                              // This makes it accessible in Dispatch<ZwpLinuxDmabufV1>.
                              // Smithay's delegate_dmabuf expects DmabufGlobal to be in ZwpLinuxDmabufV1's UserData.
    )?;

    // 5. (Optional but Recommended) Create default zwp_linux_dmabuf_feedback_v1 object.
    //    This provides clients with initial format/modifier information without needing explicit params objects.
    //    The feedback object is associated with a DRM node (tranche target device).
    let main_device = primary_drm_node.dev_id().ok_or_else(|| DmabufError::DrmNodeError("Primary DRM node has no device ID".into()))?;
    // Other DRM nodes can also be advertised if they support different formats/modifiers (tranches).

    let feedback_builder = DmabufFeedbackBuilder::new(main_device, supported_formats)
        // .add_tranche(other_drm_node_dev_id, other_supported_formats) // If supporting multiple GPUs/nodes
        ;
    let default_feedback = feedback_builder.build_global::<D>(display)?; // D is the Compositor state
    info!("Created default DMABUF feedback global: {:?}", default_feedback);


    // Ensure `delegate_dmabuf!(D)` is called in your main compositor state setup.
    // This macro sets up the necessary Dispatch implementations for:
    // - ZwpLinuxDmabufV1 (using DmabufGlobal from its UserData)
    // - ZwpLinuxDmabufParamsV1 (using DmabufParamsData as its UserData, created by DmabufGlobal)
    // It relies on `D` implementing `DmabufHandler` and having `DmabufState`.

    info!("ZwpLinuxDmabufV1 global initialized with {} supported formats.", dmabuf_global.formats.len());
    Ok(())
}

// TODO:
// - Accurate Format/Modifier Detection:
//   - Replace hardcoded formats with actual queries to the renderer (EGL, Vulkan WSI extensions).
//   - This is CRITICAL for DMABUF to work correctly.
// - Renderer Integration:
//   - The `DmabufHandler::dmabuf_imported` method needs full implementation to import the
//     `Dmabuf` into the active renderer (e.g., GlesRenderer, Vulkan renderer).
//   - This involves obtaining a renderable resource (texture, image) from the DMABUF FDs.
//   - The renderer must then be able to use this resource when a `wl_buffer` backed by this DMABUF
//     is committed to a `wl_surface`.
// - State Integration:
//   - `DmabufState` and `DmabufGlobal` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `DmabufHandler`.
//   - `delegate_dmabuf!(NovaCompositorState);` macro must be used.
// - Feedback Tranches:
//   - If multiple GPUs or DRM nodes with different capabilities are present, advertise them
//     as separate tranches in `zwp_linux_dmabuf_feedback_v1`.
// - Testing:
//   - Client (e.g., Weston's dmabuf clients, GTK4, Qt6, games) allocating and sharing DMABUFs.
//   - Test with various formats (RGBA, BGRA, YUV if supported).
//   - Test with modifiers (linear, tiled).
//   - Performance testing (CPU usage, GPU usage, frame rates) to confirm zero-copy benefits.
//   - Error handling (e.g., client provides invalid DMABUF parameters).

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod linux_dmabuf;
