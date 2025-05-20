//! Common UI components and utilities for the NovaDE UI layer.
//!
//! This module provides common UI components and utilities that are used across
//! multiple UI modules.

use iced::{Element, Length, Color, Background, alignment};
use iced::widget::{Container, Text, Row, Column, Button, Scrollable};
use crate::error::{UiError, UiResult};
use crate::styles::{ButtonStyle, ContainerStyle};

/// A standard header component.
pub struct Header {
    /// The title text.
    title: String,
    /// Whether to show a back button.
    show_back_button: bool,
    /// Whether to show a close button.
    show_close_button: bool,
}

impl Header {
    /// Creates a new header.
    ///
    /// # Arguments
    ///
    /// * `title` - The title text
    ///
    /// # Returns
    ///
    /// A new header.
    pub fn new(title: impl Into<String>) -> Self {
        Header {
            title: title.into(),
            show_back_button: false,
            show_close_button: true,
        }
    }
    
    /// Sets whether to show a back button.
    ///
    /// # Arguments
    ///
    /// * `show` - Whether to show a back button
    ///
    /// # Returns
    ///
    /// The updated header.
    pub fn show_back_button(mut self, show: bool) -> Self {
        self.show_back_button = show;
        self
    }
    
    /// Sets whether to show a close button.
    ///
    /// # Arguments
    ///
    /// * `show` - Whether to show a close button
    ///
    /// # Returns
    ///
    /// The updated header.
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }
    
    /// Builds the header element.
    ///
    /// # Type Parameters
    ///
    /// * `Message` - The message type
    ///
    /// # Arguments
    ///
    /// * `on_back` - The callback for the back button
    /// * `on_close` - The callback for the close button
    ///
    /// # Returns
    ///
    /// The header element.
    pub fn build<Message>(
        &self,
        on_back: Option<Message>,
        on_close: Option<Message>,
    ) -> Element<Message>
    where
        Message: Clone + 'static,
    {
        let mut row = Row::new()
            .width(Length::Fill)
            .align_items(alignment::Alignment::Center)
            .padding(10);
        
        if self.show_back_button {
            if let Some(on_back) = on_back.clone() {
                row = row.push(
                    Button::new(Text::new("←").size(20))
                        .on_press(on_back)
                        .style(ButtonStyle::Icon)
                        .padding(5),
                );
            } else {
                row = row.push(
                    Button::new(Text::new("←").size(20))
                        .style(ButtonStyle::Icon)
                        .padding(5),
                );
            }
        }
        
        row = row.push(
            Text::new(&self.title)
                .size(18)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center),
        );
        
        if self.show_close_button {
            if let Some(on_close) = on_close {
                row = row.push(
                    Button::new(Text::new("×").size(20))
                        .on_press(on_close)
                        .style(ButtonStyle::Icon)
                        .padding(5),
                );
            } else {
                row = row.push(
                    Button::new(Text::new("×").size(20))
                        .style(ButtonStyle::Icon)
                        .padding(5),
                );
            }
        }
        
        Container::new(row)
            .style(ContainerStyle::Header)
            .width(Length::Fill)
            .into()
    }
}

/// A standard card component.
pub struct Card {
    /// The title text.
    title: Option<String>,
    /// The content.
    content: Element<'static, CardMessage>,
    /// Whether the card is selectable.
    selectable: bool,
    /// Whether the card is selected.
    selected: bool,
}

/// Card message.
#[derive(Debug, Clone)]
pub enum CardMessage {
    /// The card was selected.
    Selected,
    /// A content message.
    Content(Box<dyn std::any::Any>),
}

impl Card {
    /// Creates a new card.
    ///
    /// # Arguments
    ///
    /// * `content` - The card content
    ///
    /// # Returns
    ///
    /// A new card.
    pub fn new<E, Message>(content: E) -> Self
    where
        E: Into<Element<'static, Message>>,
        Message: 'static,
    {
        Card {
            title: None,
            content: content.into().map(|msg| {
                CardMessage::Content(Box::new(msg))
            }),
            selectable: false,
            selected: false,
        }
    }
    
