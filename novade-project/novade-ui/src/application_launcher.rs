//! Application launcher module for the NovaDE UI layer.
//!
//! This module provides the application launcher interface for the NovaDE desktop environment.

use iced::{Element, Length, Color, Background, alignment, Command};
use iced::widget::{Container, Text, Row, Column, Button, Image, Space, TextInput, Scrollable};
use std::sync::Arc;
use novade_system::SystemContext;
use crate::error::{UiError, UiResult};
use crate::styles::{ButtonStyle, ContainerStyle, TextInputStyle, ScrollableStyle};
use crate::assets::AssetManager;
use crate::common::{Grid, Card};

/// Application launcher message.
#[derive(Debug, Clone)]
pub enum Message {
    /// The search query was changed.
    SearchQueryChanged(String),
    /// An application was selected.
    ApplicationSelected(String),
    /// An application was launched.
    ApplicationLaunched(String),
    /// The launcher was closed.
    LauncherClosed,
    /// The application list was updated.
    ApplicationListUpdated(Vec<Application>),
    /// A category was selected.
    CategorySelected(String),
}

/// Application.
#[derive(Debug, Clone)]
pub struct Application {
    /// The application ID.
    id: String,
    /// The application name.
    name: String,
    /// The application description.
    description: String,
    /// The application icon path.
    icon_path: String,
    /// The application categories.
    categories: Vec<String>,
    /// The application command.
    command: String,
}

impl Application {
    /// Creates a new application.
    ///
    /// # Arguments
    ///
    /// * `id` - The application ID
    /// * `name` - The application name
    /// * `description` - The application description
    /// * `icon_path` - The application icon path
    /// * `categories` - The application categories
    /// * `command` - The application command
    ///
    /// # Returns
    ///
    /// A new application.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        icon_path: impl Into<String>,
        categories: Vec<String>,
        command: impl Into<String>,
    ) -> Self {
        Application {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            icon_path: icon_path.into(),
            categories,
            command: command.into(),
        }
    }
    
    /// Gets the application ID.
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Gets the application name.
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Gets the application description.
    pub fn description(&self) -> &str {
        &self.description
    }
    
    /// Gets the application icon path.
    pub fn icon_path(&self) -> &str {
        &self.icon_path
    }
    
    /// Gets the application categories.
    pub fn categories(&self) -> &[String] {
        &self.categories
    }
    
    /// Gets the application command.
    pub fn command(&self) -> &str {
        &self.command
    }
}

/// Application launcher.
pub struct ApplicationLauncher {
    /// The system context.
    system_context: Arc<SystemContext>,
    /// The asset manager.
    asset_manager: AssetManager,
    /// The search query.
    search_query: String,
    /// The applications.
    applications: Vec<Application>,
    /// The selected application ID.
    selected_application_id: Option<String>,
    /// The selected category.
    selected_category: Option<String>,
    /// Whether the launcher is visible.
    visible: bool,
}

impl ApplicationLauncher {
    /// Creates a new application launcher.
    ///
    /// # Arguments
    ///
    /// * `system_context` - The system context
    ///
    /// # Returns
    ///
    /// A new application launcher.
    pub fn new(system_context: Arc<SystemContext>) -> Self {
        let asset_manager = AssetManager::new();
        
        // In a real implementation, these would be loaded from the system
        let applications = vec![
            Application::new(
                "terminal",
                "Terminal",
                "A terminal emulator",
                "icons/terminal.svg",
                vec!["System".to_string(), "Utilities".to_string()],
                "xterm",
            ),
            Application::new(
                "browser",
                "Web Browser",
                "A web browser",
                "icons/web-browser.svg",
                vec!["Internet".to_string(), "Network".to_string()],
                "firefox",
            ),
            Application::new(
                "file-manager",
                "File Manager",
                "A file manager",
                "icons/file-manager.svg",
                vec!["System".to_string(), "Utilities".to_string()],
                "nautilus",
            ),
            Application::new(
                "text-editor",
                "Text Editor",
                "A text editor",
                "icons/text-editor.svg",
                vec!["Accessories".to_string(), "Utilities".to_string()],
                "gedit",
            ),
            Application::new(
                "image-viewer",
                "Image Viewer",
                "An image viewer",
                "icons/image-viewer.svg",
                vec!["Graphics".to_string(), "Utilities".to_string()],
                "eog",
            ),
            Application::new(
                "music-player",
                "Music Player",
                "A music player",
                "icons/music-player.svg",
                vec!["Multimedia".to_string(), "Audio".to_string()],
                "rhythmbox",
            ),
            Application::new(
                "video-player",
                "Video Player",
                "A video player",
                "icons/video-player.svg",
                vec!["Multimedia".to_string(), "Video".to_string()],
                "totem",
            ),
            Application::new(
                "calculator",
                "Calculator",
                "A calculator",
                "icons/calculator.svg",
                vec!["Accessories".to_string(), "Utilities".to_string()],
                "gnome-calculator",
            ),
        ];
        
        ApplicationLauncher {
            system_context,
            asset_manager,
            search_query: String::new(),
            applications,
            selected_application_id: None,
            selected_category: None,
            visible: false,
        }
    }
    
