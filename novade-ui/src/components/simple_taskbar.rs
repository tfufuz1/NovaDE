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
    // Note: The AdwButtonContent is not given an ID, so it's not a TemplateChild here.
    // If we needed to interact with it, we'd give it an ID in the .ui file.
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