    /// Sets the card title.
    ///
    /// # Arguments
    ///
    /// * `title` - The title text
    ///
    /// # Returns
    ///
    /// The updated card.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    /// Sets whether the card is selectable.
    ///
    /// # Arguments
    ///
    /// * `selectable` - Whether the card is selectable
    ///
    /// # Returns
    ///
    /// The updated card.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
    
    /// Sets whether the card is selected.
    ///
    /// # Arguments
    ///
    /// * `selected` - Whether the card is selected
    ///
    /// # Returns
    ///
    /// The updated card.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    
    /// Builds the card element.
    ///
    /// # Type Parameters
    ///
    /// * `Message` - The message type
    ///
    /// # Arguments
    ///
    /// * `on_select` - The callback for card selection
    ///
    /// # Returns
    ///
    /// The card element.
    pub fn build<Message>(
        self,
        on_select: impl Fn() -> Message + 'static,
    ) -> Element<Message>
    where
        Message: 'static,
    {
        let mut column = Column::new()
            .width(Length::Fill)
            .spacing(10)
            .padding(15);
        
        if let Some(title) = self.title {
            column = column.push(
                Text::new(title)
                    .size(16)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            );
        }
        
        column = column.push(self.content.map(move |msg| {
            match msg {
                CardMessage::Selected => on_select(),
                CardMessage::Content(content) => {
                    // This is a simplification; in a real implementation,
                    // you would need to handle the content message appropriately
                    on_select()
                }
            }
        }));
        
        let container = Container::new(column)
            .width(Length::Fill)
            .style(if self.selected {
                ContainerStyle::SelectedCard
            } else {
                ContainerStyle::Card
            });
        
        if self.selectable {
            container
                .on_press(CardMessage::Selected)
                .into()
        } else {
            container.into()
        }
    }
}

/// A standard section component.
pub struct Section {
    /// The title text.
    title: String,
    /// The content.
    content: Element<'static, SectionMessage>,
    /// Whether the section is collapsible.
    collapsible: bool,
    /// Whether the section is collapsed.
    collapsed: bool,
}

/// Section message.
#[derive(Debug, Clone)]
pub enum SectionMessage {
    /// The section was toggled.
    Toggled,
    /// A content message.
    Content(Box<dyn std::any::Any>),
}

impl Section {
    /// Creates a new section.
    ///
    /// # Arguments
    ///
    /// * `title` - The title text
    /// * `content` - The section content
    ///
    /// # Returns
    ///
    /// A new section.
    pub fn new<E, Message>(title: impl Into<String>, content: E) -> Self
    where
        E: Into<Element<'static, Message>>,
        Message: 'static,
    {
        Section {
            title: title.into(),
            content: content.into().map(|msg| {
                SectionMessage::Content(Box::new(msg))
            }),
            collapsible: false,
            collapsed: false,
        }
    }
    
    /// Sets whether the section is collapsible.
    ///
    /// # Arguments
    ///
    /// * `collapsible` - Whether the section is collapsible
    ///
    /// # Returns
    ///
    /// The updated section.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
    
    /// Sets whether the section is collapsed.
    ///
    /// # Arguments
    ///
    /// * `collapsed` - Whether the section is collapsed
    ///
    /// # Returns
    ///
    /// The updated section.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
    
