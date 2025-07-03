// novade-system/src/compositor/protocols/idle_notify.rs
// Implementation of the ext_idle_notify_unstable_v1 Wayland protocol

use smithay::{
    delegate_idle_notifier, // Smithay's delegate macro for this protocol
    reexports::{
        wayland_protocols_ext::idle_notify::v1::server::{
            ext_idle_notification_v1::{self, ExtIdleNotificationV1, Request as NotificationRequest, Event as NotificationEvent},
            ext_idle_notifier_v1::{self, ExtIdleNotifierV1, Request as NotifierRequest},
        },
        wayland_server::{
            protocol::wl_seat, // Idle state is typically per-seat
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::{LoopHandle, Timer, EventSource, Interest, Readiness, Token, PostAction, // For timers
                  timer::{TimerHandle, TimeoutAction}},
    },
    input::Seat, // Smithay's Seat abstraction
    utils::{Serial, MonotonicTime, Duration}, // For time calculations
    wayland::idle_notify::{
        IdleNotifyHandler, IdleNotifierState, IdleNotificationData, // Smithay's types
    },
};
use std::{
    sync::{Arc, Mutex},
    time::Duration as StdDuration, // For timer durations
    collections::HashMap,
};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `IdleNotifierState` and manage user activity tracking.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Idle Notify, it would need to manage or access:
    // - IdleNotifierState
    // - SeatState and activity on each seat.
    // - A mechanism to track the last user activity time per seat.
    // - Timers to trigger idle notifications.
    // - Potentially, integration with system-level idle services (logind, upower) via D-Bus.
}

#[derive(Debug, Error)]
pub enum IdleNotifyError {
    #[error("Seat is invalid or not provided for idle notification")]
    InvalidSeat,
    #[error("Timeout value is too short or invalid")]
    InvalidTimeout,
    // No specific errors defined in the protocol for manager/notification creation.
}

// The main compositor state (e.g., NovaCompositorState) would implement IdleNotifyHandler
// and store IdleNotifierState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub idle_notifier_state: IdleNotifierState,
//     pub seat_state: SeatState<Self>,
//     pub loop_handle: LoopHandle<'static, Self>, // For managing timers
//     last_activity_per_seat: HashMap<String, MonotonicTime>, // Seat name -> last activity time
//     active_idle_timers: HashMap<ObjectId, TimerHandle<SeatIdleTimer>>, // Timer for each notification
//     ...
// }
//
// struct SeatIdleTimer { seat_name: String, notification_id: ObjectId }
//
// impl<E: EventSource> EventSource for SeatIdleTimer { /* ... */ }
//
// impl IdleNotifyHandler for NovaCompositorState {
//     fn idle_notifier_state(&mut self) -> &mut IdleNotifierState {
//         &mut self.idle_notifier_state
//     }
//
//     fn new_notification(&mut self, notification: ExtIdleNotificationV1, seat_resource: wl_seat::WlSeat, timeout_ms: u32) {
//         info!("New idle notification {:?} requested for seat {:?} with timeout {}ms", notification, seat_resource, timeout_ms);
//         // 1. Find the Smithay Seat corresponding to `seat_resource`.
//         // 2. Get the last activity time for this seat.
//         // 3. Calculate if currently idle for `timeout_ms`.
//         //    - If yes, send `idled` event immediately.
//         //    - If no, start a timer for (timeout_ms - time_since_last_activity).
//         //       When timer fires, send `idled`.
//         // Store `notification` object to send events on it. Smithay's IdleNotifierState does this.
//         // The `IdleNotificationData` created by Smithay for the `notification` resource stores the timeout and seat.
//
//         let seat_name = get_seat_name_from_resource(&seat_resource); // Your logic
//         let last_activity = self.last_activity_per_seat.get(&seat_name).cloned().unwrap_or_else(MonotonicTime::now);
//         let time_since_activity = MonotonicTime::now() - last_activity;
//         let timeout_duration = StdDuration::from_millis(timeout_ms as u64);
//
//         if time_since_activity >= timeout_duration {
//             notification.idled();
//             // Mark as idled in IdleNotificationData if needed
//         } else {
//             let remaining_time = timeout_duration - time_since_activity;
//             let timer = self.loop_handle.insert_source(
//                 SeatIdleTimer { seat_name: seat_name.clone(), notification_id: notification.id() },
//                 remaining_time,
//                 |timer_data, _timer_handle, main_state| {
//                     // Find notification object by ID, send idled.
//                     if let Some(notif_obj) = main_state.idle_notifier_state.find_notification_by_id(timer_data.notification_id) {
//                         notif_obj.idled();
//                     }
//                     TimeoutAction::Drop // Or Reschedule if it's a repeating timer (not for this protocol)
//                 }
//             ).expect("Failed to insert idle timer");
//             self.active_idle_timers.insert(notification.id(), timer);
//         }
//     }
//
//     fn notification_destroyed(&mut self, notification: ExtIdleNotificationV1, seat_resource: wl_seat::WlSeat) {
//         info!("Idle notification {:?} for seat {:?} destroyed", notification, seat_resource);
//         // Remove any active timer associated with this notification.
//         if let Some(timer_handle) = self.active_idle_timers.remove(&notification.id()) {
//             timer_handle.cancel();
//         }
//         // Smithay's IdleNotifierState handles removing the IdleNotificationData.
//     }
// }
// delegate_idle_notifier!(NovaCompositorState);

