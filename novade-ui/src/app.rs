// novade-ui/src/app.rs
use iced::{Application, Command, Element, Settings, Theme, executor, Subscription, alignment};
use iced::widget::{container, column, text, button, Space}; // Basic iced imports
use std::sync::Arc;
use tokio::sync::mpsc; // For communication between tokio tasks and Iced app

// Assuming assistant_ui is made public in lib.rs or mod.rs
use crate::assistant_ui::widgets::{AssistantMainWidget, AssistantUIMessage};
use crate::assistant_ui::controller::{AssistantUIController, AssistantUIUpdate, create_default_assistant_controller};

// Placeholder for other UI components and messages if they exist
// use crate::desktop_ui::{DesktopUi, Message as DesktopMessage};
// use crate::panel_ui::{PanelUi, Message as PanelMessage};
// ... and so on for other UI parts defined in the original app.rs

// For simplicity in this focused subtask, we'll stub out other UI parts.
struct DesktopUiStub;
impl DesktopUiStub { fn view(&self) -> Element<'static, Message> { text("Desktop Area").into() } }
struct PanelUiStub;
impl PanelUiStub { fn view(&self) -> Element<'static, Message> { text("Panel Area").into() } }


pub struct NovaDeApp {
    assistant_widget: AssistantMainWidget,
    // The controller is not stored directly in NovaDeApp state typically.
    // Its methods are called, and it communicates back via messages through the subscription.
    // The Arc<AssistantUIController> might be passed to UI elements if they need to call its methods directly
    // (but usually, they send messages that `update` then uses to call the controller).
    // For this prototype, the widget holds an Arc to the controller.

    // Channel to receive updates from AssistantUIController's async tasks
    // The receiver part is consumed by the subscription.
    // The sender part was given to the controller.
    _assistant_result_sender: mpsc::Sender<AssistantUIUpdate>, // Keep sender to avoid dropping channel immediately
    assistant_result_receiver: Option<mpsc::Receiver<AssistantUIUpdate>>, // Option to allow take() for subscription
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleAssistant,
    AssistantGUIMessage(AssistantUIMessage), // Renamed to avoid conflict if AssistantUIMessage was top-level
    AssistantControllerUpdate(AssistantUIUpdate), // Message carrying result from controller's async task
    Loaded, // Placeholder for app loaded event
    None, // No-op message
}

impl Application for NovaDeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (sender, receiver) = mpsc::channel(100);
        let assistant_controller = Arc::new(create_default_assistant_controller(sender.clone()));
        
        (
            NovaDeApp {
                assistant_widget: AssistantMainWidget::new(assistant_controller),
                _assistant_result_sender: sender,
                assistant_result_receiver: Some(receiver), // Store receiver to be used in subscription
            },
            Command::perform(async {}, |_| Message::Loaded), // Simulate app loaded
        )
    }

    fn title(&self) -> String {
        String::from("NovaDE Prototype with Assistant")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded => {
                println!("[App] NovaDeApp loaded.");
            }
            Message::ToggleAssistant => {
                println!("[App] Toggling Assistant Visibility.");
                self.assistant_widget.set_visible(!self.assistant_widget.is_visible());
            }
            Message::AssistantGUIMessage(asst_msg) => {
                // This message comes from the assistant's view elements (e.g., TextInput, Button)
                self.assistant_widget.update(asst_msg);
            }
            Message::AssistantControllerUpdate(update) => {
                // This message comes from the subscription listening to AssistantUIController results
                println!("[App] Received controller update: {:?}", update);
                self.assistant_widget.update(AssistantUIMessage::ControllerUpdate(update));
            }
            Message::None => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Consume the receiver for the subscription. This can only be done once.
        // If the app is reloaded (e.g. hot reload), this setup might need adjustment.
        // For a typical app lifecycle, it's fine.
        if let Some(receiver) = std::mem::take(&mut self.assistant_result_receiver) {
             struct AssistantEventsSubscription; // Unique ID for the subscription
             iced::subscription::channel(
                 std::any::TypeId::of::<AssistantEventsSubscription>(),
                 100, // Buffer size for the channel within Iced
                 |mut output| async move {
                     let mut local_receiver = receiver; // Move receiver into the async block
                     loop {
                         match local_receiver.recv().await {
                             Some(update) => {
                                 // Send the received update to the Iced application's update method
                                 if output.send(Message::AssistantControllerUpdate(update)).await.is_err() {
                                     eprintln!("[Subscription] Failed to send controller update to app. Channel closed.");
                                     break;
                                 }
                             }
                             None => {
                                 // Channel closed
                                 eprintln!("[Subscription] AssistantUIController result channel closed.");
                                 break;
                             }
                         }
                     }
                 },
             )
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
        let main_content = column![
            button(text("Toggle Assistant (Show/Hide)")).on_press(Message::ToggleAssistant),
            Space::with_height(iced::Length::Units(20)), // Use fixed units instead of Fill for Space
            text("NovaDE Main Content Area (Placeholder)"),
            // Other main UI elements would go here
            // Example: DesktopUiStub.view().map(|_| Message::None), // Map stub messages to None
            // Example: PanelUiStub.view().map(|_| Message::None),
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill)
        .align_items(alignment::Alignment::Center);

        let assistant_view_element = self.assistant_widget.view().map(Message::AssistantGUIMessage);

        // Layout: Main content on top, assistant UI below it.
        let full_layout = column![
            main_content,
            assistant_view_element, // This will be empty if assistant is not visible
        ]
        .width(Length::Fill)
        .align_items(alignment::Alignment::Center);

        container(full_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark // Or Theme::Light, or a custom theme
    }
}

// The main function to run the app (usually in main.rs, but included here for completeness of the example)
// pub fn main() -> iced::Result {
//     NovaDeApp::run(Settings {
//         // ... any custom settings ...
//         ..Settings::default()
//     })
// }
