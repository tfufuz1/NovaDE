pub mod session_interface;
pub use session_interface::LibinputSessionManager;

use smithay::backend::input::LibinputInputBackend;
use crate::input::errors::InputError;
use std::rc::Rc;
use std::cell::RefCell;
use smithay::reexports::calloop::{LoopHandle, RegistrationToken};
use crate::compositor::core::state::DesktopState; // Pfad anpassen
use crate::input::event_dispatcher; // Added

pub fn init_libinput_backend(
    session_interface: Rc<RefCell<dyn smithay::backend::input::LibinputInterface>>
) -> Result<LibinputInputBackend, InputError> {
    tracing::info!("Initializing Libinput-Backend...");

    // Create a new libinput context.
    // The LibinputInterface is used by libinput to open and close devices.
    let mut libinput_context = match libinput::Libinput::new_from_path(session_interface) {
        Ok(ctx) => ctx,
        Err(e) => {
            let err_msg = format!("Failed to create Libinput context from path: {:?}", e);
            tracing::error!("{}", err_msg);
            // It's not an io::Error directly, so map to a suitable InputError variant.
            // Assuming InputError::LibinputError can take a String.
            return Err(InputError::LibinputError(format!("Context creation failed: {}", err_msg)));
        }
    };

    // Assign the libinput context to a specific udev seat. "seat0" is standard.
    // This tells libinput which set of devices to manage.
    if let Err(e) = libinput_context.udev_assign_seat("seat0") {
        let err_msg = format!("Failed to assign libinput context to udev seat 'seat0': {:?}", e);
        tracing::error!("{}", err_msg);
        return Err(InputError::LibinputError(format!("Seat assignment failed: {}", err_msg)));
    }

    // Wrap the libinput context in Smithay's LibinputInputBackend.
    // This backend adapter will translate libinput events into Smithay's event system.
    // The second argument can be used for logging; here, we pass the current tracing span.
    let libinput_backend = LibinputInputBackend::new(libinput_context, Some(tracing::Span::current()));

    tracing::info!("Libinput-Backend successfully initialized.");
    Ok(libinput_backend)
}

pub fn register_libinput_source(
    loop_handle: &LoopHandle<'static, DesktopState>,
    libinput_backend: LibinputInputBackend,
    seat_name: String, // seat_name needs to be owned by the closure
) -> Result<RegistrationToken, InputError> {
    tracing::info!("Registering Libinput event source for seat '{}' in calloop event loop.", seat_name);

    match loop_handle.insert_source(libinput_backend, move |event, _metadata, desktop_state| {
        event_dispatcher::process_input_event(desktop_state, event, &seat_name);
    }) {
        Ok(token) => {
            tracing::info!("Libinput event source registered successfully for seat '{}'.", seat_name);
            Ok(token)
        }
        Err(e) => {
            let err_msg = format!("Failed to insert libinput event source for seat '{}': {}", seat_name, e);
            tracing::error!("{}", err_msg);
            Err(InputError::EventSourceSetupError(err_msg))
        }
    }
}
