//! Settings UI module for the NovaDE UI layer.
//!
//! This module provides the settings interface for the NovaDE desktop environment.

use iced::{Element, Length, Color, Background, alignment, Command};
use iced::widget::{Container, Text, Row, Column, Button, Image, Space, Checkbox, Radio, Slider, PickList};
use std::sync::Arc;
use novade_system::SystemContext;
use novade_domain::settings::core::{Settings, SettingsCategory};
use novade_domain::theming::core::{Theme, ThemeId};
use crate::error::{UiError, UiResult};
use crate::styles::{ButtonStyle, ContainerStyle};
use crate::assets::AssetManager;
use crate::common::{Section, Dialog};

/// Settings UI message.
#[derive(Debug, Clone)]
pub enum Message {
    /// The settings dialog was opened.
    SettingsOpened,
    /// The settings dialog was closed.
    SettingsClosed,
    /// A settings category was selected.
    CategorySelected(SettingsCategory),
    /// A theme was selected.
    ThemeSelected(ThemeId),
    /// The dark mode was toggled.
    DarkModeToggled(bool),
    /// The accent color was changed.
    AccentColorChanged(String),
    /// The font size was changed.
    FontSizeChanged(u32),
    /// The settings were saved.
    SettingsSaved,
    /// The settings were reset.
    SettingsReset,
    /// The settings were loaded.
    SettingsLoaded(Settings),
}

/// Settings UI.
pub struct SettingsUi {
    /// The system context.
    system_context: Arc<SystemContext>,
    /// The asset manager.
    asset_manager: AssetManager,
    /// The current settings.
    settings: Settings,
    /// The selected category.
    selected_category: SettingsCategory,
    /// Whether the settings dialog is visible.
    visible: bool,
    /// The available themes.
    available_themes: Vec<Theme>,
}

impl SettingsUi {
    /// Creates a new settings UI.
    ///
    /// # Arguments
    ///
    /// * `system_context` - The system context
    ///
    /// # Returns
    ///
    /// A new settings UI.
    pub fn new(system_context: Arc<SystemContext>) -> Self {
        let asset_manager = AssetManager::new();

        // In a real implementation, these would be loaded from the system
        let settings = Settings::default();

        // In a real implementation, these would be loaded from the system
        let available_themes = vec![
            Theme::new(
                ThemeId::new(),
                "Default Light",
                "Default light theme",
                "NovaDE Team",
                "1.0.0",
            ),
            Theme::new(
                ThemeId::new(),
                "Default Dark",
                "Default dark theme",
                "NovaDE Team",
                "1.0.0",
            ),
            Theme::new(
                ThemeId::new(),
                "Nord",
                "Nord theme",
                "NovaDE Team",
                "1.0.0",
            ),
            Theme::new(
                ThemeId::new(),
                "Solarized",
                "Solarized theme",
                "NovaDE Team",
                "1.0.0",
            ),
        ];

        SettingsUi {
            system_context,
            asset_manager,
            settings,
            selected_category: SettingsCategory::Appearance,
            visible: false,
            available_themes,
        }
    }

