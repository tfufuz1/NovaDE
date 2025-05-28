use gtk::{glib, subclass::prelude::*};
use gtk::{Label, Orientation};
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::mpsc;
// Adjust use paths as necessary based on project structure
use novade_domain::cpu_usage_service::{ICpuUsageService, SubscriptionId};
use novade_domain::error::DomainError;
// once_cell is not used in this revised snippet, remove if not needed by other parts

#[derive(Default)]
pub struct CpuUsageWidgetImp {
    pub label: RefCell<Option<Label>>,
    pub cpu_usage_service: RefCell<Option<Arc<dyn ICpuUsageService>>>,
    pub update_receiver: RefCell<Option<mpsc::UnboundedReceiver<Result<f64, DomainError>>>>,
    pub subscription_id: RefCell<Option<SubscriptionId>>,
    pub glib_update_source_id: RefCell<Option<glib::SourceId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for CpuUsageWidgetImp {
    const NAME: &'static str = "NovaDECpuUsageWidget";
    type Type = super::CpuUsageWidget;
    type ParentType = gtk::Box;

    fn new() -> Self {
        Self {
            label: RefCell::new(Some(Label::new(Some("CPU: --%")))),
            cpu_usage_service: RefCell::new(None),
            update_receiver: RefCell::new(None),
            subscription_id: RefCell::new(None),
            glib_update_source_id: RefCell::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("cpuusagewidget");
    }

    fn instance_init(_obj: &glib::subclass::InitializingObject<Self>) {
        // No longer using obj.init_template() as we are not using CompositeTemplate
    }
}

impl ObjectImpl for CpuUsageWidgetImp {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_orientation(Orientation::Horizontal); // Set orientation for the Box
        if let Some(label_widget) = self.label.borrow().as_ref() {
           obj.append(label_widget);
        }
        // Service injection and subscription are handled by explicit calls to
        // set_cpu_usage_service() and start_subscription() respectively.
    }

    fn dispose(&self) {
        self.parent_dispose();
        if let Some(source_id) = self.glib_update_source_id.take() {
            source_id.remove();
        }

        if let (Some(service), Some(sub_id)) =
            (self.cpu_usage_service.borrow_mut().take(), self.subscription_id.borrow_mut().take()) {
            // Best effort to unsubscribe. Requires a Tokio runtime.
            if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                runtime.spawn(async move {
                    if let Err(e) = service.unsubscribe_from_cpu_updates(sub_id).await {
                        tracing::error!("Error unsubscribing from CPU updates: {:?}", e);
                    }
                });
            } else {
                tracing::warn!("No Tokio runtime in dispose for CPU unsubscribe. Subscription might not be cleaned up on server.");
            }
        }
    }
}

impl WidgetImpl for CpuUsageWidgetImp {}
impl BoxImpl for CpuUsageWidgetImp {}
// Removed OrientableImpl as gtk::Box already implements it.

impl CpuUsageWidgetImp {
    pub fn set_cpu_usage_service(&self, service: Arc<dyn ICpuUsageService>) {
        self.cpu_usage_service.replace(Some(service));
    }

    // Renamed from init_subscription to avoid ambiguity with GObject init
    pub fn start_subscription_task(&self) {
        let widget_weak = self.obj().downgrade(); // Use weak reference for async task

        if self.subscription_id.borrow().is_some() {
            tracing::warn!("CPU Usage Widget: Already subscribed or subscription attempt in progress.");
            return;
        }

        let service_arc = match &*self.cpu_usage_service.borrow() {
            Some(s) => s.clone(),
            None => {
                tracing::error!("CPU Usage Widget: CpuUsageService not set before start_subscription_task.");
                if let Some(widget) = widget_weak.upgrade() {
                   if let Some(label_widget) = widget.imp().label.borrow().as_ref() {
                       label_widget.set_text("CPU: Service N/A");
                   }
                }
                return;
            }
        };

        let (tx, mut rx_for_glib_poll) = mpsc::unbounded_channel();
        // Store the receiver immediately for the GLib polling task
        // self.update_receiver.replace(Some(rx_for_glib_poll)); // This should be done on UI thread after spawn.

        // Spawn a Tokio task to handle the subscription and receive messages
        tokio::spawn(async move {
            match service_arc.subscribe_to_cpu_updates(tx).await {
                Ok(sub_id) => {
                    if let Some(widget) = widget_weak.upgrade() {
                        // Schedule on GLib main thread to store sub_id and start polling
                        glib::idle_add_local_once(move || {
                            widget.imp().subscription_id.replace(Some(sub_id));
                            // Now that sub_id is stored, store receiver and start polling
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
                               label_widget.set_text("CPU: Sub Error");
                           }
                        });
                    }
                }
            }
        });
    }

    fn schedule_glib_updates_polling(&self) {
       let widget_weak = self.obj().downgrade();
       // Take the receiver to be moved into the closure
       let mut receiver = match self.update_receiver.borrow_mut().take() {
           Some(r) => r,
           None => {
               tracing::error!("schedule_glib_updates_polling called without an update_receiver.");
               return;
           }
       };

       let source_id = glib::timeout_add_local(1000, move || {
           if let Some(widget) = widget_weak.upgrade() {
               match receiver.try_recv() {
                   Ok(Ok(percentage)) => {
                       if let Some(label_widget) = widget.imp().label.borrow().as_ref() {
                           label_widget.set_text(&format!("CPU: {:.1}%", percentage));
                       }
                   }
                   Ok(Err(e)) => {
                       tracing::error!("Error in CPU update stream: {:?}", e);
                       if let Some(label_widget) = widget.imp().label.borrow().as_ref() {
                           label_widget.set_text("CPU: Err");
                       }
                       widget.imp().glib_update_source_id.replace(None);
                       return glib::ControlFlow::Break; // Stop polling on error
                   }
                   Err(mpsc::error::TryRecvError::Empty) => {
                       // No new data, continue polling
                   }
                   Err(mpsc::error::TryRecvError::Disconnected) => {
                       tracing::warn!("CPU update channel disconnected.");
                       if let Some(label_widget) = widget.imp().label.borrow().as_ref() {
                           label_widget.set_text("CPU: Off");
                       }
                       widget.imp().glib_update_source_id.replace(None);
                       return glib::ControlFlow::Break; // Stop polling
                   }
               }
               glib::ControlFlow::Continue
           } else {
               // Widget was dropped, stop polling
               tracing::info!("CPUUsageWidget dropped, stopping GLib updates.");
               glib::ControlFlow::Break
           }
       });
       self.glib_update_source_id.replace(Some(source_id));
    }
}
