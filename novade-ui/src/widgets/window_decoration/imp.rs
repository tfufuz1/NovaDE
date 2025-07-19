// novade-ui/src/widgets/window_decoration/imp.rs

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

#[derive(Debug, Default)]
pub struct WindowDecoration {}

#[glib::object_subclass]
impl ObjectSubclass for WindowDecoration {
    const NAME: &'static str = "NovaWindowDecoration";
    type Type = super::WindowDecoration;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("window-decoration");
    }
}

impl ObjectImpl for WindowDecoration {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let title_label = gtk::Label::new(Some("Window Title"));
        title_label.set_halign(gtk::Align::Center);
        title_label.set_hexpand(true);

        let minimize_button = gtk::Button::new();
        minimize_button.add_css_class("minimize-button");
        minimize_button.connect_clicked(|_| {
            println!("Minimize button clicked");
        });

        let maximize_button = gtk::Button::new();
        maximize_button.add_css_class("maximize-button");
        maximize_button.connect_clicked(|_| {
            println!("Maximize button clicked");
        });

        let close_button = gtk::Button::new();
        close_button.add_css_class("close-button");
        close_button.connect_clicked(|_| {
            println!("Close button clicked");
        });

        obj.pack_start(&title_label, true, true, 0);
        obj.pack_end(&close_button, false, false, 0);
        obj.pack_end(&maximize_button, false, false, 0);
        obj.pack_end(&minimize_button, false, false, 0);
    }
}

impl WidgetImpl for WindowDecoration {}
impl BoxImpl for WindowDecoration {}
