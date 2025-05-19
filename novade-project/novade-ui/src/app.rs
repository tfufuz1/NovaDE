//! Main application module for the NovaDE UI layer.
//!
//! This module provides the main application for the NovaDE desktop environment.

use iced::{Application, Command, Element, Settings, Subscription, executor, window, Background, Color};
use iced::widget::{Container, Column, Row, Space};
use std::sync::Arc;
use novade_system::SystemContext;
use crate::error::{UiError, UiResult};
use crate::styles::ContainerStyle;
use crate::desktop_ui::{DesktopUi, Message as DesktopMessage};
use crate::panel_ui::{PanelUi, Message as PanelMessage};
use crate::window_manager_ui::{WindowManagerUi, Message as WindowManagerMessage};
use crate::application_launcher::{ApplicationLauncher, Message as LauncherMessage};
use crate::settings_ui::{SettingsUi, Message as SettingsMessage};

/// Main application message.
#[derive(Debug, Clone)]
pub enum Message {
    /// A desktop message.
    Desktop(DesktopMessage),
    /// A panel message.
    Panel(PanelMessage),
    /// A window manager message.
    WindowManager(WindowManagerMessage),
    /// An application launcher message.
    Launcher(LauncherMessage),
    /// A settings message.
    Settings(SettingsMessage),
    /// The application was initialized.
    Initialized,
    /// The application should exit.
    Exit,
}

/// Main application.
pub struct NovaDeApp {
    /// The system context.
    system_context: Arc<SystemContext>,
    /// The desktop UI.
    desktop_ui: DesktopUi,
    /// The panel UI.
    panel_ui: PanelUi,
    /// The window manager UI.
    window_manager_ui: WindowManagerUi,
    /// The application launcher.
    application_launcher: ApplicationLauncher,
    /// The settings UI.
    settings_ui: SettingsUi,
}

impl Application for NovaDeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    /// Creates a new application.
    ///
    /// # Arguments
    ///
    /// * `flags` - The application flags
    ///
    /// # Returns
    ///
    /// A new application.
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Initialize system context
        let system_context = Arc::new(SystemContext::new());
        
        // Initialize UI components
        let desktop_ui = DesktopUi::new(system_context.clone());
        let panel_ui = PanelUi::new(system_context.clone());
        let window_manager_ui = WindowManagerUi::new(system_context.clone());
        let application_launcher = ApplicationLauncher::new(system_context.clone());
        let settings_ui = SettingsUi::new(system_context.clone());
        
        let app = NovaDeApp {
            system_context,
            desktop_ui,
            panel_ui,
            window_manager_ui,
            application_launcher,
            settings_ui,
        };
        
        // Initialize the application
        let command = Command::perform(
            async { },
            |_| Message::Initialized
        );
        
        (app, command)
    }

    /// Gets the application title.
    ///
    /// # Returns
    ///
    /// The application title.
    fn title(&self) -> String {
        String::from("NovaDE Desktop Environment")
    }

    /// Updates the application.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A command to be processed.
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Desktop(msg) => {
                self.desktop_ui.update(msg)
                    .map(Message::Desktop)
            }
            Message::Panel(msg) => {
                self.panel_ui.update(msg)
                    .map(Message::Panel)
            }
            Message::WindowManager(msg) => {
                self.window_manager_ui.update(msg)
                    .map(Message::WindowManager)
            }
            Message::Launcher(msg) => {
                self.application_launcher.update(msg)
                    .map(Message::Launcher)
            }
            Message::Settings(msg) => {
                self.settings_ui.update(msg)
                    .map(Message::Settings)
            }
            Message::Initialized => {
                // Start subscriptions
                Command::batch(vec![
                    // Start clock subscription
                    PanelUi::start_clock_subscription()
                        .map(Message::Panel),
                    
                    // Start battery subscription
                    PanelUi::start_battery_subscription(self.system_context.power_manager.clone())
                        .map(Message::Panel),
                    
                    // Start network subscription
                    PanelUi::start_network_subscription(self.system_context.network_manager.clone())
                        .map(Message::Panel),
                    
                    // Start window list subscription
                    WindowManagerUi::start_window_list_subscription(self.system_context.window_manager.clone())
                        .map(Message::WindowManager),
                ])
            }
            Message::Exit => {
                // Exit the application
                window::close()
            }
        }
    }

    /// Renders the application.
    ///
    /// # Returns
    ///
    /// The application element.
    fn view(&self) -> Element<Message> {
        // Create the main column
        let mut content = Column::new()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);
        
        // Add the desktop
        content = content.push(
            self.desktop_ui.view()
                .map(Message::Desktop)
        );
        
        // Add overlays
        let overlays = Column::new()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);
        
        // Add the window manager
        let overlays = overlays.push(
            self.window_manager_ui.view()
                .map(Message::WindowManager)
        );
        
        // Add the application launcher
        let overlays = overlays.push(
            self.application_launcher.view()
                .map(Message::Launcher)
        );
        
        // Add the settings
        let overlays = overlays.push(
            self.settings_ui.view()
                .map(Message::Settings)
        );
        
        // Add the panel
        let panel = self.panel_ui.view()
            .map(Message::Panel);
        
        // Combine everything
        Container::new(
            Column::new()
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .push(content)
                .push(
                    Container::new(overlays)
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .style(ContainerStyle::Default)
                )
                .push(panel)
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(ContainerStyle::Default)
        .into()
    }

    /// Gets the application subscriptions.
    ///
    /// # Returns
    ///
    /// The application subscriptions.
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

/// Runs the NovaDE application.
///
/// # Returns
///
/// `Ok(())` if the application ran successfully, or an error if it failed.
pub fn run() -> UiResult<()> {
    let settings = Settings {
        window: window::Settings {
            size: (1280, 720),
            position: window::Position::Centered,
            min_size: Some((800, 600)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
        },
        default_font: None,
        default_text_size: 16,
        antialiasing: true,
        exit_on_close_request: true,
        ..Default::default()
    };
    
    NovaDeApp::run(settings)
        .map_err(|err| UiError::new(format!("Failed to run application: {}", err)))
}
