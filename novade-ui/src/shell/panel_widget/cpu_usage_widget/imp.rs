use gtk::{glib, subclass::prelude::*};
use gtk::{Label, Orientation};
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::mpsc;
use novade_domain::cpu_usage_service::{ICpuUsageService, SubscriptionId};
use novade_domain::error::DomainError;
use once_cell::sync::Lazy; // For static PROPERTIES
use glib::ParamSpec; // For ParamSpec types

// Define GObject properties
static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
    vec![
        glib::ParamSpecString::builder("label-format-string")
            .nick("Label Format String")
            .blurb("Format string for the CPU usage label. Use {usage} as placeholder.")
            .default_value(Some("CPU: {usage}%"))
            .flags(glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT)
            .build(),
    ]
});

// #[derive(Default)] // Cannot use Default with non-Default ParamSpec
pub struct CpuUsageWidgetImp {
    pub label: RefCell<Option<Label>>,
    pub cpu_usage_service: RefCell<Option<Arc<dyn ICpuUsageService>>>,
    pub update_receiver: RefCell<Option<mpsc::UnboundedReceiver<Result<f64, DomainError>>>>,
    pub subscription_id: RefCell<Option<SubscriptionId>>,
    pub glib_update_source_id: RefCell<Option<glib::SourceId>>,
    // New property field
    pub label_format_string: RefCell<String>,
    // Field to store last known usage for reformatting when format string changes
    last_known_percentage: RefCell<Option<f64>>,
}

impl Default for CpuUsageWidgetImp {
    fn default() -> Self {
        Self {
            label: RefCell::new(Some(Label::new(Some("CPU: --%")))), // Initial text
            cpu_usage_service: RefCell::new(None),
            update_receiver: RefCell::new(None),
            subscription_id: RefCell::new(None),
            glib_update_source_id: RefCell::new(None),
            label_format_string: RefCell::new("CPU: {usage}%".to_string()), // Default format
            last_known_percentage: RefCell::new(None),
        }
    }
}


#[glib::object_subclass]
impl ObjectSubclass for CpuUsageWidgetImp {
    const NAME: &'static str = "NovaDECpuUsageWidget";
    type Type = super::CpuUsageWidget;
    type ParentType = gtk::Box;

    // No need for new() if Default is sufficient and no custom construction logic needed here
    // fn new() -> Self { Self::default(); }


    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("cpuusagewidget");
        klass.install_properties(&PROPERTIES);
    }

    // fn instance_init(_obj: &glib::subclass::InitializingObject<Self>) {} // Not needed
}

impl ObjectImpl for CpuUsageWidgetImp {
    fn properties() -> &'static [ParamSpec] {
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            "label-format-string" => {
                let format_string = value.get().expect("Value must be a string for label-format-string");
                self.label_format_string.replace(format_string);
                // If we have a known percentage, update the label immediately
                if let Some(percentage) = *self.last_known_percentage.borrow() {
                    if let Some(label_widget) = self.label.borrow().as_ref() {
                        let format_str = self.label_format_string.borrow();
                        let display_text = format_str.replace("{usage}", &format!("{:.1}", percentage));
                        label_widget.set_text(&display_text);
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            "label-format-string" => self.label_format_string.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_orientation(Orientation::Horizontal);
        if let Some(label_widget) = self.label.borrow().as_ref() {
           // Initialize label with default format and placeholder
           let initial_format = self.label_format_string.borrow();
           let initial_text = initial_format.replace("{usage}", "--");
           label_widget.set_text(&initial_text);
           obj.append(label_widget);
        }
    }

    fn dispose(&self) {
        self.parent_dispose();
        if let Some(source_id) = self.glib_update_source_id.take() {
            source_id.remove();
        }
        if let (Some(service), Some(sub_id)) =
            (self.cpu_usage_service.borrow_mut().take(), self.subscription_id.borrow_mut().take()) {
            if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                runtime.spawn(async move {
                    if let Err(e) = service.unsubscribe_from_cpu_updates(sub_id).await {
                        tracing::error!("Error unsubscribing from CPU updates: {:?}", e);
                    }
                });
            } else {
                tracing::warn!("No Tokio runtime in dispose for CPU unsubscribe.");
            }
        }
    }
}

impl WidgetImpl for CpuUsageWidgetImp {}
impl BoxImpl for CpuUsageWidgetImp {}

impl CpuUsageWidgetImp {
    pub fn set_cpu_usage_service(&self, service: Arc<dyn ICpuUsageService>) {
        self.cpu_usage_service.replace(Some(service));
    }

