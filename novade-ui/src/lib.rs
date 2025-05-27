//! UI layer for the NovaDE desktop environment.
//!
//! This crate provides the graphical user interface for the NovaDE desktop environment.
//! It builds upon the System Layer to create a cohesive and user-friendly desktop experience.

// Re-export key modules for convenience
pub mod error;
pub mod window_manager_ui;
pub mod desktop_ui;
pub mod panel_ui;
pub mod application_launcher;
pub mod settings_ui;
pub mod notification_ui;
pub mod theme_ui;
pub mod workspace_ui;
pub mod system_tray;
pub mod common;
pub mod widgets;
pub mod styles;
pub mod assets;
pub mod shell;
pub mod theming_gtk;

// Re-export key types for convenience
pub use error::UiError;
pub use window_manager_ui::WindowManagerUi;
pub use desktop_ui::DesktopUi;
pub use panel_ui::PanelUi;
pub use application_launcher::ApplicationLauncher;
pub use settings_ui::SettingsUi;
pub use notification_ui::NotificationUi;
pub use theme_ui::ThemeUi;
pub use workspace_ui::WorkspaceUi;
pub use system_tray::SystemTray;

use iced::{Application, Settings, window, Color};
use novade_system::SystemContext;
use std::sync::Arc;

/// The main NovaDE application.
pub struct NovaDE {
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
    /// The notification UI.
    notification_ui: NotificationUi,
    /// The theme UI.
    theme_ui: ThemeUi,
    /// The workspace UI.
    workspace_ui: WorkspaceUi,
    /// The system tray.
    system_tray: SystemTray,
}

impl NovaDE {
    /// Creates a new NovaDE application.
    ///
    /// # Arguments
    ///
    /// * `system_context` - The system context
    ///
    /// # Returns
    ///
    /// A new NovaDE application.
    pub fn new(system_context: SystemContext) -> Self {
        let system_context = Arc::new(system_context);
        
        let desktop_ui = DesktopUi::new(Arc::clone(&system_context));
        let panel_ui = PanelUi::new(Arc::clone(&system_context));
        let window_manager_ui = WindowManagerUi::new(Arc::clone(&system_context));
        let application_launcher = ApplicationLauncher::new(Arc::clone(&system_context));
        let settings_ui = SettingsUi::new(Arc::clone(&system_context));
        let notification_ui = NotificationUi::new(Arc::clone(&system_context));
        let theme_ui = ThemeUi::new(Arc::clone(&system_context));
        let workspace_ui = WorkspaceUi::new(Arc::clone(&system_context));
        let system_tray = SystemTray::new(Arc::clone(&system_context));
        
        NovaDE {
            system_context,
            desktop_ui,
            panel_ui,
            window_manager_ui,
            application_launcher,
            settings_ui,
            notification_ui,
            theme_ui,
            workspace_ui,
            system_tray,
        }
    }
    
    /// Runs the NovaDE application.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the application ran successfully, or an error if it failed.
    pub fn run() -> Result<(), iced::Error> {
        NovaDE::run_with_settings(Settings {
            window: window::Settings {
                size: (1920, 1080),
                position: window::Position::Centered,
                min_size: Some((800, 600)),
                max_size: None,
                resizable: true,
                decorations: false,
                transparent: true,
                always_on_top: false,
                icon: None,
            },
            default_font: None,
            default_text_size: 16,
            antialiasing: true,
            exit_on_close_request: true,
            ..Settings::default()
        })
    }
    
    /// Runs the NovaDE application with custom settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings
    ///
    /// # Returns
    ///
    /// `Ok(())` if the application ran successfully, or an error if it failed.
    pub fn run_with_settings(settings: Settings<()>) -> Result<(), iced::Error> {
        NovaDeApplication::run(settings)
    }
}

/// The NovaDE application implementation for iced.
struct NovaDeApplication {
    /// The NovaDE application.
    novade: Option<NovaDE>,
    /// The application state.
    state: ApplicationState,
}

/// The application state.
#[derive(Debug, Clone)]
enum ApplicationState {
    /// The application is loading.
    Loading,
    /// The application is loaded.
    Loaded,
    /// The application failed to load.
    Failed(String),
}

