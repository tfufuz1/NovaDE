// novade-system/src/compositor/protocols/xdg_activation_v1.rs
// Implementation of the xdg-activation-v1 Wayland protocol

use smithay::{
    reexports::{
        wayland_protocols::xdg::activation::v1::server::{
            xdg_activation_v1::{self, XdgActivationV1},
            xdg_activation_token_v1::{self, XdgActivationTokenV1},
        },
        wayland_server::{
            protocol::wl_surface, // Needed to associate activation with a surface
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    utils::{Serial, Logical, Point}, // Serial might be used with tokens
    wayland::seat::{Seat, WaylandFocus}, // To activate/focus a window
    // To interact with XDG Toplevels, similar to decoration manager
    wayland::shell::xdg::{XdgShellState, XdgToplevelSurfaceData, ToplevelSurface, WindowSheet},
    desktop::Window, // Smithay's Window type
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use thiserror::Error;
use tracing::{info, warn, error, debug};
use rand::Rng; // For generating random token strings

const TOKEN_VALIDITY_DURATION: Duration = Duration::from_secs(120); // 2 minutes, example duration

// Placeholder for DesktopState or a more specific state for managing activation.
// TODO: Integrate with the actual DesktopState or Window management.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder. We need access to windows/surfaces and seats.
    // pub windows: Vec<Window>, // From smithay::desktop::Window
    // pub seat: Option<Seat<Self>>, // Assuming DesktopState is the <D> for Seat
}

/// Represents an activation token and its associated data.
#[derive(Debug, Clone)]
pub struct ActivationTokenData {
    pub token_string: String,
    pub surface: Option<wl_surface::WlSurface>, // Surface that requested the token, if any
    pub app_id: Option<String>, // App ID associated with the requesting surface
    pub seat_serial: Option<Serial>, // Serial of the user event that triggered token creation
    pub created_at: Instant,
    pub requesting_client: Client, // The client that created this token
}

impl ActivationTokenData {
    pub fn is_valid(&self) -> bool {
        self.created_at.elapsed() < TOKEN_VALIDITY_DURATION
    }
}

/// Manages active activation tokens.
/// This state would typically be part of your main compositor state (e.g., NovaCompositorState).
#[derive(Debug, Default)]
pub struct XdgActivationState {
    tokens: HashMap<String, ActivationTokenData>,
}

impl XdgActivationState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a new unique token string.
    fn generate_token_string(&self) -> String {
        let mut rng = rand::thread_rng();
        loop {
            let token: String = std::iter::repeat(())
                .map(|()| rng.sample(rand::distributions::Alphanumeric))
                .map(char::from)
                .take(32)
                .collect();
            if !self.tokens.contains_key(&token) {
                return token;
            }
        }
    }

    /// Creates a new activation token.
    pub fn new_token(
        &mut self,
        surface: Option<wl_surface::WlSurface>,
        app_id: Option<String>,
        seat_serial: Option<Serial>,
        client: Client,
    ) -> ActivationTokenData {
        let token_string = self.generate_token_string();
        let token_data = ActivationTokenData {
            token_string: token_string.clone(),
            surface,
            app_id,
            seat_serial,
            created_at: Instant::now(),
            requesting_client: client,
        };
        self.tokens.insert(token_string, token_data.clone());
        debug!("New activation token created: {:?}", token_data.token_string);
        self.cleanup_expired_tokens();
        token_data
    }

    /// Retrieves and consumes a token if it's valid.
    /// Consuming means it cannot be used again by this function (though client might hold the object).
    pub fn consume_token(&mut self, token_string: &str) -> Option<ActivationTokenData> {
        self.cleanup_expired_tokens();
        if let Some(token_data) = self.tokens.get(token_string) {
            if token_data.is_valid() {
                // Protocol says token is single-use. Remove it.
                return self.tokens.remove(token_string);
            } else {
                debug!("Token {} is expired.", token_string);
                self.tokens.remove(token_string); // Remove expired token
            }
        }
        None
    }

    fn cleanup_expired_tokens(&mut self) {
        self.tokens.retain(|_, data| data.is_valid());
    }
}


#[derive(Debug, Error)]
pub enum XdgActivationError {
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("Surface to activate not found or not an XDG toplevel")]
    SurfaceNotActivatable,
    #[error("Activation request failed")]
    ActivationFailed,
}

// The main compositor state (e.g., NovaCompositorState) would implement Dispatch for these.
// It would also hold an instance of `XdgActivationState`.
// For example:
// pub struct NovaCompositorState {
//     ...
//     pub xdg_activation_state: XdgActivationState,
//     pub seat_state: SeatState<Self>, // To get current seat
//     pub xdg_shell_state: XdgShellState, // To find windows
//     pub space: Space<Window>, // To find windows
//     ...
// }