    pub fn start_subscription_task(&self) {
        let widget_weak = self.obj().downgrade();
        if self.subscription_id.borrow().is_some() {
            tracing::warn!("CPU Usage Widget: Already subscribed.");
            return;
        }
        let service_arc = match &*self.cpu_usage_service.borrow() {
            Some(s) => s.clone(),
            None => {
                tracing::error!("CPU Usage Widget: CpuUsageService not set.");
                // Update label to show service error using current format string
                if let Some(widget) = widget_weak.upgrade() {
                   if let Some(label_widget) = widget.imp().label.borrow().as_ref() {
                       let format_str = widget.imp().label_format_string.borrow();
                       label_widget.set_text(&format_str.replace("{usage}", "Service N/A"));
                   }
                }
                return;
            }
        };
        let (tx, rx_for_glib_poll) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            match service_arc.subscribe_to_cpu_updates(tx).await {
                Ok(sub_id) => {
                    if let Some(widget) = widget_weak.upgrade() {
                        glib::idle_add_local_once(move || {
                            widget.imp().subscription_id.replace(Some(sub_id));
                            widget.imp().update_receiver.replace(Some(rx_for_glib_poll));
                            widget.imp().schedule_glib_updates_polling();
                        });
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to subscribe to CPU updates: {:?}", e);
                    if let Some(widget) = widget_weak.upgrade() {
                        glib::idle_add_local_once(move || {
                           if let Some(label_widget) = widget.imp().label.borrow().as_ref() {
                               let format_str = widget.imp().label_format_string.borrow();
                               label_widget.set_text(&format_str.replace("{usage}", "Sub Error"));
                           }
                        });
                    }
                }
            }
        });
    }

    fn schedule_glib_updates_polling(&self) {
       let widget_weak = self.obj().downgrade();
       let mut receiver = match self.update_receiver.borrow_mut().take() {
           Some(r) => r,
           None => { tracing::error!("schedule_glib_updates_polling: no update_receiver."); return; }
       };

       let source_id = glib::timeout_add_local(1000, move || {
           if let Some(widget) = widget_weak.upgrade() {
               let imp = widget.imp();
               match receiver.try_recv() {
                   Ok(Ok(percentage)) => {
                       imp.last_known_percentage.replace(Some(percentage)); // Store for reformatting
                       if let Some(label_widget) = imp.label.borrow().as_ref() {
                           let format_str = imp.label_format_string.borrow();
                           let display_text = format_str.replace("{usage}", &format!("{:.1}", percentage));
                           label_widget.set_text(&display_text);
                       }
                   }
                   Ok(Err(e)) => {
                       tracing::error!("Error in CPU update stream: {:?}", e);
                       if let Some(label_widget) = imp.label.borrow().as_ref() {
                           let format_str = imp.label_format_string.borrow();
                           label_widget.set_text(&format_str.replace("{usage}", "Err"));
                       }
                       imp.glib_update_source_id.replace(None);
                       return glib::ControlFlow::Break;
                   }
                   Err(mpsc::error::TryRecvError::Empty) => {}
                   Err(mpsc::error::TryRecvError::Disconnected) => {
                       tracing::warn!("CPU update channel disconnected.");
                       if let Some(label_widget) = imp.label.borrow().as_ref() {
                           let format_str = imp.label_format_string.borrow();
                           label_widget.set_text(&format_str.replace("{usage}", "Off"));
                       }
                       imp.glib_update_source_id.replace(None);
                       return glib::ControlFlow::Break;
                   }
               }
               glib::ControlFlow::Continue
           } else {
               tracing::info!("CPUUsageWidget dropped, stopping GLib updates.");
               glib::ControlFlow::Break
           }
       });
       self.glib_update_source_id.replace(Some(source_id));
    }
}
