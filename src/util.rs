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

use crate::consts::*;
use shlex::Shlex;
use std::path::PathBuf;
use std::process::{id, Command};

pub fn get_xdg_dirs() -> xdg::BaseDirectories {
    xdg::BaseDirectories::with_prefix(APP_NAME)
}

pub fn get_config_file(file: &str) -> Option<PathBuf> {
    get_xdg_dirs().find_config_file(file)
}

pub fn get_history_file(place: bool) -> Option<PathBuf> {
    let xdg = get_xdg_dirs();
    if place {
        xdg.place_cache_file(HISTORY_FILE).ok()
    } else {
        xdg.find_cache_file(HISTORY_FILE)
    }
}

pub fn is_cmd(text: &str, cmd_prefix: &str) -> bool {
    !cmd_prefix.is_empty() && text.starts_with(cmd_prefix)
}

// Spawned child is not awaited.
// The zombie_processes lint warns about resource exhaustion in long-running applications:
// https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/zombie_processes.rs
// I believe, based on the man7 doc, that the process will be reaped by the system on Linux:
// https://man7.org/linux/man-pages/man2/exit.2.html
// I cannot confirm/deny whether this may have unforeseen consequences on Linux nor other systems.
#[allow(clippy::zombie_processes)]
pub fn launch_cmd(cmd_line: &str) {
    let parts: Vec<String> = Shlex::new(cmd_line).collect();
    if parts.is_empty() {
        return;
    }

    let mut child = Command::new(&parts[0]);
    child.args(&parts[1..]);
    child.spawn().expect("Error spawning command");
}

pub fn launch_app(
    command_line: &str,
    is_terminal: bool,
    app_id: &str,
    term_command: Option<&str>,
    launch_cgroups: bool,
) {
    let command_string = command_line
        .replace("%U", "")
        .replace("%F", "")
        .replace("%u", "")
        .replace("%f", "");
    let mut command: Vec<String> = Shlex::new(&command_string).collect();

    if is_terminal {
        if let Some(term) = term_command {
            let command_string = term.to_string().replace("{}", &command_string);
            command = Shlex::new(&command_string).collect();
        } else if let Some(term) = std::env::var_os("TERMINAL") {
            let term = term.into_string().expect("couldn't convert to string");
            let mut command_new = vec![term, "-e".into()];
            command_new.extend(command);
            command = command_new;
        } else {
            return;
        };
    }

    if launch_cgroups {
        let mut name = app_id.to_string();
        if name.ends_with(".desktop") {
            name.truncate(name.len() - 8);
        }
        let parsed = Command::new("systemd-escape")
            .arg(&name)
            .output()
            .unwrap()
            .stdout;
        let unit = format!(
            "--unit=app-sirula-{}-{}",
            String::from_utf8_lossy(&parsed).trim(),
            id()
        );
        let mut command_new: Vec<String> = vec![
            "systemd-run".into(),
            "--scope".into(),
            "--user".into(),
            unit,
        ];
        command_new.extend(command);
        command = command_new;
    }

    // Spawned child is not awaited.
    // The zombie_processes lint warns about resource exhaustion in long-running applications:
    // https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/zombie_processes.rs
    // I believe, based on the man7 doc, that the process will be reaped by the system on Linux:
    // https://man7.org/linux/man-pages/man2/exit.2.html
    // I cannot confirm/deny whether this may have unforeseen consequences on Linux nor other systems.
    #[allow(clippy::zombie_processes)]
    if !command.is_empty() {
        Command::new(&command[0])
            .args(&command[1..])
            .spawn()
            .expect("Error launching app");
    }
}