impl IdleNotifyHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn idle_notifier_state(&mut self) -> &mut IdleNotifierState {
        // TODO: Properly integrate IdleNotifierState with DesktopState or NovaCompositorState.
        panic!("IdleNotifyHandler::idle_notifier_state() needs proper integration.");
        // Example: &mut self.nova_compositor_state.idle_notifier_state
    }

    fn new_notification(
        &mut self,
        notification_resource: ExtIdleNotificationV1, // The new notification object from client
        seat_resource: &wl_seat::WlSeat,             // The seat this notification is for
        timeout_millis: u32,                        // Timeout in milliseconds
    ) {
        info!(
            "New idle_notification {:?} requested for seat {:?} with timeout {}ms",
            notification_resource, seat_resource, timeout_millis
        );

        // A client has created an `ExtIdleNotificationV1` object, indicating it wants to be
        // notified when the given `seat_resource` has been idle for `timeout_millis`.

        // Smithay's `IdleNotifierState` (accessed via `self.idle_notifier_state()`) and the
        // `delegate_idle_notifier!` macro handle:
        // - Creating `IdleNotificationData` and associating it with `notification_resource`.
        //   This data stores the `seat_resource.id()`, `timeout_millis`, and the `notification_resource` itself.
        // - Calling this `new_notification` handler.

        // Our responsibilities here:
        // 1. Find the Smithay `Seat` corresponding to `seat_resource`.
        //    (This requires access to `SeatState` or a way to map `wl_seat` to `Seat`).
        // 2. Get the last recorded user activity time for that `Seat`.
        //    (The compositor's input loop must update this timestamp on any relevant input).
        // 3. Compare `timeout_millis` with the time elapsed since last activity.
        //    - If already idle for >= `timeout_millis`: send `idled()` event on `notification_resource` immediately.
        //      Update `IdleNotificationData` to reflect it's idled.
        //    - If not yet idle: start a timer. When the timer fires, send `idled()`.
        //      Store the `TimerHandle` so it can be cancelled if the notification is destroyed
        //      or if activity resumes before timeout.

        // TODO: Implement the logic described above. This requires:
        //  - Access to Seat objects from wl_seat resources.
        //  - A way to get/store last activity time per Seat (e.g., in Seat's UserData or a HashMap in main state).
        //  - Access to a `calloop::LoopHandle` to schedule timers.
        //  - A way to find the `ExtIdleNotificationV1` resource from the timer callback (e.g., by its ID).

        warn!(
            "TODO: Implement idle timer logic for notification {:?} (timeout: {}ms). Requires activity tracking and timers.",
            notification_resource, timeout_millis
        );

        // Example of immediate idle (conceptual, needs real state access):
        // let seat_name = get_seat_name(seat_resource); // Your function
        // let last_activity = self.get_last_activity_for_seat(&seat_name); // Your function
        // if MonotonicTime::now().duration_since(last_activity) >= StdDuration::from_millis(timeout_millis as u64) {
        //     notification_resource.idled();
        //     if let Some(data) = self.idle_notifier_state().get_notification_data(&notification_resource) {
        //         data.lock().unwrap().set_idled(true); // Mark in Smithay's data
        //     }
        // } else {
        //     // Start timer...
        // }
    }

    fn notification_destroyed(
        &mut self,
        notification_resource: &ExtIdleNotificationV1, // The notification object being destroyed
        _seat_resource: &wl_seat::WlSeat,              // The seat it was for
    ) {
        info!("Idle notification {:?} destroyed by client.", notification_resource);

        // The client has destroyed the `ExtIdleNotificationV1` object.

        // Smithay's `IdleNotifierState` will remove its `IdleNotificationData`.
        // Our responsibilities here:
        // 1. If there was an active timer associated with this `notification_resource`, cancel it.

        // TODO: Implement timer cancellation.
        // This requires storing `TimerHandle`s, e.g., in a HashMap keyed by `notification_resource.id()`.
        // if let Some(timer_handle) = self.active_idle_timers.remove(&notification_resource.id()) {
        //     timer_handle.cancel();
        //     debug!("Cancelled idle timer for destroyed notification {:?}", notification_resource);
        // }
        warn!(
            "TODO: Implement cancellation of timer for destroyed notification {:?}",
            notification_resource
        );
    }
}

