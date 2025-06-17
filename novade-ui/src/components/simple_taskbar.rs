use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};
use adw::ButtonContent; // Keep if used in .ui, though not directly in Rust code shown for clock
use std::cell::{Cell, RefCell};
use once_cell::sync::Lazy; // For static PROPERTIES
use glib::ParamSpec; // For ParamSpec types
use chrono::Local; // For getting current time

// Define GObject properties for SimpleTaskbar
static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
    vec![
        glib::ParamSpecString::builder("clock-format-string")
            .nick("Clock Format String")
            .blurb("Format string for the clock label (strftime format).")
            .default_value(Some("%H:%M"))
            .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT)
            .build(),
        glib::ParamSpecBoolean::builder("show-seconds")
            .nick("Show Seconds")
            .blurb("Whether to include seconds in the clock display.")
            .default_value(false)
            .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT)
            .build(),
    ]
});

glib::wrapper! {
    pub struct SimpleTaskbar(ObjectSubclass<SimpleTaskbarPriv>)
        @extends gtk::ConstraintLayout, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SimpleTaskbar {
    pub fn new() -> Self {
        glib::Object::new()
    }

    // Public getter/setter for clock-format-string
    pub fn clock_format_string(&self) -> String {
        self.property("clock-format-string")
    }

    pub fn set_clock_format_string(&self, format_string: &str) {
        self.set_property("clock-format-string", format_string);
    }

    // Public getter/setter for show-seconds
    pub fn show_seconds(&self) -> bool {
        self.property("show-seconds")
    }

    pub fn set_show_seconds(&self, show: bool) {
        self.set_property("show-seconds", show);
    }
}

#[derive(CompositeTemplate)]
#[template(file = "simple_taskbar.ui")]
pub struct SimpleTaskbarPriv {
    #[template_child]
    pub clock_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub task_items_box: TemplateChild<gtk::Box>,
    #[template_child]
    pub status_indicator_area: TemplateChild<gtk::DrawingArea>,
    #[template_child]
    pub animate_clock_button: TemplateChild<gtk::Button>,

    // New fields for clock logic and properties
    pub clock_format_string: RefCell<String>,
    pub show_seconds: Cell<bool>,
    pub clock_timer_id: RefCell<Option<glib::SourceId>>,
}

// Default implementation for SimpleTaskbarPriv
impl Default for SimpleTaskbarPriv {
    fn default() -> Self {
        Self {
            clock_label: TemplateChild::default(),
            task_items_box: TemplateChild::default(),
            status_indicator_area: TemplateChild::default(),
            animate_clock_button: TemplateChild::default(),
            clock_format_string: RefCell::new("%H:%M".to_string()), // Default format
            show_seconds: Cell::new(false), // Default show_seconds
            clock_timer_id: RefCell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for SimpleTaskbarPriv {
    const NAME: &'static str = "SimpleTaskbar";
    type Type = SimpleTaskbar;
    type ParentType = gtk::ConstraintLayout;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.install_properties(&PROPERTIES);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
        let widget = obj.get(); // Get the wrapper SimpleTaskbar instance
        widget.imp().setup_initial_ui_state(); // Call setup for initial state
        widget.imp().start_clock_timer_priv(); // Start the clock timer
    }
}

impl ObjectImpl for SimpleTaskbarPriv {
    fn properties() -> &'static [ParamSpec] {
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            "clock-format-string" => {
                let format_string = value.get().expect("Value must be a string for clock-format-string");
                self.clock_format_string.replace(format_string);
                self.update_clock_display_priv(); // Update display immediately
            }
            "show-seconds" => {
                let show = value.get().expect("Value must be a boolean for show-seconds");
                self.show_seconds.set(show);
                self.update_clock_display_priv(); // Update display immediately
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            "clock-format-string" => self.clock_format_string.borrow().to_value(),
            "show-seconds" => self.show_seconds.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn dispose(&self) {
        // Remove clock timer
        if let Some(source_id) = self.clock_timer_id.take() {
            source_id.remove();
        }
        // Default dispose behavior for template children
        while let Some(child) = self.obj().first_child() {
            child.unparent();
        }
    }
}

impl WidgetImpl for SimpleTaskbarPriv {}

// Implementation for SimpleTaskbarPriv specific logic
impl SimpleTaskbarPriv {
    fn setup_initial_ui_state(&self) {
        // Status Indicator (existing logic)
        let status_indicator_area = self.status_indicator_area.get();
        status_indicator_area.set_draw_func(|_drawing_area, cr, width, height| {
            let radius = (width.min(height) as f64 / 2.0) - 2.0; 
            if radius <= 0.0 { return; }
            cr.arc(width as f64 / 2.0, height as f64 / 2.0, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.set_source_rgb(0.3, 0.8, 0.3);
            if let Err(e) = cr.fill() { eprintln!("Cairo fill failed: {:?}", e); }
        });
        let accessible_indicator = status_indicator_area.accessible();
        accessible_indicator.set_accessible_role(gtk::AccessibleRole::Image);
        if let Err(e) = accessible_indicator.update_property(gtk::AccessibleProperty::Label, &"System status: OK".to_value()) {
            eprintln!("Failed to set accessible label for status indicator: {:?}", e);
        }

        // Animate Clock Button (existing logic)
        let clock_label_clone = self.clock_label.get();
        self.animate_clock_button.connect_clicked(move |_| {
            let clock_label = clock_label_clone.clone();
            let anim_fade_out = gtk::PropertyAnimation::new_for_target(&clock_label, "opacity", 1.0, 0.0);
            anim_fade_out.set_duration(500);
            anim_fade_out.set_easing(gtk::Easing::Linear);
            let anim_fade_in = gtk::PropertyAnimation::new_for_target(&clock_label, "opacity", 0.0, 1.0);
            anim_fade_in.set_duration(500);
            anim_fade_in.set_easing(gtk::Easing::Linear);
            let clock_label_for_fade_in = clock_label.clone();
            anim_fade_out.connect_done(move |_animation| {
                clock_label_for_fade_in.set_opacity(0.0); 
                anim_fade_in.play();
            });
            clock_label.set_opacity(1.0); 
            anim_fade_out.play();
        });
    }

    fn update_clock_display_priv(&self) {
        let now = Local::now();
        let base_format_str = self.clock_format_string.borrow();
        let show_secs = self.show_seconds.get();

        let effective_format_str = if show_secs && !base_format_str.contains("%S") {
            // If show_seconds is true and format doesn't have seconds, append :%S
            // This is a simple heuristic; users might want more control.
            format!("{}:%S", *base_format_str)
        } else if !show_secs && base_format_str.contains(":%S") {
            // If show_seconds is false and format has seconds, try to remove them
            // This is also heuristic. A better way is for user to manage full format.
            base_format_str.replace(":%S", "")
        } else if !show_secs && base_format_str.contains("%S") && !base_format_str.contains(":%S"){
             base_format_str.replace("%S", "") // Handles case like %H%M%S
        }
        else {
            base_format_str.clone()
        };

        let time_str = now.format(&effective_format_str).to_string();
        self.clock_label.set_text(&time_str);
    }

    fn start_clock_timer_priv(&self) {
        self.update_clock_display_priv(); // Initial update

        if self.clock_timer_id.borrow().is_some() {
            return; // Timer already started
        }

        let source_id = glib::timeout_add_seconds_local(1, clone!(@weak self.obj() as taskbar => @default-panic, move || {
            taskbar.imp().update_clock_display_priv();
            glib::ControlFlow::Continue
        }));
        self.clock_timer_id.replace(Some(source_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk::glib; // For property functions
    use chrono::Local;

    fn init_gtk() {
        if !gtk::is_initialized() {
            gtk::test_init();
        }
    }

    #[test]
    fn test_simple_taskbar_clock_format_property() {
        init_gtk();
        let taskbar = SimpleTaskbar::new();

        // Test default value
        let default_format: String = taskbar.property("clock-format-string");
        assert_eq!(default_format, "%H:%M");
        assert_eq!(taskbar.clock_format_string(), "%H:%M");

        // Test setting a new value
        let new_format = "%Y-%m-%d %H:%M";
        taskbar.set_clock_format_string(new_format);

        let retrieved_format: String = taskbar.property("clock-format-string");
        assert_eq!(retrieved_format, new_format);
        assert_eq!(taskbar.clock_format_string(), new_format);

        // Test setting via set_property
        let another_format = "%I:%M %p";
        taskbar.set_property("clock-format-string", another_format);
        assert_eq!(taskbar.clock_format_string(), another_format);
    }

    #[test]
    fn test_simple_taskbar_show_seconds_property() {
        init_gtk();
        let taskbar = SimpleTaskbar::new();

        // Test default value
        let default_show_seconds: bool = taskbar.property("show-seconds");
        assert_eq!(default_show_seconds, false);
        assert_eq!(taskbar.show_seconds(), false);

        // Test setting to true
        taskbar.set_show_seconds(true);
        let retrieved_show_seconds: bool = taskbar.property("show-seconds");
        assert_eq!(retrieved_show_seconds, true);
        assert_eq!(taskbar.show_seconds(), true);

        // Test setting via set_property
        taskbar.set_property("show-seconds", false);
        assert_eq!(taskbar.show_seconds(), false);
    }

    #[test]
    fn test_simple_taskbar_clock_label_update() {
        init_gtk();
        let taskbar = SimpleTaskbar::new();
        let taskbar_imp = taskbar.imp();

        // Test with default format, no seconds
        taskbar.set_clock_format_string("%H:%M");
        taskbar.set_show_seconds(false);
        taskbar_imp.update_clock_display_priv(); // Call private method for testing

        let now = Local::now();
        let expected_text_hm = now.format("%H:%M").to_string();
        assert_eq!(taskbar_imp.clock_label.text(), expected_text_hm);

        // Test with default format, show seconds (heuristic appends :%S)
        taskbar.set_show_seconds(true);
        // update_clock_display_priv is called automatically when show_seconds is set
        // taskbar_imp.update_clock_display_priv();
        let expected_text_hms_heuristic = now.format("%H:%M:%S").to_string();
        assert_eq!(taskbar_imp.clock_label.text(), expected_text_hms_heuristic);

        // Test with custom format that includes seconds, show_seconds=false (heuristic removes :%S)
        taskbar.set_clock_format_string("%H:%M:%S");
        taskbar.set_show_seconds(false);
        // taskbar_imp.update_clock_display_priv();
        let expected_text_custom_no_s = now.format("%H:%M").to_string();
        assert_eq!(taskbar_imp.clock_label.text(), expected_text_custom_no_s);

        // Test with custom format that includes seconds, show_seconds=true (format used as is)
        taskbar.set_clock_format_string("%I:%M:%S %p");
        taskbar.set_show_seconds(true);
        // taskbar_imp.update_clock_display_priv();
        let expected_text_custom_with_s = now.format("%I:%M:%S %p").to_string();
        assert_eq!(taskbar_imp.clock_label.text(), expected_text_custom_with_s);

        // Test with custom format that does NOT include seconds, show_seconds=true (heuristic appends :%S)
        taskbar.set_clock_format_string("%I:%M %p");
        taskbar.set_show_seconds(true);
        // taskbar_imp.update_clock_display_priv();
        let expected_text_custom_no_s_show_s = now.format("%I:%M %p:%S").to_string();
        assert_eq!(taskbar_imp.clock_label.text(), expected_text_custom_no_s_show_s);
    }

    // Note: Timer starting and continuous updates are harder to test in simple unit tests
    // without specific main loop iterations or async test utilities for GLib timeouts.
    // The `dispose` method correctly removes the timer source_id, which is good practice.
}
