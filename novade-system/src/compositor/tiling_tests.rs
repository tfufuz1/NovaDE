// ANCHOR: TilingAlgorithmTests
//! Unit tests for tiling algorithms.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use smithay::utils::{Rectangle, Logical, Point, Size};
use uuid::Uuid;

// Needs access to ManagedWindow, DomainWindowIdentifier, CompositorWorkspace, TilingLayout etc.
// Assuming these are pub from their respective modules and accessible via crate::compositor...
use crate::compositor::shell::xdg_shell::types::{ManagedWindow, DomainWindowIdentifier, WindowState, WindowManagerData, WindowSurface as CompositorWindowSurface}; // Renamed to avoid conflict
use crate::compositor::workspaces::{CompositorWorkspace, TilingLayout};
use crate::compositor::tiling::calculate_master_stack_layout;

// Minimal mock for parts of Smithay's ToplevelSurface needed by ManagedWindow::new_toplevel
// This is highly simplified and only for constructing ManagedWindow.
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::shell::xdg::{ToplevelSurface, PopupSurface, XdgSurface, WindowSurface as SmithayWindowSurface, Role, XdgPopupSurfaceData, XdgToplevelSurfaceData, XdgSurfaceUserData as SmithayXdgSurfaceUserData, XdgShellHandler}; // Added XdgShellHandler
use smithay::reexports::wayland_server::{Client, Main, Resource, UserData, DisplayHandle, Dispatch, Proxy}; // Added Proxy
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::XdgToplevel;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_popup;


// Mocking a WlSurface to the extent needed for ManagedWindow.
// This is non-trivial. For these tests, we might not need a fully live WlSurface.
// Let's assume ManagedWindow can be constructed with a simplified ToplevelSurface.

// Simplified mock ToplevelSurface constructor for testing.
// In a real test with a Wayland server, these are created differently.
fn mock_smithay_toplevel_surface(dh: &DisplayHandle, client: &Client) -> ToplevelSurface {
    let wl_surface_main = client.create_object::<WlSurface, _>(dh, 1, UserData::default()).unwrap();
    let wl_surface = wl_surface_main.into_proxy(); // Get Proxy<WlSurface>

    // Attach necessary user data for XdgSurface creation
    wl_surface.data::<UserData>().unwrap().insert_if_missing(|| Arc::new(smithay::wayland::compositor::SurfaceData::new(None, Rectangle::default())));
    wl_surface.data::<UserData>().unwrap().insert_if_missing(|| smithay::wayland::shell::xdg::XdgSurfaceData::new());

    let xdg_surface_role_data = Arc::new(smithay::wayland::shell::xdg::XdgToplevelSurfaceData::new());
    let xdg_surface = XdgSurface::new_specific(
        wl_surface.clone(), // XdgSurface::new needs &WlSurface
        Role::Toplevel,
        xdg_surface_role_data.clone() as Arc<_>
    );
    ToplevelSurface::from_xdg_surface(xdg_surface, xdg_surface_role_data).unwrap()
}


fn create_test_managed_window(title: &str, is_master: bool, dh: &DisplayHandle, client: &Client) -> Arc<ManagedWindow> {
    let mock_toplevel = mock_smithay_toplevel_surface(dh, client); // This needs a DisplayHandle and Client
    mock_toplevel.set_title(title.to_string());

    let domain_id = DomainWindowIdentifier::new_v4();
    let mw = ManagedWindow::new_toplevel(mock_toplevel, domain_id);
    *mw.tiling_master.write().unwrap() = is_master;
    Arc::new(mw)
}


#[test]
fn test_calculate_master_stack_layout_no_windows() {
    let windows = Vec::new();
    let area = Rectangle::from_loc_and_size((0, 0), (800, 600));
    let layouts = calculate_master_stack_layout(&windows, area, 0.6);
    assert!(layouts.is_empty());
}

