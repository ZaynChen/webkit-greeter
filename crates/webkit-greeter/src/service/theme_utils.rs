// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::glib::{self, tmp_dir, variant::ToVariant};

pub(super) struct ThemeUtils {
    allowed_dirs: Vec<String>,
}

impl ThemeUtils {
    pub(super) fn new(allowed_dirs: &[String], theme: &str) -> Self {
        let mut allowed_dirs: Vec<String> = allowed_dirs.iter().map(|s| s.to_string()).collect();
        if let Ok(path) = std::fs::canonicalize(theme) {
            let theme_dir = path.with_file_name("");
            allowed_dirs.push(theme_dir.to_string_lossy().to_string());
        }
        allowed_dirs.push(tmp_dir().to_string_lossy().to_string());
        Self { allowed_dirs }
    }

    pub(super) fn handle(&self, method: &str, json_args: &str) -> glib::Variant {
        let json_result = if "dirlist" == method && json_args != "[]" {
            self.dirlist(json_args)
        } else {
            "undefined".to_string()
        };
        json_result.to_variant()
    }

    fn dirlist(&self, json_args: &str) -> String {
        let args: serde_json::Value =
            serde_json::from_str(json_args).expect("args is not a JSON string");
        let (path, only_images) = if let serde_json::Value::Array(arr) = args
            && arr.len() == 2
            && let serde_json::Value::String(path) = arr[0].clone()
            && path != "/"
            && !path.starts_with("./")
            && let serde_json::Value::Bool(only_images) = arr[1]
        {
            (path, only_images)
        } else {
            return "[]".to_string();
        };

        let resolved = if let Ok(p) = std::fs::canonicalize(&path) {
            p
        } else {
            return "[]".to_string();
        };

        if !resolved.is_absolute() || !resolved.is_dir() {
            println!("{resolved:?} is not absolute nor an existing directory");
            return "[]".to_string();
        }

        if self.allowed_dirs.iter().all(|d| resolved.starts_with(d)) {
            logger::warn!("Path {resolved:?} is not allowed");
            return "[]".to_string();
        }

        let dir = match std::fs::read_dir(&resolved) {
            Ok(d) => d,
            Err(e) => {
                println!("Opendir error: '{e}'");
                return "[]".to_string();
            }
        };

        let mut files = vec![];
        let regex = glib::Regex::new(
            ".+\\.(jpe?g|png|gif|bmp|webp)",
            glib::RegexCompileFlags::CASELESS,
            glib::RegexMatchFlags::DEFAULT,
        )
        .expect("g_regex_new error")
        .expect("g_regex_new error");
        for entry in dir.filter_map(|v| v.ok()) {
            let filename = entry.file_name();
            if filename == "." || filename == ".." {
                continue;
            }

            let filepath = std::path::PathBuf::from("/")
                .join(&resolved)
                .join(&filename);
            let file_element = filepath.to_string_lossy().to_string();
            if only_images {
                let s = glib::GStr::from_str_with_nul(filename.to_str().unwrap())
                    .expect("osstring to gstr error");
                if let Ok(ft) = entry.file_type()
                    && ft.is_file()
                    && regex
                        .match_(s, glib::RegexMatchFlags::DEFAULT)
                        .expect("g_regex_match error")
                        .matches()
                {
                    files.push(file_element);
                }
            } else {
                files.push(file_element);
            }
        }
        serde_json::to_string(&files).expect("Failed to convert vec to json string")
    }
}
