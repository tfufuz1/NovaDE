use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};
use adw::ButtonContent; // Import AdwButtonContent if used in template

// Define the GObject wrapper for our widget.
glib::wrapper! {
    pub struct SimpleTaskbar(ObjectSubclass<SimpleTaskbarPriv>)
        @extends gtk::ConstraintLayout, gtk::Widget, // Changed from gtk::Box
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget; // Removed gtk::Orientable
}

// Implementation of SimpleTaskbar.
impl SimpleTaskbar {
    pub fn new() -> Self {
        glib::Object::new()
    }

    // Example: Method to update the clock label (though not a real clock yet)
    pub fn set_clock_text(&self, text: &str) {
        self.imp().clock_label.set_text(text);
    }

    // In a real taskbar, you'd have methods to add/remove task items
    // e.g., add_task_item(&self, app_id: &str, window_title: &str)
}

// Define the "private" implementation details of our GObject.
#[derive(CompositeTemplate, Default)]
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
}

// GObject subclassing boilerplate.
#[glib::object_subclass]
impl ObjectSubclass for SimpleTaskbarPriv {
    const NAME: &'static str = "SimpleTaskbar";
    type Type = SimpleTaskbar;
    type ParentType = gtk::ConstraintLayout; // Changed from gtk::Box

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // Add custom CSS for the taskbar if needed, though applying "taskbar" class in UI is better
        // klass.add_css_class("taskbar"); // This applies to the widget itself
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
        // Setup draw function for the DrawingArea
        let widget = obj.get(); // Get the wrapper SimpleTaskbar instance
        
        // Setup draw function for the DrawingArea (as before)
        widget.imp().status_indicator_area.set_draw_func(|_drawing_area, cr, width, height| {
            let radius = (width.min(height) as f64 / 2.0) - 2.0; 
            if radius <= 0.0 { return; }
            cr.arc(width as f64 / 2.0, height as f64 / 2.0, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.set_source_rgb(0.3, 0.8, 0.3); 
            if let Err(e) = cr.fill() { eprintln!("Cairo fill failed: {:?}", e); }
        });

        // Setup animation button
        let clock_label_clone = widget.imp().clock_label.get(); // Get the GtkLabel instance
        widget.imp().animate_clock_button.connect_clicked(move |_| {
            let clock_label = clock_label_clone.clone(); // Clone for use in each animation stage
            
            // Animation: 1.0 (opaque) to 0.0 (transparent)
            let anim_fade_out = gtk::PropertyAnimation::new_for_target(
                &clock_label,
                "opacity",
                1.0, // from_value (explicitly start from current or known start)
                0.0  // to_value
            );
            anim_fade_out.set_duration(500); // 500 ms
            anim_fade_out.set_easing(gtk::Easing::Linear);

            // Animation: 0.0 (transparent) to 1.0 (opaque)
            let anim_fade_in = gtk::PropertyAnimation::new_for_target(
                &clock_label,
                "opacity",
                0.0, // from_value
                1.0  // to_value
            );
            anim_fade_in.set_duration(500);
            anim_fade_in.set_easing(gtk::Easing::Linear);

            // Chain animations: play fade_in after fade_out is done
            let clock_label_for_fade_in = clock_label.clone();
            anim_fade_out.connect_done(move |_animation| {
                // Ensure opacity is actually 0 before starting fade in,
                // as animation might be interrupted or end slightly off.
                clock_label_for_fade_in.set_opacity(0.0); 
                anim_fade_in.play();
            });
            
            // Start the first animation
            // Ensure opacity is 1.0 before starting, in case it was interrupted mid-animation before
            clock_label.set_opacity(1.0); 
            anim_fade_out.play();
        });
    }
}

impl ObjectImpl for SimpleTaskbarPriv {
    fn dispose(&self) {
        while let Some(child) = self.obj().first_child() {
            child.unparent();
        }
    }
}

// Remove BoxImpl and OrientableImpl as ParentType is no longer GtkBox
// impl BoxImpl for SimpleTaskbarPriv {} // GtkConstraintLayout is not a GtkBox
impl WidgetImpl for SimpleTaskbarPriv {}
// impl OrientableImpl for SimpleTaskbarPriv {} // GtkConstraintLayout is not Orientable by default