    /// Updates the settings UI.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A command to be processed.
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SettingsOpened => {
                self.visible = true;

                Command::none()
            }
            Message::SettingsClosed => {
                self.visible = false;

                Command::none()
            }
            Message::CategorySelected(category) => {
                self.selected_category = category;

                Command::none()
            }
            Message::ThemeSelected(theme_id) => {
                self.settings.appearance.theme_id = theme_id;

                Command::none()
            }
            Message::DarkModeToggled(enabled) => {
                self.settings.appearance.dark_mode = enabled;

                Command::none()
            }
            Message::AccentColorChanged(color) => {
                self.settings.appearance.accent_color = color;

                Command::none()
            }
            Message::FontSizeChanged(size) => {
                self.settings.appearance.font_size = size;

                Command::none()
            }
            Message::SettingsSaved => {
                // In a real implementation, this would save the settings
                self.visible = false;

                Command::none()
            }
            Message::SettingsReset => {
                // In a real implementation, this would reset the settings
                self.settings = Settings::default();

                Command::none()
            }
            Message::SettingsLoaded(settings) => {
                self.settings = settings;

                Command::none()
            }
        }
    }

    /// Shows the settings dialog.
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hides the settings dialog.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Checks if the settings dialog is visible.
    ///
    /// # Returns
    ///
    /// `true` if the dialog is visible, `false` otherwise.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Renders the settings UI.
    ///
    /// # Returns
    ///
    /// The settings UI element.
    pub fn view(&self) -> Element<Message> {
        if !self.visible {
            return Space::new(Length::Fill, Length::Fill).into();
        }

        let mut content = Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .spacing(20);

        // Categories
        content = content.push(self.view_categories());

        // Settings
        content = content.push(self.view_settings());

        Dialog::new(
            "Settings",
            content,
        )
        .primary_button("Save")
        .secondary_button("Reset")
        .build(
            || Message::SettingsSaved,
            || Message::SettingsReset,
            || Message::SettingsClosed,
        )
    }

    /// Renders the categories.
    ///
    /// # Returns
    ///
    /// The categories element.
    fn view_categories(&self) -> Element<Message> {
        let mut column = Column::new()
            .width(Length::Units(200))
            .spacing(10);

        // Add category buttons
        for category in self.get_categories() {
            let is_selected = self.selected_category == category;

            column = column.push(
                Button::new(
                    Text::new(self.get_category_name(&category))
                        .size(14)
                )
                .width(Length::Fill)
                .style(if is_selected {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .on_press(Message::CategorySelected(category))
            );
        }

        Container::new(column)
            .width(Length::Units(200))
            .height(Length::Fill)
            .style(ContainerStyle::Card)
            .into()
    }

    /// Renders the settings.
    ///
    /// # Returns
    ///
    /// The settings element.
    fn view_settings(&self) -> Element<Message> {
        let content = match self.selected_category {
            SettingsCategory::Appearance => self.view_appearance_settings(),
            SettingsCategory::Desktop => self.view_desktop_settings(),
            SettingsCategory::Windows => self.view_windows_settings(),
            SettingsCategory::Input => self.view_input_settings(),
            SettingsCategory::Notifications => self.view_notifications_settings(),
            SettingsCategory::Power => self.view_power_settings(),
            SettingsCategory::Network => self.view_network_settings(),
            SettingsCategory::Sound => self.view_sound_settings(),
            SettingsCategory::Privacy => self.view_privacy_settings(),
            SettingsCategory::About => self.view_about_settings(),
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerStyle::Card)
            .into()
    }

    /// Renders the appearance settings.
    ///
    /// # Returns
    ///
    /// The appearance settings element.
    fn view_appearance_settings(&self) -> Element<Message> {
        let mut column = Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20);

        // Theme
        let theme_section = Section::new(
            "Theme",
            Column::new()
                .spacing(10)
                .push(
                    Text::new("Select a theme:")
                        .size(14)
                )
                .push(
                    // In a real implementation, this would be a proper pick list
                    // For now, we'll just use a button
                    Button::new(
                        Text::new("Default Theme")
                            .size(14)
                    )
                    .width(Length::Fill)
                    .style(ButtonStyle::Secondary)
                    .on_press(Message::ThemeSelected(ThemeId::new()))
                )
        )
        .collapsible(true)
        .build(|| Message::CategorySelected(SettingsCategory::Appearance));

        column = column.push(theme_section);

        // Dark mode
        let dark_mode_section = Section::new(
            "Dark Mode",
            Column::new()
                .spacing(10)
                .push(
                    Row::new()
                        .spacing(10)
                        .align_items(alignment::Alignment::Center)
                        .push(
                            Checkbox::new(
                                self.settings.appearance.dark_mode,
                                "Enable dark mode",
                                Message::DarkModeToggled,
                            )
                        )
                )
        )
        .collapsible(true)
        .build(|| Message::CategorySelected(SettingsCategory::Appearance));

        column = column.push(dark_mode_section);

        // Font size
        let font_size_section = Section::new(
            "Font Size",
            Column::new()
                .spacing(10)
                .push(
                    Text::new(format!("Font size: {}", self.settings.appearance.font_size))
                        .size(14)
                )
                .push(
                    // In a real implementation, this would be a proper slider
                    // For now, we'll just use buttons
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(
                                Text::new("-")
                                    .size(14)
                            )
                            .style(ButtonStyle::Secondary)
                            .on_press(Message::FontSizeChanged(
                                (self.settings.appearance.font_size - 1).max(8)
                            ))
                        )
                        .push(
                            Button::new(
                                Text::new("+")
                                    .size(14)
                            )
                            .style(ButtonStyle::Secondary)
                            .on_press(Message::FontSizeChanged(
                                (self.settings.appearance.font_size + 1).min(24)
                            ))
                        )
                )
        )
        .collapsible(true)
        .build(|| Message::CategorySelected(SettingsCategory::Appearance));

        column = column.push(font_size_section);

        column.into()
    }

    /// Renders the desktop settings.
    ///
    /// # Returns
    ///
    /// The desktop settings element.
    fn view_desktop_settings(&self) -> Element<Message> {
        // In a real implementation, this would show desktop settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Desktop Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure desktop appearance and behavior.")
                    .size(14)
            )
            .into()
    }

    /// Renders the windows settings.
    ///
    /// # Returns
    ///
    /// The windows settings element.
    fn view_windows_settings(&self) -> Element<Message> {
        // In a real implementation, this would show windows settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Windows Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure window behavior and appearance.")
                    .size(14)
            )
            .into()
    }

    /// Renders the input settings.
    ///
    /// # Returns
    ///
    /// The input settings element.
    fn view_input_settings(&self) -> Element<Message> {
        // In a real implementation, this would show input settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Input Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure keyboard, mouse, and touch input.")
                    .size(14)
            )
            .into()
    }

    /// Renders the notifications settings.
    ///
    /// # Returns
    ///
    /// The notifications settings element.
    fn view_notifications_settings(&self) -> Element<Message> {
        // In a real implementation, this would show notifications settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Notifications Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure notification behavior and appearance.")
                    .size(14)
            )
            .into()
    }

    /// Renders the power settings.
    ///
    /// # Returns
    ///
    /// The power settings element.
    fn view_power_settings(&self) -> Element<Message> {
        // In a real implementation, this would show power settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Power Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure power management and battery settings.")
                    .size(14)
            )
            .into()
    }

    /// Renders the network settings.
    ///
    /// # Returns
    ///
    /// The network settings element.
    fn view_network_settings(&self) -> Element<Message> {
        // In a real implementation, this would show network settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Network Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure network connections and settings.")
                    .size(14)
            )
            .into()
    }

    /// Renders the sound settings.
    ///
    /// # Returns
    ///
    /// The sound settings element.
    fn view_sound_settings(&self) -> Element<Message> {
        // In a real implementation, this would show sound settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Sound Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure sound devices and volume levels.")
                    .size(14)
            )
            .into()
    }

    /// Renders the privacy settings.
    ///
    /// # Returns
    ///
    /// The privacy settings element.
    fn view_privacy_settings(&self) -> Element<Message> {
        // In a real implementation, this would show privacy settings
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("Privacy Settings")
                    .size(18)
            )
            .push(
                Text::new("Configure privacy and security settings.")
                    .size(14)
            )
            .into()
    }

    /// Renders the about settings.
    ///
    /// # Returns
    ///
    /// The about settings element.
    fn view_about_settings(&self) -> Element<Message> {
        // In a real implementation, this would show about information
        // For now, we'll just show a placeholder

        Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20)
            .push(
                Text::new("About NovaDE")
                    .size(18)
            )
            .push(
                Text::new("NovaDE Desktop Environment")
                    .size(14)
            )
            .push(
                Text::new("Version 1.0.0")
                    .size(14)
            )
            .push(
                Text::new("© 2025 NovaDE Team")
                    .size(14)
            )
            .into()
    }

    /// Gets the categories.
    ///
    /// # Returns
    ///
    /// The categories.
    fn get_categories(&self) -> Vec<SettingsCategory> {
        vec![
            SettingsCategory::Appearance,
            SettingsCategory::Desktop,
            SettingsCategory::Windows,
            SettingsCategory::Input,
            SettingsCategory::Notifications,
            SettingsCategory::Power,
            SettingsCategory::Network,
            SettingsCategory::Sound,
            SettingsCategory::Privacy,
            SettingsCategory::About,
        ]
    }

    /// Gets the category name.
    ///
    /// # Arguments
    ///
    /// * `category` - The category
    ///
    /// # Returns
    ///
    /// The category name.
    fn get_category_name(&self, category: &SettingsCategory) -> &'static str {
        match category {
            SettingsCategory::Appearance => "Appearance",
            SettingsCategory::Desktop => "Desktop",
            SettingsCategory::Windows => "Windows",
            SettingsCategory::Input => "Input",
            SettingsCategory::Notifications => "Notifications",
            SettingsCategory::Power => "Power",
            SettingsCategory::Network => "Network",
            SettingsCategory::Sound => "Sound",
            SettingsCategory::Privacy => "Privacy",
            SettingsCategory::About => "About",
        }
    }
}