/// Call this function whenever user activity is detected on a specific seat.
///
/// - `compositor_state`: Your main compositor state.
/// - `seat`: The Smithay `Seat` on which activity occurred.
/// - `activity_time`: The timestamp of the activity (CLOCK_MONOTONIC).
///
/// This function will:
/// 1. Update the last activity time for the seat.
/// 2. For all `ExtIdleNotificationV1` objects associated with this `seat`:
///    - If they were in the "idled" state, send `resumed()` event.
///    - Cancel any pending "idle" timers for them.
///    - Restart their timers based on the new `activity_time` and their original timeout.
///
/// `D` is your main compositor state.
pub fn on_seat_activity<D>(
    compositor_state: &mut D, // Your main compositor state
    seat: &Seat<D>,         // The Smithay Seat where activity happened
    activity_time: MonotonicTime,
) where
    D: IdleNotifyHandler + AsMut<IdleNotifierState> + 'static, // Plus access to timers, seat activity map
{
    info!("User activity detected on seat {:?} at {:?}", seat.name(), activity_time);

    // TODO: Update last_activity_time for this seat in your main state.
    // self.last_activity_per_seat.insert(seat.name().to_string(), activity_time);

    let idle_notifier_state = compositor_state.as_mut(); // Get &mut IdleNotifierState

    // Iterate through all notifications pertinent to this seat.
    // Smithay's `IdleNotifierState` stores `IdleNotificationData` which includes the seat ID.
    // We need to find all `IdleNotificationData` for `seat.wl_seat().id()`.
    // Then, for each:
    //   - Get its `ExtIdleNotificationV1` resource.
    //   - Get its timeout.
    //   - If it was marked as "idled", send `resumed()` and clear the idled flag.
    //   - Cancel its existing timer (if any).
    //   - Start a new timer for `timeout` duration from `activity_time`.

    // This is complex because IdleNotifierState doesn't directly expose iteration by seat.
    // It might be easier to iterate all notifications and check their seat_id.
    idle_notifier_state.for_each_notification(|data_guard| {
        let mut data = data_guard.lock().unwrap(); // data is IdleNotificationDataInner
        if data.seat_id() == seat.wl_seat().unwrap().id() { // Check if this notification is for the active seat
            debug!("Processing activity for notification {:?}", data.resource());

            if data.is_idled() {
                data.resource().resumed();
                data.set_idled(false); // Clear idled state
                info!("Sent resumed for notification {:?}", data.resource());
            }

            // TODO: Cancel existing timer for `data.resource().id()`.
            // if let Some(timer_handle) = self.active_idle_timers.remove(&data.resource().id()) {
            //     timer_handle.cancel();
            // }

            // TODO: Start new timer for `data.timeout()` from `activity_time`.
            // let timeout_duration = StdDuration::from_millis(data.timeout() as u64);
            // let new_timer = self.loop_handle.insert_source( ... );
            // self.active_idle_timers.insert(data.resource().id(), new_timer);
            // info!("Restarted idle timer for notification {:?} with timeout {:?}", data.resource(), timeout_duration);
        }
    });

    warn!(
        "TODO: Implement full on_seat_activity logic (update last_activity, manage timers for notifications on seat {:?})",
        seat.name()
    );
}


