//! Styles module for the NovaDE UI layer.
//!
//! This module provides custom styles for UI components.

use iced::{Color, Background, Vector};
use iced::widget::{button, container, text, scrollable, text_input, checkbox, radio, progress_bar, rule};

/// Button style.
pub enum ButtonStyle {
    /// Primary button style.
    Primary,
    /// Secondary button style.
    Secondary,
    /// Destructive button style.
    Destructive,
    /// Icon button style.
    Icon,
    /// Text button style.
    Text,
}

impl button::StyleSheet for ButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        match self {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.5, 0.8))),
                border_radius: 4.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                text_color: Color::WHITE,
                shadow_offset: Vector::new(0.0, 1.0),
                ..Default::default()
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
                border_radius: 4.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                text_color: Color::WHITE,
                shadow_offset: Vector::new(0.0, 1.0),
                ..Default::default()
            },
            ButtonStyle::Destructive => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                border_radius: 4.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                text_color: Color::WHITE,
                shadow_offset: Vector::new(0.0, 1.0),
                ..Default::default()
            },
            ButtonStyle::Icon => button::Appearance {
                background: None,
                border_radius: 4.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                text_color: Color::WHITE,
                shadow_offset: Vector::ZERO,
                ..Default::default()
            },
            ButtonStyle::Text => button::Appearance {
                background: None,
                border_radius: 4.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                text_color: Color::from_rgb(0.2, 0.5, 0.8),
                shadow_offset: Vector::ZERO,
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        match self {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
                ..active
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.4, 0.4, 0.4))),
                ..active
            },
            ButtonStyle::Destructive => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.9, 0.3, 0.3))),
                ..active
            },
            ButtonStyle::Icon => button::Appearance {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
                ..active
            },
            ButtonStyle::Text => button::Appearance {
                text_color: Color::from_rgb(0.3, 0.6, 0.9),
                ..active
            },
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let hovered = self.hovered(style);

        match self {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.4, 0.7))),
                shadow_offset: Vector::ZERO,
                ..hovered
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                shadow_offset: Vector::ZERO,
                ..hovered
            },
            ButtonStyle::Destructive => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.7, 0.1, 0.1))),
                shadow_offset: Vector::ZERO,
                ..hovered
            },
            ButtonStyle::Icon => button::Appearance {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.2))),
                ..hovered
            },
            ButtonStyle::Text => button::Appearance {
                text_color: Color::from_rgb(0.1, 0.4, 0.7),
                ..hovered
            },
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            background: active.background.map(|bg| match bg {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                _ => bg,
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

/// Container style.
pub enum ContainerStyle {
    /// Default container style.
    Default,
    /// Card container style.
    Card,
    /// Selected card container style.
    SelectedCard,
    /// Dialog container style.
    Dialog,
    /// Header container style.
    Header,
    /// Section container style.
    Section,
    /// Section header container style.
    SectionHeader,
    /// Section content container style.
    SectionContent,
    /// Panel container style.
    Panel,
    /// Workspace container style.
    Workspace,
    /// Desktop container style.
    Desktop,
}

impl container::StyleSheet for ContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        match self {
            ContainerStyle::Default => container::Appearance {
                background: None,
                text_color: None,
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::Card => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                text_color: Some(Color::WHITE),
                border_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::SelectedCard => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.4, 0.6))),
                text_color: Some(Color::WHITE),
                border_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::Dialog => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                text_color: Some(Color::WHITE),
                border_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::Header => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                text_color: Some(Color::WHITE),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::Section => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                text_color: Some(Color::WHITE),
                border_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::SectionHeader => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                text_color: Some(Color::WHITE),
                border_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::SectionContent => container::Appearance {
                background: None,
                text_color: Some(Color::WHITE),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::Panel => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                text_color: Some(Color::WHITE),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::Workspace => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                text_color: Some(Color::WHITE),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            ContainerStyle::Desktop => container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.05))),
                text_color: Some(Color::WHITE),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

/// Text input style.
pub enum TextInputStyle {
    /// Default text input style.
    Default,
    /// Search text input style.
    Search,
}

impl text_input::StyleSheet for TextInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        match self {
            TextInputStyle::Default => text_input::Appearance {
                background: Background::Color(Color::from_rgb(0.15, 0.15, 0.15)),
                border_radius: 4.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
                icon_color: Color::from_rgb(0.7, 0.7, 0.7),
            },
            TextInputStyle::Search => text_input::Appearance {
                background: Background::Color(Color::from_rgb(0.15, 0.15, 0.15)),
                border_radius: 20.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                icon_color: Color::from_rgb(0.7, 0.7, 0.7),
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let active = self.active(style);

        match self {
            TextInputStyle::Default => text_input::Appearance {
                border_color: Color::from_rgb(0.2, 0.5, 0.8),
                ..active
            },
            TextInputStyle::Search => text_input::Appearance {
                background: Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
                ..active
            },
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.5, 0.5, 0.5)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::WHITE
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.2, 0.5, 0.8)
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        let active = self.active(style);

