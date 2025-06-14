// novade-system/src/compositor/init_tests.rs
#[cfg(test)]
mod tests {
    use novade_core::types::display::{
        Display as CoreDisplay, DisplayMode as CoreDisplayMode, DisplayConnector,
        DisplayStatus, PhysicalProperties,
    };
    use smithay::output::{Mode as SmithayMode, PhysicalProperties as SmithayPhysicalProperties, Subpixel};
    // We don't instantiate a full `smithay::output::Output` here, as it involves globals and potentially logging.
    // We test the data transformation that would be used to create and configure such an Output.

    // This function replicates the conversion logic found in init.rs
    fn convert_core_display_to_smithay_output_data(
        core_display: &CoreDisplay,
    ) -> (
        String,                 // Name (from core_display.id)
        SmithayPhysicalProperties,
        Vec<SmithayMode>,
        Option<SmithayMode>,    // Current mode
    ) {
        let smithay_modes: Vec<SmithayMode> = core_display
            .modes
            .iter()
            .map(|core_mode| SmithayMode {
                size: (core_mode.width as i32, core_mode.height as i32).into(),
                refresh: core_mode.refresh_rate / 1000, // mHz to Hz
            })
            .collect();

        let current_smithay_mode = core_display.current_mode.as_ref().and_then(|core_mode| {
            smithay_modes
                .iter()
                .find(|sm| {
                    sm.size.w == core_mode.width as i32
                        && sm.size.h == core_mode.height as i32
                        && sm.refresh == (core_mode.refresh_rate / 1000)
                })
                .cloned()
        });

        // If no current_mode from core_display matches (e.g. if current_mode was None or not in modes list),
        // pick the first mode from smithay_modes or leave as None if no modes.
        let final_current_mode = current_smithay_mode.or_else(|| smithay_modes.get(0).cloned());

        let physical_props = core_display
            .physical_properties
            .as_ref()
            .map(|pp| SmithayPhysicalProperties {
                size: (pp.width_mm as i32, pp.height_mm as i32).into(),
                subpixel: Subpixel::Unknown, // Default, as core_display doesn't store this
                make: "NovaDE_Core".to_string(), // Generic make
                model: core_display.name.clone(),   // Use core_display.name as model
            })
            .unwrap_or_else(|| SmithayPhysicalProperties {
                size: (0, 0).into(), // Indicates unknown physical size
                subpixel: Subpixel::Unknown,
                make: "NovaDE_Core".to_string(),
                model: core_display.name.clone(),
            });

        (core_display.id.clone(), physical_props, smithay_modes, final_current_mode)
    }

    #[test]
    fn test_basic_display_conversion() {
        let core_mode = CoreDisplayMode { width: 1920, height: 1080, refresh_rate: 60000 }; // 60Hz
        let core_display = CoreDisplay {
            id: "DP-1".to_string(),
            name: "Test Monitor One".to_string(),
            connector: DisplayConnector::DisplayPort,
            status: DisplayStatus::Connected,
            modes: vec![core_mode.clone()],
            current_mode: Some(core_mode.clone()),
            physical_properties: Some(PhysicalProperties { width_mm: 527, height_mm: 296 }),
            position_x: 0,
            position_y: 0,
            enabled: true,
        };

        let (name, phys_props, smithay_modes, current_mode_opt) =
            convert_core_display_to_smithay_output_data(&core_display);

        assert_eq!(name, "DP-1");
        assert_eq!(phys_props.size, (527, 296).into());
        assert_eq!(phys_props.model, "Test Monitor One");
        assert_eq!(smithay_modes.len(), 1);
        assert_eq!(smithay_modes[0].size, (1920, 1080).into());
        assert_eq!(smithay_modes[0].refresh, 60); // 60000mHz -> 60Hz

        assert!(current_mode_opt.is_some());
        let current_mode = current_mode_opt.unwrap();
        assert_eq!(current_mode.size, (1920, 1080).into());
        assert_eq!(current_mode.refresh, 60);
    }