    /// Builds the section element.
    ///
    /// # Type Parameters
    ///
    /// * `Message` - The message type
    ///
    /// # Arguments
    ///
    /// * `on_toggle` - The callback for section toggling
    ///
    /// # Returns
    ///
    /// The section element.
    pub fn build<Message>(
        self,
        on_toggle: impl Fn() -> Message + 'static,
    ) -> Element<Message>
    where
        Message: 'static,
    {
        let mut column = Column::new()
            .width(Length::Fill)
            .spacing(10);
        
        let title_row = if self.collapsible {
            Row::new()
                .align_items(alignment::Alignment::Center)
                .spacing(5)
                .push(
                    Text::new(if self.collapsed { "▶" } else { "▼" })
                        .size(12),
                )
                .push(
                    Text::new(&self.title)
                        .size(16)
                        .width(Length::Fill),
                )
        } else {
            Row::new()
                .align_items(alignment::Alignment::Center)
                .push(
                    Text::new(&self.title)
                        .size(16)
                        .width(Length::Fill),
                )
        };
        
        let title_container = Container::new(title_row)
            .width(Length::Fill)
            .padding(10)
            .style(ContainerStyle::SectionHeader);
        
        column = column.push(
            if self.collapsible {
                title_container
                    .on_press(SectionMessage::Toggled)
                    .map(move |msg| {
                        match msg {
                            SectionMessage::Toggled => on_toggle(),
                            _ => on_toggle(),
                        }
                    })
            } else {
                title_container.map(|_| on_toggle())
            }
        );
        
        if !self.collapsed {
            column = column.push(
                Container::new(self.content.map(move |msg| {
                    match msg {
                        SectionMessage::Toggled => on_toggle(),
                        SectionMessage::Content(content) => {
                            // This is a simplification; in a real implementation,
                            // you would need to handle the content message appropriately
                            on_toggle()
                        }
                    }
                }))
                .width(Length::Fill)
                .padding(10)
                .style(ContainerStyle::SectionContent),
            );
        }
        
        Container::new(column)
            .width(Length::Fill)
            .style(ContainerStyle::Section)
            .into()
    }
}

/// A standard dialog component.
pub struct Dialog {
    /// The title text.
    title: String,
    /// The content.
    content: Element<'static, DialogMessage>,
    /// The primary button text.
    primary_button: Option<String>,
    /// The secondary button text.
    secondary_button: Option<String>,
    /// The cancel button text.
    cancel_button: Option<String>,
}

/// Dialog message.
#[derive(Debug, Clone)]
pub enum DialogMessage {
    /// The primary button was pressed.
    PrimaryPressed,
    /// The secondary button was pressed.
    SecondaryPressed,
    /// The cancel button was pressed.
    CancelPressed,
    /// A content message.
    Content(Box<dyn std::any::Any>),
}

impl Dialog {
    /// Creates a new dialog.
    ///
    /// # Arguments
    ///
    /// * `title` - The title text
    /// * `content` - The dialog content
    ///
    /// # Returns
    ///
    /// A new dialog.
    pub fn new<E, Message>(title: impl Into<String>, content: E) -> Self
    where
        E: Into<Element<'static, Message>>,
        Message: 'static,
    {
        Dialog {
            title: title.into(),
            content: content.into().map(|msg| {
                DialogMessage::Content(Box::new(msg))
            }),
            primary_button: None,
            secondary_button: None,
            cancel_button: Some("Cancel".to_string()),
        }
    }
    
    /// Sets the primary button text.
    ///
    /// # Arguments
    ///
    /// * `text` - The button text
    ///
    /// # Returns
    ///
    /// The updated dialog.
    pub fn primary_button(mut self, text: impl Into<String>) -> Self {
        self.primary_button = Some(text.into());
        self
    }
    
    /// Sets the secondary button text.
    ///
    /// # Arguments
    ///
    /// * `text` - The button text
    ///
    /// # Returns
    ///
    /// The updated dialog.
    pub fn secondary_button(mut self, text: impl Into<String>) -> Self {
        self.secondary_button = Some(text.into());
        self
    }
    
    /// Sets the cancel button text.
    ///
    /// # Arguments
    ///
    /// * `text` - The button text
    ///
    /// # Returns
    ///
    /// The updated dialog.
    pub fn cancel_button(mut self, text: impl Into<String>) -> Self {
        self.cancel_button = Some(text.into());
        self
    }
    
    /// Removes the cancel button.
    ///
    /// # Returns
    ///
    /// The updated dialog.
    pub fn no_cancel_button(mut self) -> Self {
        self.cancel_button = None;
        self
    }
    