/// Handles dispatching of the XdgActivationV1 global.
/// `D` is your main compositor state, which must own `XdgActivationState`.
impl<D> GlobalDispatch<XdgActivationV1, (), D> for DesktopState // Replace DesktopState with D
where
    D: GlobalDispatch<XdgActivationV1, ()> + Dispatch<XdgActivationV1, (), D> +
       Dispatch<XdgActivationTokenV1, String, D> + 'static, // UserData for token is the token string
       // We need mutable access to XdgActivationState, Seat, and potentially XdgShellState/Space
       // This is typically achieved by D being the main state struct.
       // D: AsMut<XdgActivationState> + AsMut<SeatState<D>> + AsMut<XdgShellState> + ...
{
    fn bind(
        _state: &mut D, // The main compositor state
        _handle: &DisplayHandle,
        _client: &Client,
        resource: XdgActivationV1,
        _global_data: &(),
    ) {
        info!("Client bound XdgActivationV1: {:?}", resource);
        resource.quick_assign(|manager, request, dispatch_data| {
            // `dispatch_data` here is &mut D (our main state, e.g., NovaCompositorState)
            // We need to access `xdg_activation_state` from `dispatch_data`.
            // For this skeleton, we assume `dispatch_data` *is* or *contains* `XdgActivationState`.
            // A proper setup would be `let activation_state = &mut dispatch_data.xdg_activation_state;`

            // TODO: This requires D to actually contain XdgActivationState.
            // This is a placeholder for accessing the real XdgActivationState.
            let mut activation_state_placeholder = XdgActivationState::new(); // Incorrect, needs real state.
            let activation_state = &mut activation_state_placeholder; // Use this once D is set up.

            match request {
                xdg_activation_v1::Request::Destroy => {
                    info!("XdgActivationV1 {:?} destroyed by client", manager);
                }
                xdg_activation_v1::Request::GetActivationToken { id } => {
                    info!("Client requests GetActivationToken");
                    // No surface or app_id specified by client at this point for the token itself.
                    // These are associated when the token is *used* for activation, or implicitly by client context.
                    // The protocol implies the token is created now, and later associated with an activation request.
                    // The `activate` request takes the token string.
                    // The `xdg_activation_token_v1.set_surface` can associate a surface with the token *creator*.

                    // We need the client that requested this. `manager.client()` should give it.
                    let client = match manager.client() {
                        Some(c) => c,
                        None => {
                            // Should not happen for a bound resource
                            error!("GetActivationToken from a manager with no client");
                            return;
                        }
                    };

                    let token_data = activation_state.new_token(None, None, None, client);

                    let token_resource = id.implement_nonsend(dispatch_data, token_data.token_string.clone()).unwrap_or_else(|e| {
                        error!("Failed to implement XdgActivationTokenV1: {}", e);
                        panic!("Failed to implement XdgActivationTokenV1: {}", e);
                    });
                    token_resource.token(token_data.token_string); // Send the token string to client
                    info!("Created XdgActivationTokenV1 {:?} with token string {}", token_resource, token_data.token_string);
                }
                xdg_activation_v1::Request::Activate { token, surface } => {
                    info!("Client requests Activate with token '{}' for surface {:?}", token, surface);

                    let token_data = match activation_state.consume_token(&token) {
                        Some(data) => data,
                        None => {
                            warn!("Activation requested with invalid or expired token: {}", token);
                            // TODO: Should we send an error event on the surface or manager?
                            // The protocol doesn't specify an error for invalid token on activate.
                            // It might be that the activation simply fails silently from client's perspective,
                            // or the client is expected to handle token object destruction.
                            return;
                        }
                    };

                    debug!("Valid token data: {:?}", token_data);

                    // Now, attempt to activate the `surface`.
                    // 1. Find the window corresponding to `surface`.
                    // 2. Check if it's an XDG toplevel.
                    // 3. Bring it to the front, give it focus.
                    // This requires access to DesktopState/Space/Seat.

                    // TODO: Access real DesktopState/Space/Seat from `dispatch_data`
                    // let desktop_state = &mut dispatch_data.desktop_state;
                    // let seat = &mut dispatch_data.seat_state.get_seat(&default_seat_name); // Get a seat

                    // Placeholder for activation logic:
                    let found_window = with_states(&surface, |states| {
                        states.data_map.get::<XdgToplevelSurfaceData>().is_some() &&
                        states.data_map.get::<WindowSurfaceType>().is_some() // Check if it's a Smithay Window surface
                    });

                    if !found_window {
                        warn!("Activation requested for a surface that is not a known XDG toplevel window: {:?}", surface);
                        // TODO: Error handling?
                        return;
                    }

                    info!("Activating surface: {:?}", surface);
                    // Actual activation logic:
                    // - Find the `smithay::desktop::Window` associated with `surface`.
                    // - Get the current `Seat`.
                    // - Call `seat.get_keyboard().unwrap().set_focus(..., serial_from_token_or_now)`.
                    // - Bring window to top of stacking order (`space.raise_window(...)`).
                    // - Send `configure` events if necessary.

                    // Example using Smithay's Window and Seat (requires proper state access):
                    /*
                    if let Some(window_to_activate) = find_window_for_surface(&dispatch_data.space, &surface) {
                        dispatch_data.space.raise_window(&window_to_activate, true); // Bring to top

                        if let Some(seat) = dispatch_data.seat_state.seats.values_mut().next() { // Get first seat
                            if let Some(keyboard) = seat.get_keyboard() {
                                let focus_serial = token_data.seat_serial.unwrap_or_else(Serial::now);
                                keyboard.set_focus(dispatch_data, Some(window_to_activate.clone().into()), focus_serial);
                                info!("Set keyboard focus to {:?} with serial {:?}", window_to_activate, focus_serial);

                                // Send xdg_toplevel.configure with activated state
                                if let Kind::Xdg(xdg_surface) = window_to_activate.toplevel() {
                                    xdg_surface.with_pending_state(|toplevel_state| {
                                        toplevel_state.states.set(xdg_toplevel::State::Activated);
                                    });
                                    xdg_surface.send_configure();
                                    info!("Sent configure with Activated state for {:?}", xdg_surface);
                                }
                            } else {
                                warn!("No keyboard found on seat to set focus for activation.");
                            }
                        } else {
                            warn!("No seat found to perform activation.");
                        }
                    } else {
                        warn!("Could not find smithay::desktop::Window for surface {:?} during activation.", surface);
                    }
                    */
                    warn!("Placeholder: Actual window activation logic needs to be implemented using DesktopState/Space/Seat.");

                }
                _ => unimplemented!("Request not implemented for XdgActivationV1"),
            }
        });
    }

    fn can_view(_client: Client, _global_data: &()) -> bool {
        true // Any client can use xdg-activation
    }
}