impl Application for NovaDeApplication {
    type Executor = iced::executor::Default;
    type Message = ApplicationMessage;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (
            NovaDeApplication {
                novade: None,
                state: ApplicationState::Loading,
            },
            iced::Command::perform(
                async {
                    // Initialize the system context
                    match novade_system::initialize().await {
                        Ok(system_context) => Ok(system_context),
                        Err(e) => Err(e.to_string()),
                    }
                },
                ApplicationMessage::Initialized,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("NovaDE Desktop Environment")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            ApplicationMessage::Initialized(Ok(system_context)) => {
                self.novade = Some(NovaDE::new(system_context));
                self.state = ApplicationState::Loaded;
                iced::Command::none()
            }
            ApplicationMessage::Initialized(Err(error)) => {
                self.state = ApplicationState::Failed(error);
                iced::Command::none()
            }
            ApplicationMessage::DesktopMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.desktop_ui.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::PanelMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.panel_ui.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::WindowManagerMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.window_manager_ui.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::LauncherMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.application_launcher.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::SettingsMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.settings_ui.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::NotificationMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.notification_ui.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::ThemeMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.theme_ui.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::WorkspaceMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.workspace_ui.update(msg)
                } else {
                    iced::Command::none()
                }
            }
            ApplicationMessage::SystemTrayMessage(msg) => {
                if let Some(novade) = &mut self.novade {
                    novade.system_tray.update(msg)
                } else {
                    iced::Command::none()
                }
            }
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        match &self.state {
            ApplicationState::Loading => {
                // Show loading screen
                iced::widget::container(
                    iced::widget::text("Loading NovaDE...")
                        .size(24)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .vertical_alignment(iced::alignment::Vertical::Center),
                )
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(LoadingContainerStyle)))
                .into()
            }
            ApplicationState::Loaded => {
                if let Some(novade) = &self.novade {
                    // Show the desktop
                    iced::widget::container(
                        iced::widget::column![
                            novade.panel_ui.view().map(ApplicationMessage::PanelMessage),
                            iced::widget::row![
                                novade.workspace_ui.view().map(ApplicationMessage::WorkspaceMessage),
                                novade.desktop_ui.view().map(ApplicationMessage::DesktopMessage),
                            ]
                            .width(iced::Length::Fill)
                            .height(iced::Length::Fill),
                            novade.notification_ui.view().map(ApplicationMessage::NotificationMessage),
                        ]
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill),
                    )
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(DesktopContainerStyle)))
                    .into()
                } else {
                    // This should never happen
                    iced::widget::text("Error: NovaDE not initialized").into()
                }
            }
            ApplicationState::Failed(error) => {
                // Show error screen
                iced::widget::container(
                    iced::widget::column![
                        iced::widget::text("Failed to initialize NovaDE")
                            .size(24)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                        iced::widget::text(error)
                            .size(16)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                    ]
                    .spacing(20)
                    .width(iced::Length::Fill)
                    .align_items(iced::Alignment::Center),
                )
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(ErrorContainerStyle)))
                .into()
            }
        }
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn background_color(&self) -> Color {
        match &self.state {
            ApplicationState::Loading => Color::from_rgb(0.1, 0.1, 0.1),
            ApplicationState::Loaded => Color::TRANSPARENT,
            ApplicationState::Failed(_) => Color::from_rgb(0.1, 0.0, 0.0),
        }
    }
}

/// The application message.
#[derive(Debug, Clone)]
enum ApplicationMessage {
    /// The application has been initialized.
    Initialized(Result<SystemContext, String>),
    /// A message for the desktop UI.
    DesktopMessage(desktop_ui::Message),
    /// A message for the panel UI.
    PanelMessage(panel_ui::Message),
    /// A message for the window manager UI.
    WindowManagerMessage(window_manager_ui::Message),
    /// A message for the application launcher.
    LauncherMessage(application_launcher::Message),
    /// A message for the settings UI.
    SettingsMessage(settings_ui::Message),
    /// A message for the notification UI.
    NotificationMessage(notification_ui::Message),
    /// A message for the theme UI.
    ThemeMessage(theme_ui::Message),
    /// A message for the workspace UI.
    WorkspaceMessage(workspace_ui::Message),
    /// A message for the system tray.
    SystemTrayMessage(system_tray::Message),
}

/// The loading container style.
struct LoadingContainerStyle;

impl iced::widget::container::StyleSheet for LoadingContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

/// The desktop container style.
struct DesktopContainerStyle;

impl iced::widget::container::StyleSheet for DesktopContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        }
    }
}

/// The error container style.
struct ErrorContainerStyle;

impl iced::widget::container::StyleSheet for ErrorContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.0, 0.0))),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}