    /// Builds the dialog element.
    ///
    /// # Type Parameters
    ///
    /// * `Message` - The message type
    ///
    /// # Arguments
    ///
    /// * `on_primary` - The callback for the primary button
    /// * `on_secondary` - The callback for the secondary button
    /// * `on_cancel` - The callback for the cancel button
    ///
    /// # Returns
    ///
    /// The dialog element.
    pub fn build<Message>(
        self,
        on_primary: impl Fn() -> Message + 'static,
        on_secondary: impl Fn() -> Message + 'static,
        on_cancel: impl Fn() -> Message + 'static,
    ) -> Element<Message>
    where
        Message: 'static,
    {
        let mut column = Column::new()
            .width(Length::Fill)
            .spacing(20)
            .padding(20);
        
        column = column.push(
            Text::new(&self.title)
                .size(18)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center),
        );
        
        column = column.push(
            Container::new(self.content.map(move |msg| {
                match msg {
                    DialogMessage::PrimaryPressed => on_primary(),
                    DialogMessage::SecondaryPressed => on_secondary(),
                    DialogMessage::CancelPressed => on_cancel(),
                    DialogMessage::Content(content) => {
                        // This is a simplification; in a real implementation,
                        // you would need to handle the content message appropriately
                        on_cancel()
                    }
                }
            }))
            .width(Length::Fill)
            .padding(10),
        );
        
        let mut button_row = Row::new()
            .width(Length::Fill)
            .spacing(10)
            .align_items(alignment::Alignment::Center);
        
        if let Some(cancel_text) = self.cancel_button {
            button_row = button_row.push(
                Button::new(Text::new(cancel_text))
                    .on_press(DialogMessage::CancelPressed)
                    .style(ButtonStyle::Secondary)
                    .width(Length::Fill),
            );
        }
        
        if let Some(secondary_text) = self.secondary_button {
            button_row = button_row.push(
                Button::new(Text::new(secondary_text))
                    .on_press(DialogMessage::SecondaryPressed)
                    .style(ButtonStyle::Secondary)
                    .width(Length::Fill),
            );
        }
        
        if let Some(primary_text) = self.primary_button {
            button_row = button_row.push(
                Button::new(Text::new(primary_text))
                    .on_press(DialogMessage::PrimaryPressed)
                    .style(ButtonStyle::Primary)
                    .width(Length::Fill),
            );
        }
        
        column = column.push(button_row);
        
        Container::new(column)
            .width(Length::Fill)
            .max_width(400)
            .style(ContainerStyle::Dialog)
            .into()
    }
}

/// A standard list component.
pub struct List<T> {
    /// The items.
    items: Vec<T>,
    /// The item renderer.
    renderer: Box<dyn Fn(&T, bool) -> Element<'static, ListMessage>>,
    /// The selected item index.
    selected: Option<usize>,
}

/// List message.
#[derive(Debug, Clone)]
pub enum ListMessage {
    /// An item was selected.
    Selected(usize),
    /// A content message.
    Content(Box<dyn std::any::Any>),
}