/// Handles dispatching of XdgActivationTokenV1 objects.
/// `D` is your main compositor state.
impl<D> Dispatch<XdgActivationTokenV1, String, D> for DesktopState // Replace DesktopState with D
where
    D: Dispatch<XdgActivationTokenV1, String, D> + 'static,
    // D: AsMut<XdgActivationState> // To modify token data
{
    fn request(
        state: &mut D, // The main compositor state
        _client: &Client,
        resource: &XdgActivationTokenV1, // The token object
        request: xdg_activation_token_v1::Request,
        token_string_from_data: &String, // UserData (the token string)
        _dhandle: &DisplayHandle,
        _data_init: &mut smithay::reexports::wayland_server::DataInit<'_, D>,
    ) {
        debug!("Request for XdgActivationTokenV1 {:?} (token: {}): {:?}", resource, token_string_from_data, request);

        // TODO: Access real XdgActivationState from `state`
        let mut activation_state_placeholder = XdgActivationState::new(); // Incorrect
        let activation_state = &mut activation_state_placeholder;

        match request {
            xdg_activation_token_v1::Request::SetSerial { serial, seat } => {
                // Client provides serial of the event that triggered the token request.
                // This is useful for focus stealing prevention.
                if let Some(token_data) = activation_state.tokens.get_mut(token_string_from_data) {
                    token_data.seat_serial = Some(serial);
                    // We might also want to store the `seat` itself if relevant.
                    info!("Token {}: associated serial {:?} from seat {:?}", token_string_from_data, serial, seat);
                } else {
                    warn!("SetSerial for a token ({}) that no longer exists or is invalid.", token_string_from_data);
                }
            }
            xdg_activation_token_v1::Request::SetSurface { surface } => {
                // Client associates the token with a surface it owns.
                // This surface might be the one requesting activation for another app,
                // or the one to be activated if the token is passed around.
                if let Some(token_data) = activation_state.tokens.get_mut(token_string_from_data) {
                    token_data.surface = Some(surface.clone()); // Clone wl_surface if needed
                    // We could also try to get app_id here if not already set.
                    if token_data.app_id.is_none() {
                        let app_id = with_states(&surface, |states| {
                            states.data_map.get::<XdgToplevelSurfaceData>()
                                .and_then(|data| data.app_id.clone())
                        });
                        if app_id.is_some() {
                           token_data.app_id = app_id;
                        }
                    }
                    info!("Token {}: associated with surface {:?}, app_id {:?}", token_string_from_data, surface, token_data.app_id);
                } else {
                     warn!("SetSurface for a token ({}) that no longer exists or is invalid.", token_string_from_data);
                }
            }
            xdg_activation_token_v1::Request::SetAppId { app_id } => {
                // Client explicitly sets an app_id for the token.
                 if let Some(token_data) = activation_state.tokens.get_mut(token_string_from_data) {
                    token_data.app_id = Some(app_id);
                    info!("Token {}: associated with app_id {:?}", token_string_from_data, token_data.app_id);
                } else {
                    warn!("SetAppId for a token ({}) that no longer exists or is invalid.", token_string_from_data);
                }
            }
            xdg_activation_token_v1::Request::Destroy => {
                info!("XdgActivationTokenV1 {:?} (token: {}) destroyed by client", resource, token_string_from_data);
                // The token object is destroyed. The actual token data in XdgActivationState
                // might persist until it expires or is consumed by `Activate`.
                // The protocol says: "the compositor may invalidate the token".
                // Let's remove it from our active list if the client destroys the object,
                // as it implies the client is done with it.
                if activation_state.tokens.remove(token_string_from_data).is_some() {
                    debug!("Removed token data for {} due to object destruction.", token_string_from_data);
                }
            }
            _ => unimplemented!("Request not implemented for XdgActivationTokenV1"),
        }
    }

    fn destroyed(
        _state: &mut D,
        _client_id: wayland_server::backend::ClientId,
        resource_id: wayland_server::backend::ObjectId,
        token_string_from_data: &String, // UserData
    ) {
        info!("XdgActivationTokenV1 resource (id: {:?}, token: {}) fully destroyed (e.g. client disconnect)", resource_id, token_string_from_data);
        // Similar to Destroy request, ensure token data is cleaned up if not already.
        // This is more for compositor-side cleanup if the client disappears.
        // TODO: Access real XdgActivationState.
        // let mut activation_state_placeholder = XdgActivationState::new();
        // let activation_state = &mut activation_state_placeholder;
        // if activation_state.tokens.remove(token_string_from_data).is_some() {
        //     debug!("Removed token data for {} due to resource destruction (e.g. client disconnect).", token_string_from_data);
        // }
        warn!("Token {} destroyed. Ensure XdgActivationState is properly accessed for cleanup.", token_string_from_data);
    }
}


