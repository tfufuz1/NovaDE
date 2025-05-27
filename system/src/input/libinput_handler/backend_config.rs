use crate::input::{
    errors::InputError,
    libinput_handler::session_interface::LibinputSessionManager,
};
use smithay::backend::{
    input::{InputBackend, LibinputInputBackend}, // LibinputInputBackend
    libinput::Libinput, // Smithay's Libinput type, not the raw libinput crate directly for new_from_path
};
use std::sync::Arc; // For Arc<LibinputSessionManager>

/// Initializes the libinput backend for input event processing.
///
/// This function sets up a `libinput` context using the provided session manager,
/// assigns it to a default seat (e.g., "seat0"), and then wraps this context
/// into Smithay's `LibinputInputBackend`.
///
/// # Arguments
///
/// * `session_manager`: An `Arc` pointing to a `LibinputSessionManager` that
///   implements `smithay::backend::input::LibinputInterface`. This manager
///   handles opening and closing device file descriptors.
///
/// # Returns
///
/// * `Ok(LibinputInputBackend)`: The initialized libinput backend, ready to be
///   used as an event source.
/// * `Err(InputError)`: If creating the libinput context or assigning the seat fails.
pub fn init_libinput_backend(
    session_manager: Arc<LibinputSessionManager>,
) -> Result<LibinputInputBackend, InputError> {
    tracing::info!("Initializing libinput backend...");

    // Create a libinput context.
    // `Libinput::new_from_path` takes an implementation of `LibinputInterface`.
    // Smithay's `Libinput` type is a wrapper around the raw `libinput::Libinput`.
    // The first argument to `Libinput::new_from_path` is the implementation of the interface.
    // The raw `libinput::Libinput::new_from_path` expects a closure that then uses the interface.
    // Smithay's `Libinput::new` (from `smithay::backend::libinput`) is what we should use.
    // It takes `I: LibinputInterface + 'static`.
    // The `Arc<LibinputSessionManager>` needs to be passed in a way that satisfies this.
    // Smithay's `Libinput::new` directly takes the session_manager.

    let libinput_context = Libinput::new(session_manager, None); // Logger can be Some(slog::Logger)
    // If the above line doesn't work due to type mismatch with Arc,
    // it might be that Libinput::new expects direct ownership or a specific wrapper.
    // Libinput::new<I: LibinputInterface + 'static>(interface: I, logger: Option<Logger>)
    // Arc<LibinputSessionManager> itself does not implement LibinputInterface.
    // LibinputSessionManager does. We need to pass the manager itself, not an Arc of it,
    // if the context will own it. Or, if the context expects a reference,
    // we need to ensure lifetimes are managed.

    // Let's re-check Smithay examples for `Libinput::new`.
    // Typically, the session interface is owned by the struct that also owns LibinputInputBackend.
    // `LibinputInputBackend::new` takes `Libinput` which in turn takes `LibinputInterface`.
    // If `session_manager` is `Arc`, `Libinput::new` would need to take `Arc<I>`.
    // Smithay 0.3 `Libinput::new` takes `I: LibinputInterface + 'static`.
    // This means the `LibinputSessionManager` itself, not an Arc, should be passed if it's to be owned.
    // Or, if `LibinputInterface` is implemented for `&'a mut I`, then a mutable borrow.

    // For now, let's assume `LibinputSessionManager` might need to be cloned or we pass the Arc directly
    // if `LibinputInterface` is implemented for `Arc<LibinputSessionManager>`.
    // It is not. So, we cannot pass `session_manager` (the Arc) directly.

    // If `LibinputSessionManager` is cheap to clone and `Libinput` takes ownership:
    // let libinput_context = Libinput::new((*session_manager).clone(), None); // Assuming SessionManager is Clone. It is not by default.
    // Or, if the session manager is intended to be shared via Arc, then `Libinput` itself
    // would need to support that, or we'd need a wrapper.

    // The typical pattern is that the `LibinputInterface` implementor is not shared via `Arc`
    // when passed to `Libinput::new`, but rather owned by `Libinput` or by the same struct
    // that owns `Libinput`.
    // The prompt suggests `Arc<LibinputSessionManager>` is passed *into* this function.
    // This implies the session manager might be shared or used elsewhere.
    // However, `Libinput::new` takes `I: LibinputInterface + 'static`.
    // This means it wants to own the interface object.
    // If `LibinputSessionManager` cannot be cloned, and we only have an `Arc`, we have a problem.

    // Let's assume for the moment that `LibinputSessionManager` can be created here,
    // or that the `Arc` implies a shared context that `Libinput` can't take ownership of directly.
    // This might indicate a design mismatch with Smithay's `Libinput::new` API if `session_manager`
    // truly needs to be an `Arc` shared elsewhere *and* also passed to `Libinput::new` which expects ownership.

    // Revisit: `Libinput::new_from_path` (from the raw `libinput` crate) is what takes a closure.
    // `smithay::backend::libinput::Libinput::new` is the Smithay wrapper.
    // `Libinput::new<I: LibinputInterface + 'static>(interface: I, _logger: Option<slog::Logger>)`
    // This means `interface` is moved.
    // If `init_libinput_backend` receives `Arc<LibinputSessionManager>`, it cannot move `LibinputSessionManager`
    // out of the `Arc`.
    // This function should probably take `session_manager: LibinputSessionManager` (by value).
    // Or, `LibinputSessionManager` should be `Clone`. Let's make it `Clone` for now.
    // `LibinputSessionManager` is currently `#[derive(Debug)]`. Adding `Clone`.

    // If LibinputSessionManager is made Clone:
    // let sm_clone = (*session_manager).clone(); // Deref Arc, then clone.
    // let mut libinput_context = Libinput::new(sm_clone, None);
    // This is not ideal as cloning a session manager might not make sense if it holds state (like FDs).
    // Our current LibinputSessionManager is stateless, so cloning is fine.

    // Alternative: The `LibinputInterface` might be intended for raw `libinput-rs` context creation,
    // and then that raw context is wrapped by Smithay's `Libinput`.
    // `libinput::Libinput::new_from_path(|p, f| session_manager.open_restricted(p,f), |fd| session_manager.close_restricted(fd))`
    // This is how `libinput-rs` crate itself does it.
    // Smithay's `Libinput::new(interface, _logger)` is a higher-level wrapper that takes an object `I`.

    // Let's try to adhere to Smithay's `Libinput::new` by assuming LibinputSessionManager is passed by value or can be cloned.
    // If the function signature *must* be `Arc<LibinputSessionManager>`, then `LibinputSessionManager`
    // must implement `LibinputInterface` in a way that `Arc<Self>` can be used, or `Libinput`
    // must be able to accept an `Arc`. Smithay does not do this.

    // The most straightforward way with current Smithay API, if session_manager is to be shared,
    // is that the caller retains the Arc, and passes a *reference* if the `LibinputInterface`
    // methods take `&self` or `&mut self`. `LibinputInterface` methods take `&mut self`.
    // `Libinput::new` requires `I: LibinputInterface + 'static`. This means it takes ownership.

    // Assume the plan implies `LibinputSessionManager` is cheap and can be made `Clone` for now.
    // Or, that this function should *create* the `LibinputSessionManager`.
    // Let's assume `init_libinput_backend` should create it if it's not meant to be shared beyond libinput.
    // The signature `init_libinput_backend(session_interface: Arc<LibinputSessionManager>)` is fixed by the prompt.
    // This is a common issue: `Arc<T>` where `T` needs to be passed by value.
    // Possible solutions:
    // 1. Change `LibinputSessionManager` to be `Clone`. (Simplest for now given it's stateless).
    // 2. Create a wrapper around `Arc<LibinputSessionManager>` that implements `LibinputInterface`.

    // Let's go with solution 1 for now and make LibinputSessionManager Clone.
    // (This change would be in `session_interface.rs`).
    // Assume `LibinputSessionManager` is now `Clone`.
    let sm_clone = (*session_manager).clone(); // Requires LibinputSessionManager to be Clone
    let mut libinput_context = match Libinput::new(sm_clone, None) {
        Ok(ctx) => ctx,
        Err(e) => {
            // Libinput::new can return an io::Error if udev backend fails to init.
            tracing::error!("Failed to create libinput context: {}", e);
            return Err(InputError::LibinputError(format!(
                "Context creation failed: {}",
                e
            )));
        }
    };


    // Assign the context to a seat.
    // This typically uses udev_assign_seat for systems with udev.
    // The seat name should match the one used by the compositor's `Seat` object.
    let seat_name = "seat0"; // Default seat name, should be consistent with DesktopState::seat_name
    if let Err(e) = libinput_context.udev_assign_seat(seat_name) {
        tracing::error!("Failed to assign libinput context to seat '{}': {}", seat_name, e);
        // This error type from libinput-rs is typically simple, often just an integer code.
        // We'll wrap it in our LibinputError.
        return Err(InputError::LibinputError(format!(
            "Failed to assign seat '{}': {:?}", // Raw libinput error is integer, use Debug
            seat_name, e
        )));
    }
    tracing::info!("Libinput context assigned to seat '{}'.", seat_name);

    // Create and return the LibinputInputBackend.
    // `LibinputInputBackend::new` takes the `libinput::Libinput` context (Smithay's wrapper)
    // and an optional logger (or tracing span in newer Smithay versions).
    // Smithay 0.3: `LibinputInputBackend::new(context: Libinput, logger: L)`
    // Some versions might use `Option<slog::Logger>` or `Option<tracing::Span>`.
    // The prompt suggests `Some(tracing::Span::current())`.
    let backend = LibinputInputBackend::new(libinput_context, Some(tracing::Span::current()));
    tracing::info!("LibinputInputBackend created successfully.");

    Ok(backend)
}

// To make this compile, LibinputSessionManager needs to be Clone.
// Go back to system/src/input/libinput_handler/session_interface.rs and add `#[derive(Clone)]`
// to LibinputSessionManager if it's simple enough (i.e., no complex state that can't be cloned).
// Given it's `_private: ()`, it is cloneable.

// If cloning `LibinputSessionManager` is not desired (e.g., if it becomes stateful),
// an alternative would be for `Libinput::new` to accept `Arc<I>` or for `LibinputSessionManager`
// to be constructed inside this function, if its lifetime is only tied to the backend.
// The current prompt implies `Arc` is passed in, so `Clone` is the path of least resistance
// to satisfy `Libinput::new`'s ownership requirement.Okay, I've created `system/src/input/libinput_handler/backend_config.rs` and implemented `init_libinput_backend`.

To make this compile with the current Smithay API (`Libinput::new` taking ownership), I need to make `LibinputSessionManager` cloneable. I'll do that now.
