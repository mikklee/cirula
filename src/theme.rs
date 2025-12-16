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

use crate::config::{CustomPalette, ThemeName};
use iced::widget::{container, scrollable, text_input};
use iced::{Border, Color, Theme};

/// Default border radius for UI elements
pub const BORDER_RADIUS: f32 = 12.0;

/// Convert ThemeName to iced::Theme, with optional custom palette override
pub fn get_theme_with_custom(name: &ThemeName, custom: &CustomPalette) -> Theme {
    // If custom palette is fully defined, use it
    if let Some(palette) = custom.to_iced_palette() {
        return Theme::custom("Custom".to_string(), palette);
    }
    get_theme(name)
}

/// Convert ThemeName to iced::Theme
pub fn get_theme(name: &ThemeName) -> Theme {
    match name {
        ThemeName::Light => Theme::Light,
        ThemeName::Dark => Theme::Dark,
        ThemeName::Dracula => Theme::Dracula,
        ThemeName::Nord => Theme::Nord,
        ThemeName::SolarizedLight => Theme::SolarizedLight,
        ThemeName::SolarizedDark => Theme::SolarizedDark,
        ThemeName::GruvboxLight => Theme::GruvboxLight,
        ThemeName::GruvboxDark => Theme::GruvboxDark,
        ThemeName::CatppuccinLatte => Theme::CatppuccinLatte,
        ThemeName::CatppuccinFrappe => Theme::CatppuccinFrappe,
        ThemeName::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
        ThemeName::CatppuccinMocha => Theme::CatppuccinMocha,
        ThemeName::TokyoNight => Theme::TokyoNight,
        ThemeName::TokyoNightStorm => Theme::TokyoNightStorm,
        ThemeName::TokyoNightLight => Theme::TokyoNightLight,
        ThemeName::KanagawaWave => Theme::KanagawaWave,
        ThemeName::KanagawaDragon => Theme::KanagawaDragon,
        ThemeName::KanagawaLotus => Theme::KanagawaLotus,
        ThemeName::Moonfly => Theme::Moonfly,
        ThemeName::Nightfly => Theme::Nightfly,
        ThemeName::Oxocarbon => Theme::Oxocarbon,
        ThemeName::Ferra => Theme::Ferra,
    }
}

/// Selected item background color (derived from theme)
pub fn selected_bg(theme: &Theme) -> Color {
    theme.extended_palette().background.weak.color
}

/// Extra field text color (dimmer version of text color)
pub fn extra_text_color(theme: &Theme) -> Color {
    theme.extended_palette().secondary.weak.color
}

/// Rounded text input style
pub fn rounded_text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();
    let mut style = text_input::default(theme, status);
    style.border = Border {
        radius: BORDER_RADIUS.into(),
        width: 1.0,
        color: palette.background.strong.color,
    };
    style.background = iced::Background::Color(palette.background.weak.color);
    style
}

/// Container style for app rows
pub fn app_row_container(theme: &Theme, is_selected: bool) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(if is_selected {
            selected_bg(theme)
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
