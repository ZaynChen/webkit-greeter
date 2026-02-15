// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use serde::Serialize;

use std::sync::OnceLock;

use super::dbus::AccountsService;

#[derive(Debug, Serialize)]
pub struct User {
    home_directory: Option<String>,
    #[serde(rename(serialize = "image"))]
    icon_file: Option<String>,
    language: Option<String>,
    #[serde(rename(serialize = "display_name"))]
    real_name: Option<String>,
    session: Option<String>,
    #[serde(skip_serializing)]
    uid: Option<u64>,
    #[serde(rename(serialize = "username"))]
    user_name: Option<String>,
}

impl User {
    fn new(
        home_directory: Option<String>,
        icon_file: Option<String>,
        language: Option<String>,
        real_name: Option<String>,
        session: Option<String>,
        uid: Option<u64>,
        user_name: Option<String>,
    ) -> Self {
        Self {
            home_directory,
            icon_file,
            language,
            real_name,
            session,
            uid,
            user_name,
        }
    }
    /// HomeDirectory property
    pub fn home_directory(&self) -> Option<&str> {
        self.home_directory.as_deref()
    }

    /// IconFile property
    pub fn icon_file(&self) -> Option<&str> {
        self.icon_file.as_deref()
    }

    /// Language property
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// RealName property
    pub fn real_name(&self) -> Option<&str> {
        self.real_name.as_deref()
    }

    /// Session property
    pub fn session(&self) -> Option<&str> {
        self.session.as_deref()
    }

    /// Uid property
    pub fn uid(&self) -> Option<u64> {
        self.uid
    }

    /// UserName property
    pub fn user_name(&self) -> Option<&str> {
        self.user_name.as_deref()
    }
}

pub struct UserManager {
    users: Vec<User>,
}

impl UserManager {
    pub fn instance() -> &'static Self {
        static USER_MANAGER: OnceLock<UserManager> = OnceLock::new();
        USER_MANAGER.get_or_init(|| {
            let accounts_proxy = AccountsService::accounts_proxy();
            let users: Vec<_> = accounts_proxy
                .list_cached_users()
                .unwrap()
                .into_iter()
                .map(|o| {
                    let user = AccountsService::user_proxy(o);
                    User::new(
                        user.home_directory().ok(),
                        user.icon_file().ok(),
                        user.language().ok(),
                        user.real_name().ok(),
                        user.session().ok(),
                        user.uid().ok(),
                        user.user_name().ok(),
                    )
                })
                .collect();
            Self { users }
        })
    }

    pub fn list_users(&self) -> &[User] {
        &self.users
    }

    pub fn set_language(username: &str, language: &str) -> Result<(), String> {
        AccountsService::accounts_proxy()
            .find_user_by_name(username)
            .and_then(|o| AccountsService::user_proxy(o).set_language(language))
            .map_err(|e| e.to_string())
    }
}
