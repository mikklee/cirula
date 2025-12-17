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

use fuzzy_matcher::skim::SkimMatcherV2;
use iced::widget::operation::{focus, scroll_to, AbsoluteOffset};
use iced::widget::{column, container, image, row, scrollable, svg, text, text_input, Column};
use iced::{event, keyboard, Element, Event, Length, Subscription, Task};
use iced_layershell::build_pattern::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use std::collections::HashMap;
use std::sync::OnceLock;

const ITEM_HEIGHT: f32 = 50.0; // Approximate height of each entry row

mod consts;
use consts::*;

mod config;
use config::*;

mod util;
use util::*;

mod app_entry;
use app_entry::*;

mod locale;

mod history;
use history::*;

mod theme;
use theme::{
    app_row_container, extra_text_color, get_theme_with_custom, rounded_text_input_style,
    scrollable_style,
};

static INPUT_ID: OnceLock<iced::widget::Id> = OnceLock::new();
static SCROLL_ID: OnceLock<iced::widget::Id> = OnceLock::new();
static CONFIG: OnceLock<Config> = OnceLock::new();
static THEME: OnceLock<iced::Theme> = OnceLock::new();

fn get_input_id() -> iced::widget::Id {
    INPUT_ID.get_or_init(iced::widget::Id::unique).clone()
}

fn get_scroll_id() -> iced::widget::Id {
    SCROLL_ID.get_or_init(iced::widget::Id::unique).clone()
}

fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}

fn get_theme() -> &'static iced::Theme {
    THEME.get_or_init(|| {
        let config = get_config();
        get_theme_with_custom(&config.app_theme, &config.custom_palette)
    })
}

fn scroll_to_selected(selected: usize) -> Task<Message> {
    let offset = AbsoluteOffset {
        x: 0.0,
        y: selected as f32 * ITEM_HEIGHT,
    };
    scroll_to(get_scroll_id(), offset)
}

fn main() -> Result<(), iced_layershell::Error> {
    // Initialize tracing (set RUST_LOG=sirula=debug to see debug output)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("iced_layershell=warn".parse().unwrap())
                .add_directive("iced=warn".parse().unwrap())
                .add_directive("wgpu=warn".parse().unwrap())
                .add_directive("naga=warn".parse().unwrap())
                .add_directive("cosmic_text=warn".parse().unwrap())
                .add_directive("usvg=error".parse().unwrap()),
        )
        .init();

    let config = get_config();

    // Calculate size based on config
    let width = if config.width > 0 {
        config.width as u32
    } else {
        400
    };
    let height = if config.height > 0 {
        config.height as u32
    } else {
        600
    };

    // Build anchor from config
    let mut anchor = Anchor::empty();
    if config.anchor_left {
        anchor |= Anchor::Left;
    }
    if config.anchor_right {
        anchor |= Anchor::Right;
    }
    if config.anchor_top {
        anchor |= Anchor::Top;
    }
    if config.anchor_bottom {
        anchor |= Anchor::Bottom;
    }

    application(Launcher::new, namespace, update, view)
        .subscription(subscription)
        .style(style)
        .theme(theme)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                size: Some((width, height)),
                anchor,
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                margin: (
                    config.margin_top,
                    config.margin_right,
                    config.margin_bottom,
                    config.margin_left,
                ),
                exclusive_zone: if config.exclusive { -1 } else { 0 },
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

fn namespace() -> String {
    APP_NAME.to_string()
}

fn subscription(_state: &Launcher) -> Subscription<Message> {
    event::listen_with(|event, _status, _id| {
        // Only capture keyboard key press events, ignore everything else
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = &event {
            match key {
                keyboard::Key::Named(
                    keyboard::key::Named::Escape
                    | keyboard::key::Named::ArrowUp
                    | keyboard::key::Named::ArrowDown
                    | keyboard::key::Named::Tab,
                ) => Some(Message::IcedEvent(event)),
                _ => None, // Let text_input handle other keys
            }
        } else {
            None // Ignore mouse events, key releases, etc.
        }
    })
}