impl<T> List<T>
where
    T: 'static,
{
    /// Creates a new list.
    ///
    /// # Arguments
    ///
    /// * `items` - The list items
    /// * `renderer` - The item renderer
    ///
    /// # Returns
    ///
    /// A new list.
    pub fn new<F, E, Message>(
        items: Vec<T>,
        renderer: F,
    ) -> Self
    where
        F: Fn(&T, bool) -> E + 'static,
        E: Into<Element<'static, Message>>,
        Message: 'static,
    {
        List {
            items,
            renderer: Box::new(move |item, selected| {
                renderer(item, selected).into().map(|msg| {
                    ListMessage::Content(Box::new(msg))
                })
            }),
            selected: None,
        }
    }
    
    /// Sets the selected item index.
    ///
    /// # Arguments
    ///
    /// * `selected` - The selected item index
    ///
    /// # Returns
    ///
    /// The updated list.
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }
    
    /// Builds the list element.
    ///
    /// # Type Parameters
    ///
    /// * `Message` - The message type
    ///
    /// # Arguments
    ///
    /// * `on_select` - The callback for item selection
    ///
    /// # Returns
    ///
    /// The list element.
    pub fn build<Message>(
        self,
        on_select: impl Fn(usize) -> Message + 'static,
    ) -> Element<Message>
    where
        Message: 'static,
    {
        let mut column = Column::new()
            .width(Length::Fill)
            .spacing(5);
        
        for (i, item) in self.items.iter().enumerate() {
            let is_selected = self.selected.map_or(false, |s| s == i);
            
            column = column.push(
                Container::new((self.renderer)(item, is_selected).map(move |msg| {
                    match msg {
                        ListMessage::Selected(index) => on_select(index),
                        ListMessage::Content(_) => on_select(i),
                    }
                }))
                .width(Length::Fill)
                .on_press(ListMessage::Selected(i)),
            );
        }
        
        Scrollable::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// A standard grid component.
pub struct Grid<T> {
    /// The items.
    items: Vec<T>,
    /// The item renderer.
    renderer: Box<dyn Fn(&T, bool) -> Element<'static, GridMessage>>,
    /// The selected item index.
    selected: Option<usize>,
    /// The number of columns.
    columns: usize,
}

/// Grid message.
#[derive(Debug, Clone)]
pub enum GridMessage {
    /// An item was selected.
    Selected(usize),
    /// A content message.
    Content(Box<dyn std::any::Any>),
}

impl<T> Grid<T>
where
    T: 'static,
{
    /// Creates a new grid.
    ///
    /// # Arguments
    ///
    /// * `items` - The grid items
    /// * `renderer` - The item renderer
    /// * `columns` - The number of columns
    ///
    /// # Returns
    ///
    /// A new grid.
    pub fn new<F, E, Message>(
        items: Vec<T>,
        renderer: F,
        columns: usize,
    ) -> Self
    where
        F: Fn(&T, bool) -> E + 'static,
        E: Into<Element<'static, Message>>,
        Message: 'static,
    {
        Grid {
            items,
            renderer: Box::new(move |item, selected| {
                renderer(item, selected).into().map(|msg| {
                    GridMessage::Content(Box::new(msg))
                })
            }),
            selected: None,
            columns: columns.max(1),
        }
    }
    
    /// Sets the selected item index.
    ///
    /// # Arguments
    ///
    /// * `selected` - The selected item index
    ///
    /// # Returns
    ///
    /// The updated grid.
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }
    
    /// Sets the number of columns.
    ///
    /// # Arguments
    ///
    /// * `columns` - The number of columns
    ///
    /// # Returns
    ///
    /// The updated grid.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }
    
    /// Builds the grid element.
    ///
    /// # Type Parameters
    ///
    /// * `Message` - The message type
    ///
    /// # Arguments
    ///
    /// * `on_select` - The callback for item selection
    ///
    /// # Returns
    ///
    /// The grid element.
    pub fn build<Message>(
        self,
        on_select: impl Fn(usize) -> Message + 'static,
    ) -> Element<Message>
    where
        Message: 'static,
    {
        let mut column = Column::new()
            .width(Length::Fill)
            .spacing(10);
        
        let mut current_row = Row::new()
            .width(Length::Fill)
            .spacing(10);
        
        for (i, item) in self.items.iter().enumerate() {
            let is_selected = self.selected.map_or(false, |s| s == i);
            
            current_row = current_row.push(
                Container::new((self.renderer)(item, is_selected).map(move |msg| {
                    match msg {
                        GridMessage::Selected(index) => on_select(index),
                        GridMessage::Content(_) => on_select(i),
                    }
                }))
                .width(Length::FillPortion(1))
                .on_press(GridMessage::Selected(i)),
            );
            
            if (i + 1) % self.columns == 0 || i == self.items.len() - 1 {
                column = column.push(current_row);
                current_row = Row::new()
                    .width(Length::Fill)
                    .spacing(10);
            }
        }
        
        Scrollable::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