#[test]
fn test_calculate_master_stack_layout_one_window() {
    let mut display = smithay::reexports::wayland_server::Display::<()>::new().unwrap();
    let dh = display.handle();
    let client = Client::new_for_testing(&dh, UserData::default());

    let windows = vec![create_test_managed_window("Win1", false, &dh, &client)];
    let area = Rectangle::from_loc_and_size((0, 0), (800, 600));
    let layouts = calculate_master_stack_layout(&windows, area, 0.6);

    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts.get(&windows[0].domain_id), Some(&area));
}

#[test]
fn test_calculate_master_stack_layout_two_windows_no_explicit_master() {
    let mut display = smithay::reexports::wayland_server::Display::<()>::new().unwrap();
    let dh = display.handle();
    let client = Client::new_for_testing(&dh, UserData::default());

    let windows = vec![
        create_test_managed_window("Win1", false, &dh, &client), // Becomes master by default
        create_test_managed_window("Win2", false, &dh, &client),
    ];
    let area = Rectangle::from_loc_and_size((0, 0), (1000, 600)); // Use 1000 for easy 60/40 split
    let layouts = calculate_master_stack_layout(&windows, area, 0.6);

    assert_eq!(layouts.len(), 2);
    // Master (Win1)
    assert_eq!(
        layouts.get(&windows[0].domain_id),
        Some(&Rectangle::from_loc_and_size((0, 0), (600, 600)))
    );
    // Stack (Win2)
    assert_eq!(
        layouts.get(&windows[1].domain_id),
        Some(&Rectangle::from_loc_and_size((600, 0), (400, 600)))
    );
}

#[test]
fn test_calculate_master_stack_layout_three_windows_explicit_master() {
    let mut display = smithay::reexports::wayland_server::Display::<()>::new().unwrap();
    let dh = display.handle();
    let client = Client::new_for_testing(&dh, UserData::default());

    let windows = vec![
        create_test_managed_window("Win1", false, &dh, &client),
        create_test_managed_window("MasterWin", true, &dh, &client), // Explicit master
        create_test_managed_window("Win3", false, &dh, &client),
    ];
    let area = Rectangle::from_loc_and_size((0, 0), (1000, 600));
    let layouts = calculate_master_stack_layout(&windows, area, 0.6);

    assert_eq!(layouts.len(), 3);
    // Master (MasterWin)
    assert_eq!(
        layouts.get(&windows[1].domain_id), // Index 1 is MasterWin
        Some(&Rectangle::from_loc_and_size((0, 0), (600, 600)))
    );
    // Stack (Win1, Win3)
    assert_eq!(
        layouts.get(&windows[0].domain_id), // Win1
        Some(&Rectangle::from_loc_and_size((600, 0), (400, 300)))
    );
    assert_eq!(
        layouts.get(&windows[2].domain_id), // Win3
        Some(&Rectangle::from_loc_and_size((600, 300), (400, 300)))
    );
}

#[test]
fn test_calculate_master_stack_layout_master_only_with_stack_width_zero() {
    let mut display = smithay::reexports::wayland_server::Display::<()>::new().unwrap();
    let dh = display.handle();
    let client = Client::new_for_testing(&dh, UserData::default());

    let windows = vec![
        create_test_managed_window("Master", true, &dh, &client),
        create_test_managed_window("Stack1", false, &dh, &client),
    ];
    let area = Rectangle::from_loc_and_size((0, 0), (1000, 600));
    let layouts = calculate_master_stack_layout(&windows, area, 1.0); // Master takes 100%

    assert_eq!(layouts.len(), 2);
    assert_eq!(
        layouts.get(&windows[0].domain_id), // Master
        Some(&Rectangle::from_loc_and_size((0, 0), (1000, 600)))
    );
    assert_eq!(
        layouts.get(&windows[1].domain_id), // Stack1 (should be 0-width or handled)
        Some(&Rectangle::from_loc_and_size((0,0), (0,0))) // Current impl gives 0x0 at origin
    );
}
// ANCHOR_END: TilingAlgorithmTests
