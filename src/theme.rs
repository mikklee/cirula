/*
Copyright (C) 2024

sirula is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

sirula is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with sirula.  If not, see <https://www.gnu.org/licenses/>.
*/

use iced::widget::{container, scrollable, text_input};
use iced::{Border, Color, Theme};

/// Default border radius for UI elements
pub const BORDER_RADIUS: f32 = 12.0;

/// Cyberpunk theme palette
pub fn cyberpunk_palette() -> iced::theme::Palette {
    iced::theme::Palette {
        background: Color::from_rgb(0.08, 0.08, 0.12), // Dark blue-black
        text: Color::from_rgb(0.95, 0.95, 0.98),       // Near white
        primary: Color::from_rgb(0.5, 0.4, 0.9),       // Purple accent
        success: Color::from_rgb(0.3, 0.8, 0.6),       // Teal/green
        danger: Color::from_rgb(0.9, 0.3, 0.4),        // Red/pink
        warning: Color::from_rgb(0.9, 0.7, 0.3),       // Orange/yellow
    }
}

/// Create the cyberpunk theme
pub fn cyberpunk_theme() -> Theme {
    Theme::custom("Cyberpunk".to_string(), cyberpunk_palette())
}

/// Selected item background color (slightly lighter)
pub fn selected_bg() -> Color {
    Color::from_rgb(0.15, 0.15, 0.22)
}

/// Hover item background color
pub fn hover_bg() -> Color {
    Color::from_rgb(0.12, 0.12, 0.18)
}

/// Highlight color for matched text
pub fn highlight_color() -> Color {
    Color::from_rgb(0.5, 0.4, 0.9) // Purple accent (same as primary)
}

/// Extra field text color (dimmer)
pub fn extra_text_color() -> Color {
    Color::from_rgb(0.6, 0.6, 0.65)
}

/// Rounded text input style
pub fn rounded_text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.border = Border {
        radius: BORDER_RADIUS.into(),
        width: 1.0,
        color: Color::from_rgb(0.3, 0.3, 0.4),
    };
    style.background = iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.15));
    style
}

/// Container style for app rows
pub fn app_row_container(is_selected: bool) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(if is_selected {
            selected_bg()
        } else {
            Color::TRANSPARENT
        })),
        border: Border {
            radius: (BORDER_RADIUS / 2.0).into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Main container style (semi-transparent background)
pub fn main_container_style() -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.08, 0.08, 0.12, 0.95,
        ))),
        ..Default::default()
    }
}

/// Scrollable style
pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    // Make scrollbar semi-transparent
    style.container = container::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        ..Default::default()
    };
    style
}
