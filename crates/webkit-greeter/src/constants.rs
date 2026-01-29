// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

pub const APPLICATION_ID: &str = "com.github.zaynchen.webkit-greeter";

pub const DEFAULT_BACKGROUND_IMAGES_DIR: &str = "/usr/share/backgrounds";

pub const DEFAULT_THEME: &str = "litarvan";

pub const GREETER_RESOURCE_PREFIX: &str = "/com/github/zaynchen/webkit-greeter";

pub const WEBKIT_APPLICATION_INFO: &str = "com.github.zaynchen.webkit-greeter";

lazy_static::lazy_static! {
    pub static ref DEFAULT_THEME_DIR: String = {
        gtk::glib::system_data_dirs()
            .iter()
            .map(|s| s.join("webkit-greeter/themes"))
            .find(|s| s.is_dir())
            .expect("Neither \"/usr/local/share/webkit-greeter/themes\" nor \"/usr/share/webkit-greeter/themes\" exist")
            .to_string_lossy()
            .to_string()
    };

    pub static ref WEB_EXTENSIONS_DIR: &'static str = {
        ["/usr/local/lib/webkit-greeter", "/usr/lib/webkit-greeter"]
            .iter()
            .find(|dir| std::path::Path::new(dir).is_dir())
            .expect("Neither \"/usr/lib/webkit-greeter\" nor \"/usr/local/lib/webkit-greeter\" exist")
    };
}