    #[test]
    fn test_conversion_multiple_modes_no_current() {
        let mode1 = CoreDisplayMode { width: 1920, height: 1080, refresh_rate: 60000 };
        let mode2 = CoreDisplayMode { width: 1280, height: 720, refresh_rate: 50000 }; // 50Hz
        let core_display = CoreDisplay {
            id: "HDMI-A-1".to_string(),
            name: "TV".to_string(),
            connector: DisplayConnector::HDMI,
            status: DisplayStatus::Connected,
            modes: vec![mode1.clone(), mode2.clone()],
            current_mode: None, // No current mode explicitly set
            physical_properties: None, // No physical properties
            position_x: 0,
            position_y: 0,
            enabled: true,
        };

        let (name, phys_props, smithay_modes, current_mode_opt) =
            convert_core_display_to_smithay_output_data(&core_display);

        assert_eq!(name, "HDMI-A-1");
        assert_eq!(phys_props.size, (0,0).into()); // Default physical size
        assert_eq!(phys_props.model, "TV");

        assert_eq!(smithay_modes.len(), 2);
        assert_eq!(smithay_modes[0].size, (1920, 1080).into());
        assert_eq!(smithay_modes[0].refresh, 60);
        assert_eq!(smithay_modes[1].size, (1280, 720).into());
        assert_eq!(smithay_modes[1].refresh, 50);

        // If current_mode is None, but modes exist, conversion logic should pick the first mode.
        assert!(current_mode_opt.is_some());
        let current_mode = current_mode_opt.unwrap();
        assert_eq!(current_mode.size, smithay_modes[0].size);
        assert_eq!(current_mode.refresh, smithay_modes[0].refresh);
    }

    #[test]
    fn test_conversion_no_modes() {
        let core_display = CoreDisplay {
            id: "VGA-1".to_string(),
            name: "Old Monitor".to_string(),
            connector: DisplayConnector::VGA,
            status: DisplayStatus::Connected,
            modes: vec![], // No modes
            current_mode: None,
            physical_properties: None,
            position_x: 0,
            position_y: 0,
            enabled: true,
        };

        let (_name, _phys_props, smithay_modes, current_mode_opt) =
            convert_core_display_to_smithay_output_data(&core_display);

        assert_eq!(smithay_modes.len(), 0);
        assert!(current_mode_opt.is_none()); // No modes, so current_mode should be None
    }

    #[test]
    fn test_conversion_current_mode_not_in_list() {
        // This case tests if current_mode is set to something not in the modes list.
        // The conversion logic should then default to the first mode in the list if available.
        let mode_in_list = CoreDisplayMode { width: 1920, height: 1080, refresh_rate: 60000 };
        let current_mode_not_in_list = CoreDisplayMode { width: 800, height: 600, refresh_rate: 75000 };
        let core_display = CoreDisplay {
            id: "DVI-1".to_string(),
            name: "Digital Panel".to_string(),
            connector: DisplayConnector::DVI,
            status: DisplayStatus::Connected,
            modes: vec![mode_in_list.clone()],
            current_mode: Some(current_mode_not_in_list.clone()), // This mode is not in the `modes` Vec
            physical_properties: None,
            position_x: 0,
            position_y: 0,
            enabled: true,
        };

        let (_name, _phys_props, smithay_modes, current_mode_opt) =
            convert_core_display_to_smithay_output_data(&core_display);

        assert_eq!(smithay_modes.len(), 1);
        assert_eq!(smithay_modes[0].size, (1920,1080).into());

        // current_mode_opt should fall back to the first mode in smithay_modes
        assert!(current_mode_opt.is_some());
        let selected_current_mode = current_mode_opt.unwrap();
        assert_eq!(selected_current_mode.size, smithay_modes[0].size);
        assert_eq!(selected_current_mode.refresh, smithay_modes[0].refresh);
    }
}