/// Initializes and registers the XDG Activation V1 global.
/// `D` is your main compositor state type.
pub fn init_xdg_activation<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed
) -> Result<Global<XdgActivationV1>, Box<dyn std::error::Error>>
where
    D: GlobalDispatch<XdgActivationV1, ()> + Dispatch<XdgActivationV1, (), D> +
       Dispatch<XdgActivationTokenV1, String, D> + 'static,
       // D: AsMut<XdgActivationState> + ... // Ensure D can provide the necessary states
{
    info!("Initializing XDG Activation V1 global (xdg_activation_v1)");

    // Create XdgActivationState. This state needs to be managed by your compositor (in D).
    // Example: state.xdg_activation_state = XdgActivationState::new();

    let global = display.create_global::<D, XdgActivationV1, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    );
    Ok(global)
}

// TODO:
// - Proper State Integration:
//   - `XdgActivationState` must be part of the main compositor state `D`.
//   - Dispatch implementations need mutable access to `XdgActivationState`, `SeatState`,
//     `XdgShellState`, and `Space`/`DesktopState` for window manipulation.
//     The current placeholder `DesktopState` and direct instantiation of `XdgActivationState`
//     in handlers are incorrect and need to be replaced with proper access via `D`.
// - Activation Logic:
//   - The `Activate` request handler needs full implementation:
//     - Find the `smithay::desktop::Window` for the `wl_surface`.
//     - Use `Seat::set_focus()` and `Space::raise_window()`.
//     - Send `xdg_toplevel.configure` with `activated` state.
//     - Handle focus stealing prevention using token's serial and surface data.
// - Testing:
//   - App A gets a token.
//   - App A passes token to App B (e.g., via environment variable or custom protocol).
//   - App B uses token to activate a surface of App A.
//   - Token expiration and reuse prevention.
//   - Activation of a surface of the token-requesting app itself.

// Helper function (conceptual) to find a Smithay Window for a WlSurface
// This would live in your compositor's window management logic.
/*
fn find_window_for_surface<'a, D>(space: &'a Space<Window>, surface_to_find: &wl_surface::WlSurface) -> Option<&'a Window>
where
    D: 'static, // Adjust D as per your Space's UserData if any
{
    space.elements().find(|window| {
        window.wl_surface().map_or(false, |s| s == surface_to_find)
    })
}
*/

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod xdg_activation_v1;