fn style(_state: &Launcher, _theme: &iced::Theme) -> iced::theme::Style {
    let palette = get_theme().palette();
    iced::theme::Style {
        background_color: iced::Color::from_rgba(
            palette.background.r,
            palette.background.g,
            palette.background.b,
            0.95,
        ),
        text_color: palette.text,
    }
}

fn theme(_state: &Launcher) -> iced::Theme {
    get_theme().clone()
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Submit,
    IcedEvent(Event),
    Launch(usize),
    FocusInput,
}

#[derive(Clone)]
enum CachedIcon {
    Svg(svg::Handle),
    Image(image::Handle),
}

struct Launcher {
    query: String,
    entries: Vec<AppEntry>,
    filtered_indices: Vec<usize>,
    selected: usize,
    matcher: SkimMatcherV2,
    history: HashMap<String, HistoryData>,
    icon_cache: HashMap<usize, CachedIcon>, // index -> cached icon handle
}

impl Launcher {
    fn new() -> (Self, Task<Message>) {
        let config = get_config();
        let history = load_history(config.prune_history);
        let entries = load_entries(config, &history);
        let filtered_indices: Vec<usize> = (0..entries.len()).collect();

        // Pre-cache all icon handles
        let mut icon_cache = HashMap::new();
        for (idx, entry) in entries.iter().enumerate() {
            if let Some(ref icon_path) = entry.icon_path {
                let ext = icon_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let cached = if ext == "svg" {
                    CachedIcon::Svg(svg::Handle::from_path(icon_path))
                } else {
                    CachedIcon::Image(image::Handle::from_path(icon_path))
                };
                icon_cache.insert(idx, cached);
            }
        }

        (
            Self {
                query: String::new(),
                entries,
                filtered_indices,
                selected: 0,
                matcher: SkimMatcherV2::default(),
                history,
                icon_cache,
            },
            Task::done(Message::FocusInput),
        )
    }
}

fn update(state: &mut Launcher, message: Message) -> Task<Message> {
    match message {
        Message::InputChanged(query) => {
            state.query = query.clone();

            // Check if it's a command
            if is_cmd(&state.query, &get_config().command_prefix) {
                // In command mode, hide all entries
                for entry in &mut state.entries {
                    entry.hide();
                }
                state.filtered_indices.clear();
            } else {
                // Update matches
                for entry in &mut state.entries {
                    entry.update_match(&query, &state.matcher);
                }

                // Sort entries
                let mut indexed: Vec<(usize, &AppEntry)> =
                    state.entries.iter().enumerate().collect();
                indexed.sort_by(|(_, a), (_, b)| a.cmp(b));

                // Filter visible entries
                state.filtered_indices = indexed
                    .into_iter()
                    .filter(|(_, e)| !e.hidden())
                    .map(|(i, _)| i)
                    .collect();
            }

            state.selected = 0;
            Task::none()
        }
        Message::Submit => {
            if is_cmd(&state.query, &get_config().command_prefix) {
                let cmd_line = &state.query[get_config().command_prefix.len()..].trim();
                launch_cmd(cmd_line);
                return iced::exit();
            }

            if let Some(&idx) = state.filtered_indices.get(state.selected) {
                let entry = &state.entries[idx];
                launch_app(
                    &entry.command_line,
                    entry.is_terminal,
                    &entry.id,
                    get_config().term_command.as_deref(),
                    get_config().cgroups,
                );

                // Update history
                update_history(&mut state.history, &entry.id);
                save_history(&state.history);

                return iced::exit();
            }
            Task::none()
        }
        Message::IcedEvent(event) => {
            if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        return iced::exit();
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        if !state.filtered_indices.is_empty() {
                            state.selected =
                                (state.selected + 1).min(state.filtered_indices.len() - 1);
                            return scroll_to_selected(state.selected);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        if state.selected > 0 {
                            state.selected -= 1;
                            return scroll_to_selected(state.selected);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Tab) => {
                        if !state.filtered_indices.is_empty() {
                            state.selected = (state.selected + 1) % state.filtered_indices.len();
                            return scroll_to_selected(state.selected);
                        }
                    }
                    _ => {}
                }
            }
            Task::none()
        }
        Message::FocusInput => focus(get_input_id()),
        Message::Launch(idx) => {
            if let Some(entry) = state.entries.get(idx) {
                launch_app(
                    &entry.command_line,
                    entry.is_terminal,
                    &entry.id,
                    get_config().term_command.as_deref(),
                    get_config().cgroups,
                );

                // Update history
                let id = entry.id.clone();
                update_history(&mut state.history, &id);
                save_history(&state.history);

                return iced::exit();
            }
            Task::none()
        }
        // Layer shell messages - not used but required by the macro
        _ => Task::none(),
    }
}

