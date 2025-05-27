use glib;
use gtk::glib::subclass::Signal;
use gtk::subclass::prelude::*;
use gtk::{glib, ApplicationWindow, Box, CompositeTemplate};
use std::cell::{Cell, RefCell};
use once_cell::sync::Lazy;
use gtk4_layer_shell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "NovaDEPanelPosition")]
pub enum PanelPosition {
    #[enum_value(name = "Top", nick = "top")]
    Top,
    #[enum_value(name = "Bottom", nick = "bottom")]
    Bottom,
}

impl Default for PanelPosition {
    fn default() -> Self {
        PanelPosition::Top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModulePosition {
    Start,
    Center,
    End,
}

#[derive(CompositeTemplate, Default)]
#[template(string = "")] // Will define UI in code for now
pub struct PanelWidget {
    #[template_child]
    pub main_box: TemplateChild<Box>,
    #[template_child]
    pub start_box: TemplateChild<Box>,
    #[template_child]
    pub center_box: TemplateChild<Box>,
    #[template_child]
    pub end_box: TemplateChild<Box>,

    // Properties
    pub position: RefCell<PanelPosition>,
    pub panel_height: Cell<i32>,
    pub transparency_enabled: Cell<bool>,
    pub leuchtakzent_color: RefCell<Option<gdk::RGBA>>,
    pub leuchtakzent_intensity: Cell<f64>,
    pub drawing_area: RefCell<Option<gtk::DrawingArea>>,
}

#[glib::object_subclass]
impl ObjectSubclass for PanelWidget {
    const NAME: &'static str = "NovaDEPanelWidget";
    type Type = super::PanelWidget;
    type ParentType = gtk::ApplicationWindow;

    fn new() -> Self {
        Self {
            main_box: TemplateChild::default(),
            start_box: TemplateChild::default(),
            center_box: TemplateChild::default(),
            end_box: TemplateChild::default(),
            position: RefCell::new(PanelPosition::Top),
            panel_height: Cell::new(48), // Default height
            transparency_enabled: Cell::new(false),
            leuchtakzent_color: RefCell::new(None),
            leuchtakzent_intensity: Cell::new(0.5), // Default intensity
            drawing_area: RefCell::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        // PanelWidget::bind_template(klass); // No template for now

        klass.install_properties(&PROPERTIES);
        klass.set_css_name("panelwidget");
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
    vec![
        glib::ParamSpecEnum::new(
            "position",
            "Position",
            "The position of the panel on the screen.",
            PanelPosition::static_type(),
            PanelPosition::Top as i32,
            glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
        ),
        glib::ParamSpecInt::new(
            "panel-height",
            "Panel Height",
            "The height of the panel.",
            0,
            i32::MAX,
            48, // Default height
            glib::ParamFlags::READWRITE,
        ),
        glib::ParamSpecBoolean::new(
            "transparency-enabled",
            "Transparency Enabled",
            "Whether the panel should be transparent.",
            false,
            glib::ParamFlags::READWRITE,
        ),
        glib::ParamSpecBoxed::new(
            "leuchtakzent-color",
            "Leuchtakzent Color",
            "The color of the Leuchtakzent.",
            gdk::RGBA::static_type(),
            glib::ParamFlags::READWRITE,
        ),
        glib::ParamSpecDouble::new(
            "leuchtakzent-intensity",
            "Leuchtakzent Intensity",
            "The intensity of the Leuchtakzent.",
            0.0,
            1.0,
            0.5, // Default intensity
            glib::ParamFlags::READWRITE,
        ),
    ]
});

impl ObjectImpl for PanelWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        // Initialize boxes
        let main_box_inner = Box::new(gtk::Orientation::Horizontal, 0);
        let start_box_inner = Box::new(gtk::Orientation::Horizontal, 0);
        let center_box_inner = Box::new(gtk::Orientation::Horizontal, 0);
        let end_box_inner = Box::new(gtk::Orientation::Horizontal, 0);
        
        self.main_box.set_inner(&main_box_inner);
        self.start_box.set_inner(&start_box_inner);
        self.center_box.set_inner(&center_box_inner);
        self.end_box.set_inner(&end_box_inner);

        center_box_inner.set_hexpand(true);

        // DrawingArea for Leuchtakzent
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        // Prepend to main_box_inner so it's drawn under other modules
        main_box_inner.prepend(&drawing_area); 
        self.drawing_area.replace(Some(drawing_area.clone()));

        drawing_area.set_draw_func({
            let widget = obj.downgrade();
            move |da, cr, width, height| {
                if let Some(widget) = widget.upgrade() {
                    widget.imp().draw_leuchtakzent(da, cr, width, height);
                }
            }
        });
        
        main_box_inner.append(&*self.start_box);
        main_box_inner.append(&*self.center_box);
        main_box_inner.append(&*self.end_box);
        
        obj.set_child(Some(&*self.main_box));

        obj.setup_layer_shell();
        obj.update_layout();
        obj.update_transparency(); 

        obj.add_css_class("nova-panel");
        match *self.position.borrow() {
            PanelPosition::Top => obj.add_css_class("panel-top"),
            PanelPosition::Bottom => obj.add_css_class("panel-bottom"),
        }

        // Simulate theme change for testing Leuchtakzent
        // In a real scenario, this would come from a theme service event or GSettings
        // For now, we set it directly to test the drawing logic.
        // The ThemeService stub will also call this with its predefined color.
        obj.set_leuchtakzent_color(Some(gdk::RGBA::new(0.9, 0.2, 0.2, 1.0))); // Example Red, to see it's different from theme service
        obj.set_leuchtakzent_intensity(0.8);


        // Conceptual theme service subscription:
        // let theme_service = crate::theming_gtk::ThemeService::new(); // Or access a global instance
        // let widget_clone = obj.clone();
        // theme_service.subscribe_to_theme_changes(move |event| {
        //     println!("ThemeChangedEvent received in PanelWidget: {:?}", event.accent_color);
        //     // Ensure this runs on the GTK main thread if the event comes from another thread
        //     glib::MainContext::default().spawn_local(async move {
        //        widget_clone.set_leuchtakzent_color(Some(event.accent_color));
        //     });
        // });
        // For this stub, the ThemeService's subscribe method calls the callback immediately,
        // so the color set by ThemeService might override the one set above if not handled carefully.
        // For now, the direct call above will be the one primarily tested visually unless that is changed.
        // Let's ensure the ThemeService is instantiated and its subscription is called to demonstrate linkage:
        
        let theme_service = crate::theming_gtk::ThemeService::new();
        let widget_clone = obj.downgrade(); // Use downgrade for closures to avoid cycles
        theme_service.subscribe_to_theme_changes(move |event| {
            if let Some(widget) = widget_clone.upgrade() {
                 println!("ThemeChangedEvent (stub) received in PanelWidget: new accent_color: {:?}", event.accent_color);
                 // To make the theme service color visible, we set it here.
                 // This will override the direct red color set above because the stub service calls back immediately.
                 widget.set_leuchtakzent_color(Some(event.accent_color));
            }
        });

    }

    fn properties() -> &'static [glib::ParamSpec] {
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        let obj = self.obj();
        match pspec.name() {
            "position" => {
                let new_position = value.get().expect("Value must be a PanelPosition");
                let old_position = self.position.replace(new_position);
                if old_position != new_position {
                    obj.setup_layer_shell();
                    match old_position {
                        PanelPosition::Top => obj.remove_css_class("panel-top"),
                        PanelPosition::Bottom => obj.remove_css_class("panel-bottom"),
                    }
                    match new_position {
                        PanelPosition::Top => obj.add_css_class("panel-top"),
                        PanelPosition::Bottom => obj.add_css_class("panel-bottom"),
                    }
                    if let Some(da) = self.drawing_area.borrow().as_ref() {
                        da.queue_draw();
                    }
                }
            }
            "panel-height" => {
                let height = value.get().expect("Value must be an i32");
                self.panel_height.set(height);
                self.main_box.set_height_request(height);
                obj.set_default_size(1, height);
                if let Some(da) = self.drawing_area.borrow().as_ref() {
                    da.queue_draw();
                }
            }
            "transparency-enabled" => {
                let enabled = value.get().expect("Value must be a boolean");
                self.transparency_enabled.set(enabled);
                obj.update_transparency();
            }
            "leuchtakzent-color" => {
                self.leuchtakzent_color
                    .replace(value.get().expect("Value must be an RGBA for leuchtakzent-color"));
                if let Some(da) = self.drawing_area.borrow().as_ref() {
                    da.queue_draw();
                }
            }
            "leuchtakzent-intensity" => {
                self.leuchtakzent_intensity
                    .set(value.get().expect("Value must be a f64 for leuchtakzent-intensity"));
                if let Some(da) = self.drawing_area.borrow().as_ref() {
                    da.queue_draw();
                }
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "position" => self.position.borrow().to_value(),
            "panel-height" => self.panel_height.get().to_value(),
            "transparency-enabled" => self.transparency_enabled.get().to_value(),
            "leuchtakzent-color" => self.leuchtakzent_color.borrow().to_value(),
            "leuchtakzent-intensity" => self.leuchtakzent_intensity.get().to_value(),
            _ => unimplemented!(),
        }
    }

    // No signals defined yet
    // fn signals() -> &'static [Signal] {
    //     &[]
    // }
}

impl WidgetImpl for PanelWidget {
    // Override measure to ensure drawing_area gets the correct size if needed,
    // though hexpand/vexpand should handle it for simple cases.
}
impl WindowImpl for PanelWidget {}
impl ApplicationWindowImpl for PanelWidget {}

// Methods for PanelWidget accessible from super::PanelWidget in mod.rs
impl PanelWidget {
    fn draw_leuchtakzent(&self, _drawing_area: &gtk::DrawingArea, cr: &cairo::Context, width: i32, height: i32) {
        let color_opt = self.leuchtakzent_color.borrow();
        let intensity = self.leuchtakzent_intensity.get();

        if let Some(rgba) = color_opt.as_ref() {
            if intensity > 0.0 {
                let panel_pos = *self.position.borrow();
                let line_width = 2.0; // Thickness of the Leuchtakzent line

                cr.set_source_rgba(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    (rgba.alpha() as f64) * intensity,
                );
                cr.set_line_width(line_width);

                let edge_y = if panel_pos == PanelPosition::Top {
                    height as f64 - line_width / 2.0 // Draw along the bottom edge
                } else {
                    line_width / 2.0 // Draw along the top edge
                };
                
                cr.move_to(0.0, edge_y);
                cr.line_to(width as f64, edge_y);
                cr.stroke().expect("Cairo stroke operation failed");
            }
        }
    }

