use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::{glib::subclass::Signal, Label, CompositeTemplate}; // Added Label
use std::cell::{Cell, RefCell};
use once_cell::sync::Lazy;
use chrono::Local;

// Not using CompositeTemplate for now, will create Label manually.
#[derive(Default)]
pub struct ClockDateTimeWidget {
    pub format_string: RefCell<String>,
    pub show_calendar_on_click: Cell<bool>,
    pub timer_id: RefCell<Option<glib::SourceId>>,
    // If not using TemplateChild, this would be an Option<gtk::Label> initialized in constructed
    pub time_label: RefCell<Option<Label>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ClockDateTimeWidget {
    const NAME: &'static str = "NovaDEClockDateTimeWidget";
    type Type = super::ClockDateTimeWidget;
    type ParentType = gtk::Button;

    fn new() -> Self {
        Self {
            format_string: RefCell::new("%H:%M".to_string()), // Default format
            show_calendar_on_click: Cell::new(true),
            timer_id: RefCell::new(None),
            time_label: RefCell::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
        klass.set_css_name("clockdatetimewidget");
    }
}

static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
    vec![
        glib::ParamSpecString::new(
            "format-string",
            "Format String",
            "The strftime format string for the clock.",
            Some("%H:%M"), // Default value
            glib::ParamFlags::READWRITE,
        ),
        glib::ParamSpecBoolean::new(
            "show-calendar-on-click",
            "Show Calendar on Click",
            "Whether to show a calendar popover when clicked.",
            true, // Default value
            glib::ParamFlags::READWRITE,
        ),
    ]
});

impl ObjectImpl for ClockDateTimeWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        // Initialize time_label
        let label = Label::new(None);
        self.time_label.replace(Some(label.clone()));
        obj.set_child(Some(self.time_label.borrow().as_ref().unwrap()));
        
        // Default values are set in new() or by ParamSpec, but ensure consistency if needed
        // self.format_string.replace("%H:%M".to_string());
        // self.show_calendar_on_click.set(true);

        obj.update_display_priv();
        obj.setup_timer_priv();

        obj.connect_clicked(|_btn| {
            // Placeholder for popover functionality
            // let show_calendar = btn.imp().show_calendar_on_click.get();
            // if show_calendar {
            //     println!("Clock clicked! Calendar popover would show here.");
            // }
        });
    }

    fn dispose(&self) {
        if let Some(source_id) = self.timer_id.borrow_mut().take() {
            source_id.remove();
        }
    }

    fn properties() -> &'static [glib::ParamSpec] {
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "format-string" => {
                let format_str = value.get().expect("Value must be a String for format-string");
                self.format_string.replace(format_str);
                self.obj().update_display_priv();
            }
            "show-calendar-on-click" => {
                let show = value.get().expect("Value must be a boolean for show-calendar-on-click");
                self.show_calendar_on_click.set(show);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "format-string" => self.format_string.borrow().to_value(),
            "show-calendar-on-click" => self.show_calendar_on_click.get().to_value(),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for ClockDateTimeWidget {}
impl ButtonImpl for ClockDateTimeWidget {}

// Private helper methods for ClockDateTimeWidget
impl ClockDateTimeWidget {
    pub fn update_display_priv_impl(&self) {
        if let Some(label) = self.time_label.borrow().as_ref() {
            let now = Local::now();
            let formatted_time = now.format(&self.format_string.borrow()).to_string();
            label.set_text(&formatted_time);
        }
    }

    pub fn setup_timer_priv_impl(&self) {
        let obj = self.obj();
        let source_id = glib::timeout_add_seconds_local(1, move || {
            obj.update_display_priv();
            glib::ControlFlow::Continue
        });
        self.timer_id.replace(Some(source_id));
    }
}
