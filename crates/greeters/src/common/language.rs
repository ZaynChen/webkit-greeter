// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use gettextrs::{LocaleCategory, dgettext, setlocale};
use serde::Serialize;

use std::{ffi::CStr, process::Command, sync::OnceLock};

#[derive(Debug, Clone, Serialize)]
pub struct Language {
    code: String,
    name: String,
    territory: String,
}

pub struct LanguageManager;
impl LanguageManager {
    pub fn current() -> Option<Language> {
        if let Ok(code) = std::env::var("LANG")
            && (code.ends_with(".utf8") || code.ends_with(".UTF-8"))
        {
            let lang = code.split('.').next();
            Self::languages()
                .iter()
                .find(|l| l.code.split('.').next() == lang)
                .cloned()
        } else {
            None
        }
    }

    pub fn languages() -> &'static [Language] {
        static LANGUAGES: OnceLock<Vec<Language>> = OnceLock::new();
        LANGUAGES.get_or_init(|| {
            const _NL_IDENTIFICATION_LANGUAGE: i32 = 786439;
            const _NL_IDENTIFICATION_TERRITORY: i32 = 786440;
            gettextrs::bindtextdomain("iso_639-3", "/usr/share/locale").unwrap();
            gettextrs::bindtextdomain("iso_3166-1", "/usr/share/locale").unwrap();

            match Command::new("locale").arg("-a").output() {
                Ok(output) => String::from_utf8(output.stdout).map_or_default(|s| {
                    s.split('\n')
                        .filter(|code| code.ends_with(".utf8"))
                        .map(|code| {
                            let current = unsafe { setlocale(LocaleCategory::LcAll, []) }.unwrap();

                            unsafe {
                                setlocale(LocaleCategory::LcIdentification, code);
                                setlocale(LocaleCategory::LcMessages, "");
                            }

                            let language_en = unsafe {
                                CStr::from_ptr(libc::nl_langinfo(_NL_IDENTIFICATION_LANGUAGE))
                                    .to_string_lossy()
                                    .to_string()
                            };
                            let name = if language_en.is_empty() {
                                code.split(['_', '.', '@']).next().unwrap().to_string()
                            } else {
                                dgettext("iso_639-3", &language_en)
                            };
                            let territory_en = unsafe {
                                CStr::from_ptr(libc::nl_langinfo(_NL_IDENTIFICATION_TERRITORY))
                                    .to_string_lossy()
                                    .to_string()
                            };

                            let territory = if !code.contains('_') {
                                String::new()
                            } else if territory_en.is_empty() {
                                code.split(['_', '.', '@']).nth(1).unwrap().to_string()
                            } else {
                                dgettext("iso_3166-1", &territory_en)
                            };

                            unsafe {
                                setlocale(LocaleCategory::LcAll, current);
                            }
                            Language {
                                code: code.to_string(),
                                name,
                                territory,
                            }
                        })
                        .collect::<Vec<_>>()
                }),
                Err(e) => {
                    log::error!("Failed to run 'locale -a': {e}");
                    Vec::default()
                }
            }
        })
    }
}
