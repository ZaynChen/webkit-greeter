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
    uid: Option<u32>,
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
        uid: Option<u32>,
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
    pub fn uid(&self) -> Option<u32> {
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
            let mut users = match AccountsService::accounts_proxy() {
                Some(accounts_proxy) => accounts_proxy
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
                            user.uid().map(|u| u as u32).ok(),
                            user.user_name().ok(),
                        )
                    })
                    .collect(),
                None => vec![],
            };
            let pwd_users: Vec<_> = pwd::Passwd::iter()
                .map(|u| {
                    User::new(
                        Some(u.dir),
                        None,
                        None,
                        u.gecos,
                        None,
                        Some(u.uid),
                        Some(u.name),
                    )
                })
                .filter(|u| {
                    let (uid_min, uid_max) = get_uid_minmax();
                    u.uid().is_some_and(|uid| uid_min <= uid && uid <= uid_max)
                })
                .collect();

            let uids: Vec<_> = users.iter().map(|u| u.uid()).collect();
            users.extend(
                pwd_users
                    .into_iter()
                    .filter(|u| !uids.contains(&u.uid()))
                    .collect::<Vec<_>>(),
            );
            Self { users }
        })
    }

    pub fn list_users(&self) -> &[User] {
        &self.users
    }

    pub fn set_language(username: &str, language: &str) -> Result<(), String> {
        match AccountsService::accounts_proxy() {
            Some(proxy) => proxy
                .find_user_by_name(username)
                .and_then(|o| AccountsService::user_proxy(o).set_language(language))
                .map_err(|e| e.to_string()),
            None => Err("Failed to connect to accountsservice".to_string()),
        }
    }
}

fn get_uid_minmax() -> (u32, u32) {
    match std::fs::read_to_string("/etc/login.defs") {
        Ok(content) => {
            let minmax: Vec<_> = content
                .lines()
                .filter(|l| l.starts_with("UID_MIN") || l.starts_with("UID_MAX"))
                .filter_map(|s| {
                    s.trim_start_matches("UID_MIN")
                        .trim_start_matches("UID_MAX")
                        .trim()
                        .parse::<u32>()
                        .ok()
                })
                .collect();
            if minmax.len() == 2 {
                (minmax[0], minmax[1])
            } else {
                log::error!("/etc/login.defs is broken, using default UID_MIN=1000, UID_MAX=60000");
                (1000, 60000)
            }
        }
        Err(_) => {
            log::error!(
                "Failed to read /etc/login.defs, using default UID_MIN=1000, UID_max=60000"
            );
            (1000, 60000)
        }
    }
}
