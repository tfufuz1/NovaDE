// novade-ui/src/assistant_ui/widgets.rs
use iced::{Element, widget::{TextInput, Button, Column, Text, Row, Container, Space}, Length, Alignment};
use std::sync::Arc;
use super::controller::AssistantUIController;

#[derive(Debug, Clone)]
pub enum AssistantUIMessage {
    InputChanged(String),
    SubmitInput,
    ControllerUpdate(super::controller::AssistantUIUpdate), // Messages from controller
}

pub struct AssistantMainWidget {
    input_value: String,
    display_message: String,
    controller: Arc<AssistantUIController>,
    is_visible: bool, // Added to control visibility
}

impl AssistantMainWidget {
    pub fn new(controller: Arc<AssistantUIController>) -> Self {
        AssistantMainWidget {
            input_value: String::new(),
            display_message: "Welcome to Nova Assistant! Type 'open firefox' or 'tell me a joke'.".to_string(),
            controller,
            is_visible: false, // Start hidden
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
        if visible {
            self.display_message = "Welcome to Nova Assistant! Type 'open firefox' or 'tell me a joke'.".to_string();
            self.input_value.clear(); // Clear input when shown
        } else {
            // Optionally clear message when hiding, or let it persist
            // self.display_message = String::new();
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    // This update method is called by the main app's update loop
    pub fn update(&mut self, message: AssistantUIMessage) {
        match message {
            AssistantUIMessage::InputChanged(value) => {
                self.input_value = value;
            }
            AssistantUIMessage::SubmitInput => {
                if !self.input_value.is_empty() {
                    self.display_message = format!("Processing: {}...", self.input_value);
                    // The controller will send an AssistantUIUpdate message back via the subscription
                    self.controller.on_text_input_submit(self.input_value.clone());
                    // Input is cleared via ControllerUpdate::ClearInput for better UX
                }
            }
            AssistantUIMessage::ControllerUpdate(update) => {
                // Handle updates from the controller (received via app's subscription)
                match update {
                    super::controller::AssistantUIUpdate::DisplayProcessOutput(output) => {
                        match output {
                            novade_domain::ai_interaction_service::ProcessOutput::DirectResponse(text) => self.display_message = text,
                            novade_domain::ai_interaction_service::ProcessOutput::CommandToExecute(cmd) => {
                                // This state is transient; actual result comes via DisplayExecutionResult
                                self.display_message = format!("Attempting to execute: {}...", cmd.intent);
                            }
                            novade_domain::ai_interaction_service::ProcessOutput::NeedsClarification{query, ..} => self.display_message = format!("Clarification needed: {}", query),
                            novade_domain::ai_interaction_service::ProcessOutput::NoActionableIntent(text) => self.display_message = format!("Sorry, I couldn't understand that. (Details: {})", text),
                        }
                    }
                    super::controller::AssistantUIUpdate::DisplayExecutionResult(result) => {
                        self.display_message = result.message_to_user.unwrap_or_else(|| format!("Command execution success: {}", result.success));
                    }
                    super::controller::AssistantUIUpdate::DisplayError(err_msg) => {
                        self.display_message = format!("Error: {}", err_msg);
                    }
                    super::controller::AssistantUIUpdate::ClearInput => {
                        self.input_value.clear();
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<AssistantUIMessage> {
        if !self.is_visible {
            // If not visible, return an empty element that takes no space.
            // Using Space::with_height(Length::Fixed(0.0)) is one way.
            return Column::new().into();
        }

        let text_input = TextInput::new("Ask NovaDE...", &self.input_value)
            .on_input(AssistantUIMessage::InputChanged)
            .on_submit(AssistantUIMessage::SubmitInput)
            .padding(10);

        let submit_button = Button::new(Text::new("Submit"))
            .on_press(AssistantUIMessage::SubmitInput)
            .padding(10);

        let input_row = Row::new()
            .spacing(10)
            .align_items(Alignment::Center)
            .push(text_input)
            .push(submit_button);

        let display_text = Text::new(&self.display_message).size(16);

        let content_column = Column::new()
            .spacing(15)
            .padding(20)
            .align_items(Alignment::Start) // Align content to the start (left)
            .push(input_row)
            .push(display_text);

        // Wrap in a container to give it some bounds and basic styling
        Container::new(content_column)
            .width(Length::Fill) // Take full available width
            .height(Length::Shrink) // Adjust height to content
            .padding(15)
            .style(iced::theme::Container::Box) // Basic visible styling
            .into()
    }
}
