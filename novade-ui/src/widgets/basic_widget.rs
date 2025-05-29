use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{prelude::*, CompositeTemplate};

// Define the GObject wrapper for our widget.
glib::wrapper! {
    pub struct BasicWidget(ObjectSubclass<BasicWidgetPriv>)
        @extends gtk::Box, gtk::Widget, // Specify GtkBox as the parent class in the template
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

// Implementation of BasicWidget.
impl BasicWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    // Example public method to interact with the widget's components
    pub fn set_label_text(&self, text: &str) {
        self.imp().label.set_text(text);
    }

    pub fn connect_button_clicked<F: Fn(&Self) + 'static>(&self, callback: F) {
        self.imp().button.connect_clicked(glib::clone!(@weak self as widget => move |_| {
            callback(&widget);
        }));
    }

    pub fn set_main_image_from_icon_name(&self, icon_name: &str) {
        self.imp().image.set_from_icon_name(Some(icon_name));
    }

    pub fn set_status_image_from_icon_name(&self, icon_name: &str) {
        self.imp().status_image.set_from_icon_name(Some(icon_name));
    }

    pub fn set_status_image_from_resource(&self, resource_path: &str) {
        self.imp().status_image.set_from_resource(Some(resource_path));
    }
}

// Define the "private" implementation details of our GObject.
// This is where @template parts are specified.
#[derive(CompositeTemplate, Default)]
#[template(file = "basic_widget.ui")] // Path relative to Cargo.toml or using a specific resolver
pub struct BasicWidgetPriv {
    #[template_child]
    pub label: TemplateChild<gtk::Label>,
    #[template_child]
    pub button: TemplateChild<gtk::Button>,
    #[template_child]
    pub image: TemplateChild<gtk::Image>, // This is the main image
    #[template_child]
    pub status_image: TemplateChild<gtk::Image>, // This is the new status image
}

// GObject subclassing boilerplate.
#[glib::object_subclass]
impl ObjectSubclass for BasicWidgetPriv {
    const NAME: &'static str = "BasicWidget";
    type Type = BasicWidget;
    type ParentType = gtk::Box; // Corresponds to parent in UI file <template class="BasicWidget" parent="GtkBox">

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait implementations for GObject.
impl ObjectImpl for BasicWidgetPriv {
    // `dispose` is where you free resources allocated by your widget.
    fn dispose(&self) {
        while let Some(child) = self.obj().first_child() {
            child.unparent();
        }
    }
}

// Trait implementations for the GtkBox aspects of our widget.
impl BoxImpl for BasicWidgetPriv {} // Needed because BasicWidget extends gtk::Box
impl WidgetImpl for BasicWidgetPriv {} // Basic GtkWidget behavior
impl OrientableImpl for BasicWidgetPriv {} // If your widget is orientable

// Note on UI file path in `#[template(file = "basic_widget.ui")]`:
// The path is usually relative to the source file (`basic_widget.rs`).
// So, `basic_widget.ui` should be in the same directory as `basic_widget.rs`.
// If it's not found, GTK will panic at runtime with a "Could not find template" error.
// I have placed `basic_widget.ui` in `novade-ui/src/widgets/basic_widget.ui`.
// The macro `#[template(file = "basic_widget.ui")]` should work if the ui file is in the same directory.
// If `basic_widget.rs` is in `src/widgets/` and `basic_widget.ui` is also in `src/widgets/`, it should be fine.
// Let's ensure the `Cargo.toml` of `novade-ui` is aware of this structure if needed, though typically not for .rs and .ui files directly.
// The path used in `#[template(file = "basic_widget.ui")]` is relative to the file containing the macro.
// So, if `basic_widget.rs` is in `novade-ui/src/widgets/`, then `basic_widget.ui` should also be there.
// This matches the paths I've used.

// I'll also need to make sure the `novade-ui` crate's `lib.rs` or `main.rs` declares the `widgets` module.
// `main.rs` is a binary, so `lib.rs` for `novade-ui` should declare `pub mod widgets;` if it exists,
// or `main.rs` should if `widgets` is local to the binary's module structure.
// Given the previous structure, `novade-ui/src/lib.rs` probably should declare `pub mod widgets;`
// Or, if `main.rs` is the crate root for the binary, `mod widgets;` there.
// I'll check `novade-ui/src/lib.rs` or `novade-ui/src/main.rs` for module declaration.
// The subtask asks for `novade-ui/src/widgets/mod.rs`, which I created.
// This means `novade-ui/src/main.rs` should have `mod widgets;`
// or `novade-ui/src/lib.rs` (if `novade-ui` is also a library) should have `pub mod widgets;`.
// Let's assume `main.rs` needs `mod widgets;`.

```