        text_input::Appearance {
            background: match active.background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                _ => active.background,
            },
            ..active
        }
    }
}

/// Scrollable style.
pub enum ScrollableStyle {
    /// Default scrollable style.
    Default,
    /// Dark scrollable style.
    Dark,
}

impl scrollable::StyleSheet for ScrollableStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> scrollable::Appearance {
        match self {
            ScrollableStyle::Default => scrollable::Appearance {
                background: None,
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    border_radius: 4.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            },
            ScrollableStyle::Dark => scrollable::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    border_radius: 4.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> scrollable::Appearance {
        let active = self.active(style);

        match self {
            ScrollableStyle::Default => scrollable::Appearance {
                scroller: scrollable::Scroller {
                    color: if is_mouse_over_scrollbar {
                        Color::from_rgb(0.7, 0.7, 0.7)
                    } else {
                        active.scroller.color
                    },
                    ..active.scroller
                },
                ..active
            },
            ScrollableStyle::Dark => scrollable::Appearance {
                scroller: scrollable::Scroller {
                    color: if is_mouse_over_scrollbar {
                        Color::from_rgb(0.5, 0.5, 0.5)
                    } else {
                        active.scroller.color
                    },
                    ..active.scroller
                },
                ..active
            },
        }
    }

    fn dragging(&self, style: &Self::Style) -> scrollable::Appearance {
        let hovered = self.hovered(style, true);

        match self {
            ScrollableStyle::Default => scrollable::Appearance {
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.8, 0.8, 0.8),
                    ..hovered.scroller
                },
                ..hovered
            },
            ScrollableStyle::Dark => scrollable::Appearance {
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.6, 0.6, 0.6),
                    ..hovered.scroller
                },
                ..hovered
            },
        }
    }
}

/// Checkbox style.
pub struct CheckboxStyle;

impl checkbox::StyleSheet for CheckboxStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: if is_checked {
                Background::Color(Color::from_rgb(0.2, 0.5, 0.8))
            } else {
                Background::Color(Color::from_rgb(0.2, 0.2, 0.2))
            },
            border_radius: 4.0,
            border_width: 1.0,
            border_color: if is_checked {
                Color::from_rgb(0.2, 0.5, 0.8)
            } else {
                Color::from_rgb(0.3, 0.3, 0.3)
            },
            icon_color: Color::WHITE,
            text_color: Some(Color::WHITE),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let active = self.active(style, is_checked);

        checkbox::Appearance {
            background: if is_checked {
                Background::Color(Color::from_rgb(0.3, 0.6, 0.9))
            } else {
                Background::Color(Color::from_rgb(0.25, 0.25, 0.25))
            },
            border_color: if is_checked {
                Color::from_rgb(0.3, 0.6, 0.9)
            } else {
                Color::from_rgb(0.4, 0.4, 0.4)
            },
            ..active
        }
    }
}

/// Radio style.
pub struct RadioStyle;

impl radio::StyleSheet for RadioStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, is_selected: bool) -> radio::Appearance {
        radio::Appearance {
            background: if is_selected {
                Background::Color(Color::from_rgb(0.2, 0.5, 0.8))
            } else {
                Background::Color(Color::from_rgb(0.2, 0.2, 0.2))
            },
            dot_color: Color::WHITE,
            border_width: 1.0,
            border_color: if is_selected {
                Color::from_rgb(0.2, 0.5, 0.8)
            } else {
                Color::from_rgb(0.3, 0.3, 0.3)
            },
            text_color: Some(Color::WHITE),
        }
    }

    fn hovered(&self, style: &Self::Style, is_selected: bool) -> radio::Appearance {
        let active = self.active(style, is_selected);

        radio::Appearance {
            background: if is_selected {
                Background::Color(Color::from_rgb(0.3, 0.6, 0.9))
            } else {
                Background::Color(Color::from_rgb(0.25, 0.25, 0.25))
            },
            border_color: if is_selected {
                Color::from_rgb(0.3, 0.6, 0.9)
            } else {
                Color::from_rgb(0.4, 0.4, 0.4)
            },
            ..active
        }
    }
}

/// Progress bar style.
pub struct ProgressBarStyle;

impl progress_bar::StyleSheet for ProgressBarStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
            bar: Background::Color(Color::from_rgb(0.2, 0.5, 0.8)),
            border_radius: 4.0,
        }
    }
}

/// Rule style.
pub enum RuleStyle {
    /// Default rule style.
    Default,
    /// Light rule style.
    Light,
}

impl rule::StyleSheet for RuleStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        match self {
            RuleStyle::Default => rule::Appearance {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1,
                radius: 0.0,
                fill_mode: rule::FillMode::Full,
            },
            RuleStyle::Light => rule::Appearance {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1,
                radius: 0.0,
                fill_mode: rule::FillMode::Full,
            },
        }
    }
}