fn view(state: &Launcher) -> Element<'_, Message> {
    let search_input = text_input("Search...", &state.query)
        .id(get_input_id())
        .on_input(Message::InputChanged)
        .on_submit(Message::Submit)
        .padding(12)
        .size(18)
        .style(rounded_text_input_style);

    let entries_list: Element<'_, Message> = if state.filtered_indices.is_empty() {
        if is_cmd(&state.query, &get_config().command_prefix) {
            container(
                text(format!(
                    "Run: {}",
                    &state.query[get_config().command_prefix.len()..].trim()
                ))
                .size(16),
            )
            .padding(20)
            .into()
        } else {
            container(text("No matches").size(16)).padding(20).into()
        }
    } else {
        let icon_size = get_config().icon_size;
        let items: Vec<Element<'_, Message>> = state
            .filtered_indices
            .iter()
            .enumerate()
            .take(50) // Limit visible items for performance
            .map(|(display_idx, &entry_idx)| {
                let entry = &state.entries[entry_idx];
                let is_selected = display_idx == state.selected;

                render_entry(
                    entry,
                    entry_idx,
                    is_selected,
                    icon_size,
                    state.icon_cache.get(&entry_idx),
                )
            })
            .collect();

        scrollable(Column::with_children(items).spacing(2).width(Length::Fill))
            .id(get_scroll_id())
            .height(Length::Fill)
            .style(scrollable_style)
            .into()
    };

    let content = column![search_input, entries_list].spacing(8).padding(12);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_entry<'a>(
    entry: &'a AppEntry,
    idx: usize,
    is_selected: bool,
    icon_size: i32,
    cached_icon: Option<&'a CachedIcon>,
) -> Element<'a, Message> {
    let theme = get_theme();
    let name_text = text(&entry.name).size(16);

    let mut content_col = Column::new().push(name_text);

    if let Some(ref extra) = entry.extra_text {
        content_col = content_col.push(text(extra).size(12).color(extra_text_color(theme)));
    }

    let mut row_content = row![].spacing(12).padding(8).width(Length::Fill);

    // Add icon from cache, or fallback to Font Awesome icon
    match cached_icon {
        Some(CachedIcon::Svg(handle)) => {
            let icon_widget = svg(handle.clone())
                .width(icon_size as f32)
                .height(icon_size as f32);
            row_content = row_content.push(icon_widget);
        }
        Some(CachedIcon::Image(handle)) => {
            let icon_widget = image(handle.clone())
                .width(icon_size as f32)
                .height(icon_size as f32);
            row_content = row_content.push(icon_widget);
        }
        None => {
            // Fallback to Font Awesome cube icon
            let fallback_icon: iced_font_awesome::Icon<'_, iced::Theme> =
                iced_font_awesome::fa_icon_solid("cube").size(icon_size as f32);
            row_content = row_content.push(fallback_icon);
        }
    }

    row_content = row_content.push(content_col);

    iced::widget::mouse_area(
        container(row_content)
            .width(Length::Fill)
            .style(move |_| app_row_container(theme, is_selected)),
    )
    .on_press(Message::Launch(idx))
    .into()
}
