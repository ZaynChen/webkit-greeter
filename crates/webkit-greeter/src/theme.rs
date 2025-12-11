// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::constants::{DEFAULT_THEME, DEFAULT_THEME_DIR};

pub fn print_themes(themes_dir: &str) {
    let themes = list_themes(themes_dir);
    if themes.is_empty() {
        return;
    }

    println!("Themes are located at {themes_dir}\n");
    themes.iter().for_each(|t| println!("- {t}"));
}

fn list_themes(themes_dir: &str) -> Vec<String> {
    let mut themes = match std::fs::read_dir(themes_dir) {
        Ok(dir) => dir
            .filter_map(|ent| ent.ok())
            .filter(|ent| ent.file_type().is_ok_and(|ft| ft.is_dir()))
            .map(|ent| ent.file_name().to_string_lossy().to_string())
            .collect(),
        Err(_) => {
            println!("Threre are no themes located at {themes_dir}");
            vec![]
        }
    };
    themes.sort();
    themes
}

pub fn load_theme_html(theme_dir: &Path) -> (String, String) {
    let (primary, secondary) = load_theme_config(theme_dir);

    let primary_path = theme_dir.join(&primary);
    let primary_html = if primary_path.is_file() && primary.ends_with(".html") {
        primary_path.to_string_lossy().to_string()
    } else {
        PathBuf::from(DEFAULT_THEME_DIR.clone())
            .join(DEFAULT_THEME)
            .join("index.html")
            .to_string_lossy()
            .to_string()
    };

    let secondary_html = secondary
        .filter(|s| s.ends_with(".html"))
        .map(|s| theme_dir.join(s))
        .filter(|path| path.is_file())
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or(primary_html.clone());

    (primary_html, secondary_html)
}

fn load_theme_config(theme_dir: &Path) -> (String, Option<String>) {
    match std::fs::read_to_string(theme_dir.join("index.yml")) {
        Ok(content) => {
            let config_map: HashMap<_, _> = content
                .lines()
                .map(str::trim)
                .filter(|l| l.starts_with("primary_html") || l.starts_with("secondary_html"))
                .filter_map(|s| s.split_once(':'))
                .map(|(k, v)| (k.trim(), v.trim().trim_matches(['\"', '\''])))
                .collect();
            if config_map.is_empty() {
                logger::error!("Failed to read theme config file, use default setting");
            }
            let primary = config_map
                .get("primary_html")
                .unwrap_or(&"index.html")
                .to_string();
            let secondary = config_map.get("secondary_html");
            if let Some(s) = secondary {
                (primary, Some(s.to_string()))
            } else {
                (primary, None)
            }
        }
        Err(e) => {
            logger::error!("Theme config was not loaded:\n\t{e}");
            ("index.html".to_string(), None)
        }
    }
}
