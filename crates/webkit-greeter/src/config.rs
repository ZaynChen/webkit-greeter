// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Deserialize;

use crate::constants::{DEFAULT_BACKGROUND_IMAGES_DIR, DEFAULT_THEME, DEFAULT_THEME_DIR};

#[derive(Clone, Default, Debug, Deserialize)]
pub struct Config {
    branding: Branding,
    greeter: Greeter,
    #[serde(default = "default_themes_dir")]
    themes_dir: String,
    primary_monitor: Option<String>,
    // #[serde(default = "Vec::new")]
    // layouts: Vec<String>,
}

pub fn default_themes_dir() -> String {
    DEFAULT_THEME_DIR.clone()
}

impl Config {
    pub fn new(debug: bool, theme: Option<&str>, dm: &str) -> Self {
        let config_path = 
            [
                format!("/usr/local/etc/{dm}/webkit-greeter.toml"),
                format!("/etc/{dm}/webkit-greeter.toml")
            ].into_iter()
                .find(|path| std::path::Path::new(path).is_file())
                .unwrap_or_else(|| panic!("Neither \"/usr/local/etc/{dm}/webkit-greeter.toml\" nor \"/etc/{dm}/webkit-greeter.toml\" exist"));

        let content = std::fs::read_to_string(config_path).expect("Can not read config file");
        let mut config: Config = toml::from_str(&content).expect("config file structure error");
        if debug {
            config.set_debug_mode(true);
        }
        if let Some(theme) = theme {
            config.set_theme(theme);
        }

        logger::debug!("Configuration loaded");
        config
    }

    pub fn debug_mode(&self) -> bool {
        self.greeter.debug_mode
    }

    pub fn detect_theme_errors(&self) -> bool {
        self.greeter.detect_theme_errors
    }

    pub fn screensaver_timeout(&self) -> u32 {
        self.greeter.screensaver_timeout
    }

    pub fn secure_mode(&self) -> bool {
        self.greeter.secure_mode
    }

    pub fn theme(&self) -> &str {
        &self.greeter.theme
    }

    pub fn icon_theme(&self) -> Option<&str> {
        self.greeter.icon_theme.as_deref()
    }

    pub fn time_language(&self) -> Option<&str> {
        self.greeter.time_language.as_deref()
    }

    pub fn background_images_dir(&self) -> &str {
        &self.branding.background_images_dir
    }

    pub fn logo_image(&self) -> &str {
        &self.branding.logo_image
    }

    pub fn user_image(&self) -> &str {
        &self.branding.user_image
    }

    pub fn themes_dir(&self) -> &str {
        &self.themes_dir
    }

    pub fn primary_monitor(&self) -> Option<&str> {
        self.primary_monitor.as_deref()
    }

    // pub fn layouts(&self) -> &[String] {
    //     self.layouts.as_slice()
    // }

    fn set_debug_mode(&mut self, debug_mode: bool) {
        self.greeter.debug_mode |= debug_mode;
    }

    fn set_theme(&mut self, theme: &str) {
        self.greeter.theme = theme.to_string();
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Branding {
    background_images_dir: String,
    logo_image: String,
    user_image: String,
}

impl Default for Branding {
    fn default() -> Self {
        Self {
            background_images_dir: DEFAULT_BACKGROUND_IMAGES_DIR.to_string(),
            logo_image: Default::default(),
            user_image: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Greeter {
    debug_mode: bool,
    detect_theme_errors: bool,
    screensaver_timeout: u32,
    secure_mode: bool,
    theme: String,
    icon_theme: Option<String>,
    time_language: Option<String>,
}

impl Default for Greeter {
    fn default() -> Self {
        Self {
            debug_mode: false,
            detect_theme_errors: true,
            screensaver_timeout: 300,
            secure_mode: true,
            theme: DEFAULT_THEME.to_string(),
            icon_theme: Default::default(),
            time_language: Default::default(),
        }
    }
}
