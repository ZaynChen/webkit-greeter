// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use gettextrs::{LocaleCategory, dgettext, setlocale};

use std::{ffi::CStr, process::Command, sync::OnceLock};

#[derive(Debug, Clone)]
pub struct Language {
    code: String,
    name: String,
    territory: String,
}

impl Language {
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn territory(&self) -> &str {
        &self.territory
    }
}

pub struct LanguageManager;
impl LanguageManager {
    pub fn current() -> Option<Language> {
        let lang = std::env::var("LANG");
        match lang {
            Ok(code) => {
                let prefix = code.split('.').next();
                Self::languages()
                    .iter()
                    .find(|l| l.code.split('.').next() == prefix)
                    .cloned()
            }
            Err(_) => None,
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
                Ok(output) => String::from_utf8(output.stdout)
                    .expect("The output of 'locale -a' is not encoded as utf8")
                    .split('\n')
                    .filter(|code| code.ends_with(".utf8"))
                    .map(|code| {
                        let current = setlocale(LocaleCategory::LcAll, []).unwrap();

                        setlocale(LocaleCategory::LcIdentification, code);
                        setlocale(LocaleCategory::LcMessages, "");

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

                        setlocale(LocaleCategory::LcAll, current);
                        Language {
                            code: code.to_string(),
                            name,
                            territory,
                        }
                    })
                    .collect::<Vec<_>>(),
                Err(e) => {
                    logger::error!("Failed to run 'locale -a': {e}");
                    Vec::with_capacity(0)
                }
            }
        })
    }
}
