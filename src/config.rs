/*
Copyright (C) 2020 Dorian Rudolph

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

use super::consts::*;
use super::util::get_config_file;
use iced::Color;
use icu_locale::Locale;
use serde::Deserialize;
use std::collections::HashMap;
use strum::{Display, EnumString};

/// Parse a hex color string (supports #RGB, #RGBA, #RRGGBB, #RRGGBBAA)
fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim_start_matches('#');
    match s.len() {
        3 => {
            // #RGB
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(Color::from_rgb8(r, g, b))
        }
        4 => {
            // #RGBA
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&s[3..4], 16).ok()? * 17;
            Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
        }
        6 => {
            // #RRGGBB
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
        }
        8 => {
            // #RRGGBBAA
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
        }
        _ => None,
    }
}

/// Custom palette configuration
#[derive(Deserialize, Debug, Clone, Default)]
pub struct CustomPalette {
    pub background: Option<String>,
    pub text: Option<String>,
    pub primary: Option<String>,
    pub success: Option<String>,
    pub danger: Option<String>,
    pub warning: Option<String>,
}

impl CustomPalette {
    pub fn to_iced_palette(&self) -> Option<iced::theme::Palette> {
        // All colors must be specified for a custom palette
        Some(iced::theme::Palette {
            background: parse_hex_color(self.background.as_ref()?)?,
            text: parse_hex_color(self.text.as_ref()?)?,
            primary: parse_hex_color(self.primary.as_ref()?)?,
            success: parse_hex_color(self.success.as_ref()?)?,
            danger: parse_hex_color(self.danger.as_ref()?)?,
            warning: parse_hex_color(self.warning.as_ref()?)?,
        })
    }
}

macro_rules! make_config {
    ($name:ident { $($field:ident : $type:ty $( = ($default:expr) $field_str:literal )? ),* }) => {
        #[derive(Deserialize, Debug)]
        pub struct $name { $(
            #[serde( $(default = $field_str )? )]
            pub $field: $type,
        )* }
        $( $( fn $field() -> $type { $default } )? )*
    };
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Comment,
    Id,
    IdSuffix,
    Executable,
    Commandline,
}

#[derive(Deserialize, Debug, Clone, Default, EnumString, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ThemeName {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    #[default]
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

make_config!(Config {
    app_theme: ThemeName = (ThemeName::default()) "app_theme",
    custom_palette: CustomPalette = (CustomPalette::default()) "custom_palette",
    locale: Option<Locale> = (None) "locale",
    exclusive: bool = (true) "exclusive",
    frequent_first: bool = (false) "frequent_first",
    recent_first: bool = (true) "recent_first",
    prune_history: u32 = (0) "prune_history",
    icon_size: i32 = (64) "icon_size",
    margin_left: i32 = (0) "margin_left",
    margin_right: i32 = (0) "margin_right",
    margin_top: i32 = (0) "margin_top",
    margin_bottom: i32 = (0) "margin_bottom",
    anchor_left: bool = (false) "anchor_left",
    anchor_right: bool = (true) "anchor_right",
    anchor_top: bool = (true) "anchor_top",
    anchor_bottom: bool = (true) "anchor_bottom",
    width: i32 = (-1) "width",
    height: i32 = (-1) "height",
    extra_field: Vec<Field> = (vec![Field::IdSuffix]) "extra_field",
    extra_field_newline: bool = (false) "extra_field_newline",
    hidden_fields: Vec<Field> = (Vec::new()) "hidden_fields",
    name_overrides: HashMap<String, String> = (HashMap::new()) "name_overrides",
    hide_extra_if_contained: bool = (true) "hide_extra_if_contained",
    cgroups: bool = (true) "cgroups",
    command_prefix: String = (":".into()) "command_prefix",
    exclude: Vec<String> = (Vec::new()) "exclude",
    term_command: Option<String> = (None) "term_command"
});

impl Config {
    pub fn load() -> Config {
        let config_str = match get_config_file(CONFIG_FILE) {
            Some(file) => std::fs::read_to_string(file).expect("Cannot read config"),
            _ => "".to_owned(),
        };
        let config: Config = toml::from_str(&config_str).expect("Cannot parse config");
        config
    }
}
