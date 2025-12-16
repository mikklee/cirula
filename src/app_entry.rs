/*
This file is part of sirula.

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

use crate::locale::string_collate;
use freedesktop_entry_parser::{parse_entry, Entry};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use regex::RegexSet;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, trace};

use super::{Config, Field, HistoryData};

#[derive(Clone)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub display_string: String,
    pub search_string: String,
    pub extra_text: Option<String>,
    pub command_line: String,
    pub icon_path: Option<PathBuf>,
    pub is_terminal: bool,
    pub score: i64,
    pub history: HistoryData,
}

impl AppEntry {
    pub fn update_match(&mut self, pattern: &str, matcher: &SkimMatcherV2) -> Vec<usize> {
        if pattern.is_empty() {
            self.score = 100;
            return vec![];
        }

        if let Some((score, indices)) = matcher.fuzzy_indices(&self.search_string, pattern) {
            self.score = score;
            indices
                .into_iter()
                .filter(|&i| i < self.display_string.len())
                .collect()
        } else {
            self.score = 0;
            vec![]
        }
    }

    pub fn hide(&mut self) {
        self.score = 0;
    }

    pub fn hidden(&self) -> bool {
        self.score == 0
    }
}

impl PartialEq for AppEntry {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score) && self.history.eq(&other.history)
    }
}

impl Eq for AppEntry {}

impl Ord for AppEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.score.cmp(&other.score) {
            Ordering::Equal => match self.history.usage_count.cmp(&other.history.usage_count) {
                Ordering::Equal => match self.history.last_used.cmp(&other.history.last_used) {
                    Ordering::Equal => string_collate(&self.display_string, &other.display_string),
                    ord => ord.reverse(),
                },
                ord => ord.reverse(),
            },
            ord => ord.reverse(),
        }
    }
}

impl PartialOrd for AppEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Helper to get first value from attr (freedesktop_entry_parser 2.0 returns &[String])
fn get_attr(entry: &Entry, key: &str) -> Option<String> {
    entry
        .section("Desktop Entry")
        .and_then(|s| s.attr(key).first().map(|s| s.to_string()))
}

fn get_app_field(entry: &Entry, field: Field) -> Option<String> {
    match field {
        Field::Comment => get_attr(entry, "Comment"),
        Field::Id => None,
        Field::IdSuffix => None,
        Field::Executable => get_attr(entry, "Exec").and_then(|exec| {
            exec.split_whitespace().next().map(|s| {
                std::path::Path::new(s)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| s.to_string())
            })
        }),
        Field::Commandline => get_attr(entry, "Exec"),
    }
}

fn get_id_field(id: &str, field: Field) -> Option<String> {
    let id_without_desktop = id.strip_suffix(".desktop").unwrap_or(id);
    match field {
        Field::Id => Some(id_without_desktop.to_string()),
        Field::IdSuffix => {
            let parts: Vec<&str> = id_without_desktop.split('.').collect();
            parts
                .get(parts.len().saturating_sub(1))
                .map(|s| s.to_string())
        }
        _ => None,
    }
}

/// Fallback icon lookup for NixOS and other systems where freedesktop-icons may not find icons
/// Prefers SVG icons, then highest resolution PNG
fn lookup_icon(icon_name: &str, _icon_size: i32) -> Option<PathBuf> {
    // Get search dirs from XDG_DATA_DIRS environment variable
    let mut search_dirs: Vec<PathBuf> = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();

    trace!(icon_name, "XDG_DATA_DIRS has {} entries", search_dirs.len());

    // Add common system paths that might not be in XDG_DATA_DIRS
    search_dirs.push(PathBuf::from("/run/current-system/sw/share"));
    search_dirs.push(PathBuf::from("/usr/share"));
    search_dirs.push(PathBuf::from("/usr/local/share"));

    // Add home data dir
    if let Ok(home) = std::env::var("HOME") {
        search_dirs.push(PathBuf::from(home).join(".local/share"));
    }

    // Icon sizes to try, prefer scalable (SVG) and larger sizes first
    let sizes = [
        "scalable", "512x512", "256x256", "128x128", "96x96", "72x72", "64x64", "48x48", "32x32",
        "24x24", "22x22", "16x16",
    ];

    let themes = [
        "hicolor",
        "Adwaita",
        "breeze",
        "Cosmic",
        "Pop",
        "MoreWaita",
        "gnome",
        "oxygen",
    ];
    let categories = ["apps", "applications", "mimetypes", "places", "devices"];

    // First pass: look for SVG icons (best quality, scalable)
    for base in &search_dirs {
        // Check pixmaps for SVG first
        let path = base.join("pixmaps").join(format!("{}.svg", icon_name));
        if path.exists() {
            trace!(icon_name, ?path, "Found SVG icon in pixmaps");
            return Some(path);
        }

        // Check icon themes for SVG (scalable directory)
        for theme in &themes {
            for category in &categories {
                let path = base
                    .join("icons")
                    .join(theme)
                    .join("scalable")
                    .join(category)
                    .join(format!("{}.svg", icon_name));
                if path.exists() {
                    trace!(icon_name, ?path, "Found scalable SVG icon");
                    return Some(path);
                }
            }
        }

        // Check sized directories for SVG (some themes put SVGs in sized dirs)
        for theme in &themes {
            for size in &sizes {
                for category in &categories {
                    let path = base
                        .join("icons")
                        .join(theme)
                        .join(size)
                        .join(category)
                        .join(format!("{}.svg", icon_name));
                    if path.exists() {
                        trace!(icon_name, ?path, "Found SVG icon");
                        return Some(path);
                    }
                }
            }
        }
    }

    // Second pass: look for PNG icons, preferring highest resolution
    for base in &search_dirs {
        for theme in &themes {
            for size in &sizes {
                if *size == "scalable" {
                    continue; // Skip scalable for PNG lookup
                }
                for category in &categories {
                    let path = base
                        .join("icons")
                        .join(theme)
                        .join(size)
                        .join(category)
                        .join(format!("{}.png", icon_name));
                    if path.exists() {
                        trace!(icon_name, ?path, "Found PNG icon");
                        return Some(path);
                    }
                }
            }
        }

        // Check pixmaps for PNG (usually lower quality, check last)
        let path = base.join("pixmaps").join(format!("{}.png", icon_name));
        if path.exists() {
            trace!(icon_name, ?path, "Found PNG icon in pixmaps");
            return Some(path);
        }
    }

    // Third pass: XPM as last resort
    for base in &search_dirs {
        let path = base.join("pixmaps").join(format!("{}.xpm", icon_name));
        if path.exists() {
            trace!(icon_name, ?path, "Found XPM icon in pixmaps");
            return Some(path);
        }
    }

    debug!(icon_name, "Icon not found");
    None
}

pub fn load_entries(config: &Config, history: &HashMap<String, HistoryData>) -> Vec<AppEntry> {
    let mut entries = Vec::new();
    let exclude = RegexSet::new(&config.exclude).expect("Invalid regex");

    // Find all .desktop files in XDG data directories
    let data_dirs = xdg::BaseDirectories::new();
    let mut desktop_files: Vec<PathBuf> = Vec::new();

    // Search in applications directories
    for dir in data_dirs.get_data_dirs() {
        let apps_dir = dir.join("applications");
        if apps_dir.exists() {
            if let Ok(read_dir) = std::fs::read_dir(&apps_dir) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "desktop") {
                        desktop_files.push(path);
                    }
                }
            }
        }
    }

    // Also check user's local applications
    if let Some(data_home) = data_dirs.get_data_home() {
        let local_apps = data_home.join("applications");
        if local_apps.exists() {
            if let Ok(read_dir) = std::fs::read_dir(&local_apps) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "desktop") {
                        desktop_files.push(path);
                    }
                }
            }
        }
    }

    for path in desktop_files {
        let entry = match parse_entry(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Skip if NoDisplay or Hidden
        if get_attr(&entry, "NoDisplay").is_some_and(|v| v == "true") {
            continue;
        }
        if get_attr(&entry, "Hidden").is_some_and(|v| v == "true") {
            continue;
        }

        // Get app ID from filename
        let id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if id.is_empty() {
            continue;
        }

        if exclude.is_match(&id) {
            continue;
        }

        let name = match get_attr(&entry, "Name") {
            Some(n) => n,
            None => continue,
        };

        let command_line = match get_attr(&entry, "Exec") {
            Some(e) => e,
            None => continue,
        };

        let is_terminal = get_attr(&entry, "Terminal").is_some_and(|t| t == "true" || t == "1");
        let icon_path = get_attr(&entry, "Icon").and_then(|icon_name| {
            // Search XDG data dirs and common paths
            lookup_icon(&icon_name, config.icon_size)
        });

        // Build display string
        let (display_string, extra_text) = if let Some(override_name) =
            get_id_field(&id, Field::Id).and_then(|app_id| config.name_overrides.get(&app_id))
        {
            let i = override_name.find('\r');
            (
                override_name.replace('\r', " "),
                i.map(|idx| override_name[idx + 1..].to_string()),
            )
        } else {
            let extra = config
                .extra_field
                .first()
                .and_then(|f| get_app_field(&entry, *f).or_else(|| get_id_field(&id, *f)));
            match extra {
                Some(e)
                    if (!config.hide_extra_if_contained
                        || !name.to_lowercase().contains(&e.to_lowercase())) =>
                {
                    let separator = if config.extra_field_newline {
                        "\n"
                    } else {
                        " "
                    };
                    (format!("{}{}{}", name, separator, e), Some(e))
                }
                _ => (name.clone(), None),
            }
        };

        let hidden = config
            .hidden_fields
            .iter()
            .filter_map(|f| get_app_field(&entry, *f).or_else(|| get_id_field(&id, *f)))
            .collect::<Vec<String>>()
            .join(" ");

        let search_string = if hidden.is_empty() {
            display_string.clone()
        } else {
            format!("{} {}", display_string, hidden)
        };

        let history_data = history.get(&id).copied().unwrap_or_default();
        let last_used = if config.recent_first {
            history_data.last_used
        } else {
            0
        };
        let usage_count = if config.frequent_first {
            history_data.usage_count
        } else {
            0
        };

        entries.push(AppEntry {
            id,
            name,
            display_string,
            search_string,
            extra_text,
            command_line,
            icon_path,
            is_terminal,
            score: 100,
            history: HistoryData {
                last_used,
                usage_count,
            },
        });
    }

    entries.sort();
    entries
}