    /// Updates the application launcher.
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
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                
                Command::none()
            }
            Message::ApplicationSelected(id) => {
                self.selected_application_id = Some(id);
                
                Command::none()
            }
            Message::ApplicationLaunched(id) => {
                // In a real implementation, this would launch the application
                self.visible = false;
                
                Command::none()
            }
            Message::LauncherClosed => {
                self.visible = false;
                
                Command::none()
            }
            Message::ApplicationListUpdated(applications) => {
                self.applications = applications;
                
                Command::none()
            }
            Message::CategorySelected(category) => {
                if self.selected_category == Some(category.clone()) {
                    self.selected_category = None;
                } else {
                    self.selected_category = Some(category);
                }
                
                Command::none()
            }
        }
    }
    
    /// Shows the application launcher.
    pub fn show(&mut self) {
        self.visible = true;
        self.search_query = String::new();
        self.selected_application_id = None;
    }
    
    /// Hides the application launcher.
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    /// Checks if the application launcher is visible.
    ///
    /// # Returns
    ///
    /// `true` if the launcher is visible, `false` otherwise.
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Renders the application launcher.
    ///
    /// # Returns
    ///
    /// The application launcher element.
    pub fn view(&self) -> Element<Message> {
        if !self.visible {
            return Space::new(Length::Fill, Length::Fill).into();
        }
        
        let mut content = Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .spacing(20);
        
        // Search bar
        content = content.push(
            TextInput::new(
                "Search applications...",
                &self.search_query,
                Message::SearchQueryChanged,
            )
            .padding(10)
            .size(16)
            .width(Length::Fill)
            .style(TextInputStyle::Search)
        );
        
        // Categories
        content = content.push(self.view_categories());
        
        // Applications
        content = content.push(self.view_applications());
        
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerStyle::Card)
            .into()
    }
    
    /// Renders the categories.
    ///
    /// # Returns
    ///
    /// The categories element.
    fn view_categories(&self) -> Element<Message> {
        let categories = self.get_categories();
        
        let mut row = Row::new()
            .spacing(10)
            .padding(5)
            .width(Length::Fill);
        
        for category in categories {
            let is_selected = self.selected_category.as_ref() == Some(&category);
            
            row = row.push(
                Button::new(
                    Text::new(&category)
                        .size(14)
                )
                .style(if is_selected {
                    ButtonStyle::Primary
                } else {
                    ButtonStyle::Secondary
                })
                .on_press(Message::CategorySelected(category.clone()))
            );
        }
        
        row.into()
    }
    
    /// Renders the applications.
    ///
    /// # Returns
    ///
    /// The applications element.
    fn view_applications(&self) -> Element<Message> {
        let filtered_applications = self.get_filtered_applications();
        
        let grid = Grid::new(
            filtered_applications,
            |app, selected| {
                self.view_application(app, selected)
            },
            4,
        )
        .selected(None)
        .build(|index| {
            let app = &self.get_filtered_applications()[index];
            Message::ApplicationSelected(app.id().to_string())
        });
        
        Container::new(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerStyle::Default)
            .into()
    }
    
    /// Renders an application.
    ///
    /// # Arguments
    ///
    /// * `application` - The application to render
    /// * `selected` - Whether the application is selected
    ///
    /// # Returns
    ///
    /// The application element.
    fn view_application(&self, application: &Application, selected: bool) -> Element<Message> {
        // In a real implementation, this would load the icon from the file system
        // For now, we'll just use a placeholder
        let icon = self.asset_manager.get_placeholder_icon();
        
        let card = Card::new(
            Column::new()
                .width(Length::Fill)
                .spacing(10)
                .align_items(alignment::Alignment::Center)
                .push(
                    Image::new(icon)
                        .width(Length::Units(48))
                        .height(Length::Units(48))
                )
                .push(
                    Text::new(application.name())
                        .size(14)
                        .width(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                )
        )
        .selected(selected)
        .build(|| {
            Message::ApplicationLaunched(application.id().to_string())
        });
        
        Container::new(card)
            .width(Length::Fill)
            .padding(10)
            .style(ContainerStyle::Default)
            .into()
    }
    
    /// Gets the filtered applications.
    ///
    /// # Returns
    ///
    /// The filtered applications.
    fn get_filtered_applications(&self) -> Vec<&Application> {
        let mut filtered = self.applications.iter().collect::<Vec<_>>();
        
        // Filter by search query
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            filtered.retain(|app| {
                app.name().to_lowercase().contains(&query) ||
                app.description().to_lowercase().contains(&query)
            });
        }
        
        // Filter by category
        if let Some(category) = &self.selected_category {
            filtered.retain(|app| {
                app.categories().iter().any(|c| c == category)
            });
        }
        
        filtered
    }
    
    /// Gets the categories.
    ///
    /// # Returns
    ///
    /// The categories.
    fn get_categories(&self) -> Vec<String> {
        let mut categories = std::collections::HashSet::new();
        
        for app in &self.applications {
            for category in app.categories() {
                categories.insert(category.clone());
            }
        }
        
        let mut categories = categories.into_iter().collect::<Vec<_>>();
        categories.sort();
        
        categories
    }
}
