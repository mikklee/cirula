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

use crate::util::get_history_file;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, Default, Eq, Deserialize, Serialize)]
pub struct HistoryData {
    pub last_used: u64,
    pub usage_count: u32,
}

impl PartialEq for HistoryData {
    fn eq(&self, other: &Self) -> bool {
        self.last_used.eq(&other.last_used) && self.usage_count.eq(&other.usage_count)
    }
}

pub fn load_history(days: u32) -> HashMap<String, HistoryData> {
    match get_history_file(false) {
        Some(file) => {
            let history_str = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(_) => return HashMap::new(),
            };
            let epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let cutoff = epoch.as_secs() - (days as u64) * 86400;
            let mut history: HashMap<String, HistoryData> = toml::from_str(&history_str)
                .unwrap_or_else(|err| {
                    eprintln!("Cannot parse history file: {}", err);
                    HashMap::new()
                });
            history.retain(|_, data: &mut HistoryData| days == 0 || data.last_used >= cutoff);
            history
        }
        None => HashMap::new(),
    }
}

pub fn save_history(history: &HashMap<String, HistoryData>) {
    let file = match get_history_file(true) {
        Some(f) => f,
        None => {
            eprintln!("Cannot create history file or cache directory");
            return;
        }
    };
    let mut file = match File::create(file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open history file for writing: {}", e);
            return;
        }
    };
    let s = toml::to_string(history).unwrap();
    if let Err(e) = file.write_all(s.as_bytes()) {
        eprintln!("Cannot write to history file: {}", e);
    }
}

pub fn update_history(history: &mut HashMap<String, HistoryData>, id: &str) {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let usage_count = history.get(id).map_or(0, |h| h.usage_count) + 1;

    history.insert(
        id.to_string(),
        HistoryData {
            last_used: epoch.as_secs(),
            usage_count,
        },
    );
}
