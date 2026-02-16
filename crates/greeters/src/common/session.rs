// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use serde::Serialize;
use webkit::glib::{
    KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_HIDDEN, KEY_FILE_DESKTOP_KEY_NO_DISPLAY,
    KEY_FILE_DESKTOP_KEY_TRY_EXEC, KeyFile, KeyFileFlags, find_program_in_path, system_data_dirs,
};

use std::{collections::HashMap, fs::read_dir, path::PathBuf, sync::OnceLock};

#[derive(Debug, Serialize)]
pub struct Session {
    key: String,
    #[serde(rename(serialize = "type"))]
    type_: String,
    name: String,
    comment: String,
    #[serde(skip_serializing)]
    exec: String,
}

impl Session {
    fn new(key: String, type_: String, name: String, comment: String, exec: String) -> Self {
        Self {
            key,
            type_,
            name,
            comment,
            exec,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn type_(&self) -> &str {
        &self.type_
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn exec(&self) -> &str {
        &self.exec
    }
}

pub struct SessionManager;
impl SessionManager {
    // TODO: handle duplicated sessions
    fn available_sessions_map() -> &'static HashMap<String, Session> {
        static SESSION_FILES: OnceLock<HashMap<String, Session>> = OnceLock::new();
        SESSION_FILES.get_or_init(|| {
            system_data_dirs() // ["/usr/local/share", "/usr/share"]
                .iter()
                .flat_map(|dir| {
                    [
                        load_session_dir(dir.join("xsessions"), "x"),
                        load_session_dir(dir.join("wayland-sessions"), "wayland"),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<HashMap<_, _>>()
                })
                .collect()
        })
    }

    pub fn sessions() -> Vec<&'static Session> {
        let mut sessions: Vec<_> = Self::available_sessions_map().values().collect();
        sessions.sort_by_key(|s| s.key());
        sessions
    }

    pub fn session(key: &str) -> Option<&Session> {
        Self::available_sessions_map().get(key)
    }
}

fn is_session_desktop_file(keyfile: &KeyFile) -> bool {
    let no_display = keyfile
        .boolean(KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_NO_DISPLAY)
        .is_ok_and(|no_display| no_display);
    let hidden = keyfile
        .boolean(KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_HIDDEN)
        .is_ok_and(|hidden| hidden);
    let tryexec_failed = keyfile
        .string(KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_TRY_EXEC)
        .is_ok_and(|try_exec| find_program_in_path(try_exec).is_none());
    !no_display && !hidden && !tryexec_failed
}

fn load_session_dir(dir: PathBuf, session_type: &str) -> HashMap<String, Session> {
    if !dir.is_dir() {
        return HashMap::with_capacity(0);
    }
    read_dir(dir)
        .unwrap()
        .filter_map(|ent| ent.ok())
        .filter_map(|file| {
            let keyfile = KeyFile::new();
            let filepath = file.path();
            let filepath_str = filepath.to_str().unwrap();
            if let Err(e) = keyfile.load_from_file(&filepath, KeyFileFlags::NONE) {
                logger::warn!("Failed to load \"{filepath_str}\": {e}");
            } else if keyfile.has_group(KEY_FILE_DESKTOP_GROUP) {
                if !is_session_desktop_file(&keyfile) {
                    logger::warn!(
                        "\"{filepath_str}\" is hidden, {}, {}",
                        "contains non-executable TryExec program",
                        "or is otherwise not capable of being used"
                    );
                } else if keyfile
                    .has_key(KEY_FILE_DESKTOP_GROUP, "Name")
                    .is_ok_and(|b| b)
                    && keyfile
                        .has_key(KEY_FILE_DESKTOP_GROUP, "Exec")
                        .is_ok_and(|b| b)
                {
                    let key = file
                        .file_name()
                        .to_string_lossy()
                        .trim_end_matches(".desktop")
                        .to_string();
                    let name = keyfile
                        .locale_string(KEY_FILE_DESKTOP_GROUP, "Name", None)
                        .unwrap()
                        .into();
                    let comment = keyfile
                        .locale_string(KEY_FILE_DESKTOP_GROUP, "Comment", None)
                        .unwrap_or_default()
                        .into();
                    let exec = keyfile
                        .string(KEY_FILE_DESKTOP_GROUP, "Exec")
                        .unwrap_or_default()
                        .into();
                    return Some((
                        key.clone(),
                        Session::new(key, session_type.into(), name, comment, exec),
                    ));
                } else {
                    logger::warn!("{filepath_str} contains no \"Name\" or \"Exec\" key");
                }
            }
            None
        })
        .collect()
}