// delegate_idle_notifier!(DesktopState); // Needs to be NovaCompositorState

/// Initializes and registers the ExtIdleNotifierV1 global.
/// `D` is your main compositor state type.
pub fn init_idle_notifier<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // Needed for timers, pass to D or make accessible
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<ExtIdleNotifierV1, ()> +
       Dispatch<ExtIdleNotifierV1, (), D> +
       Dispatch<ExtIdleNotificationV1, UserData, D> + // UserData for notification (Smithay uses IdleNotificationData)
       IdleNotifyHandler + SeatHandler<D> + 'static,
       // D must also own IdleNotifierState, SeatState, and have access to a Calloop LoopHandle.
{
    info!("Initializing ExtIdleNotifierV1 global (ext-idle-notify-unstable-v1)");

    // Create IdleNotifierState. This state needs to be managed by your compositor (in D).
    // Example: state.idle_notifier_state = IdleNotifierState::new();

    // The compositor needs to track user activity on each seat.
    // This involves monitoring keyboard, pointer, touch events in the main input loop.
    // When activity is detected, call `on_seat_activity`.

    display.create_global::<D, ExtIdleNotifierV1, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_idle_notifier!(D)` is called in your main compositor state setup.
    // This macro handles:
    // - Dispatching ExtIdleNotifierV1 requests (`get_idle_notification`).
    //   It calls `IdleNotifyHandler::new_notification`.
    // - Dispatching ExtIdleNotificationV1 requests (destroy).
    //   It calls `IdleNotifyHandler::notification_destroyed`.
    // - Managing `IdleNotificationData` for each notification resource.

    info!("ExtIdleNotifierV1 global initialized.");
    Ok(())
}

// TODO:
// - Activity Tracking:
//   - Implement robust user activity tracking per seat in the compositor's main input loop.
//     Any relevant input event (key press, pointer motion/button, touch) should update
//     the last activity timestamp for the seat and call `on_seat_activity`.
// - Timer Management:
//   - Fully implement timer creation in `IdleNotifyHandler::new_notification` (when not immediately idle).
//   - Store `TimerHandle`s and implement timer cancellation in `notification_destroyed`
//     and during `on_seat_activity` (before restarting timers).
//   - The timer callback must correctly find the `ExtIdleNotificationV1` resource (e.g., by ID
//     stored in timer's data) and send the `idled()` event.
// - State Integration:
//   - `IdleNotifierState`, `SeatState`, a `LoopHandle` for timers, and the map of last activity times
//     (and active timers) must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `IdleNotifyHandler` and `SeatHandler`.
//   - `delegate_idle_notifier!(NovaCompositorState);` macro must be used.
// - System Idle Integration (Optional but Recommended):
//   - For more accurate or system-wide idle detection, consider integrating with D-Bus services
//     like `org.freedesktop.login1.Manager` (provides `IdleHint` signal) or `org.freedesktop.PowerManagement`
//     (or UPower). This can supplement or override input-based activity tracking.
// - Testing:
//   - Use clients that utilize this protocol (e.g., `swayidle`, custom scripts, or parts of desktop shells).
//   - Verify that `idled` events are sent after the correct timeout of inactivity.
//   - Verify that `resumed` events are sent immediately upon user activity after being idled.
//   - Test with multiple notifications for the same seat with different timeouts.
//   - Test with multiple seats if the compositor supports them.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod idle_notify;