    pub fn setup_layer_shell_priv(&self) {
        let obj = self.obj();
        // gtk4_layer_shell::init_for_window(&obj); // This is called once in main.rs
        gtk4_layer_shell::set_layer(&obj, gtk4_layer_shell::Layer::Top);
        gtk4_layer_shell::set_keyboard_mode(&obj, gtk4_layer_shell::KeyboardMode::None);
        gtk4_layer_shell::auto_exclusive_zone_enable(&obj);
        // gtk4_layer_shell::set_monitor(&obj, &gdk::Display::default().unwrap().primary_monitor().unwrap());
        gtk4_layer_shell::set_namespace(&obj, "NovaDEPanel");

        let position = *self.position.borrow();
        match position {
            PanelPosition::Top => {
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Top, true);
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Left, true);
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Right, true);
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Bottom, false);
            }
            PanelPosition::Bottom => {
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Bottom, true);
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Left, true);
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Right, true);
                gtk4_layer_shell::set_anchor(&obj, gtk4_layer_shell::Edge::Top, false);
            }
        }
    }

    pub fn update_layout_priv(&self) {
        self.main_box.queue_resize();
        if let Some(da) = self.drawing_area.borrow().as_ref() {
            da.queue_draw();
        }
    }
    
    pub fn update_transparency_priv(&self) {
        let obj = self.obj();
        if self.transparency_enabled.get() {
            if let Some(visual) = obj.display().rgba_visual() { // Use obj.display()
                obj.set_visual(Some(&visual));
            } else {
                obj.set_visual(None); 
            }
        } else {
            obj.set_visual(None); 
        }
    }

    pub fn add_module_priv(&self, module: &impl glib::IsA<gtk::Widget>, position: ModulePosition, _order: i32) {
        match position {
            ModulePosition::Start => self.start_box.append(module.upcast_ref()),
            ModulePosition::Center => self.center_box.append(module.upcast_ref()),
            ModulePosition::End => self.end_box.append(module.upcast_ref()),
        }
        self.update_layout_priv();
    }

    pub fn remove_module_priv(&self, module: &impl glib::IsA<gtk::Widget>) {
        let widget = module.upcast_ref();
        if let Some(parent) = widget.parent() {
            if parent.is::<Box>() {
                let parent_box = parent.downcast_ref::<Box>().unwrap();
                parent_box.remove(widget);
                self.update_layout_priv();
            }
        }
    }
}
