// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use std::sync::OnceLock;

use zbus::{blocking::Connection, proxy};

use super::constants::LOGIN_UID_MINMAX;

pub struct User {
    home_directory: Option<String>,
    icon_file: Option<String>,
    language: Option<String>,
    real_name: Option<String>,
    session: Option<String>,
    uid: Option<u64>,
    user_name: Option<String>,
}

impl User {
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

impl<'p> From<UserProxyBlocking<'p>> for User {
    fn from(user: UserProxyBlocking) -> Self {
        let home_directory = user.home_directory().ok();
        let icon_file = user.icon_file().ok();
        let language = user.language().ok();
        let real_name = user.real_name().ok();
        let session = user.session().ok();
        let uid = user.uid().ok();
        let user_name = user.user_name().ok();
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
}

pub struct UserManager {
    users: Vec<User>,
}

impl UserManager {
    pub fn instance() -> &'static Self {
        static USER_MANAGER: OnceLock<UserManager> = OnceLock::new();
        USER_MANAGER.get_or_init(|| {
            let (uid_min, uid_max) = *LOGIN_UID_MINMAX;
            logger::warn!("UID_MIN={uid_min}, UID_MAX={uid_max}");
            // TODO: passwd
            let conn = Connection::system().unwrap();
            let accounts = AccountsProxyBlocking::new(&conn).unwrap();
            let users: Vec<_> = accounts
                .list_cached_users()
                .unwrap()
                .into_iter()
                .map(|o| {
                    UserProxyBlocking::builder(&conn)
                        .path(o.clone())
                        .unwrap()
                        .build()
                        .unwrap()
                        .into()
                })
                .collect();
            Self { users }
        })
    }

    pub fn list_users(&self) -> &[User] {
        &self.users
    }
}

#[proxy(
    interface = "org.freedesktop.Accounts",
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts"
)]
pub trait Accounts {
    /// ListCachedUsers method
    fn list_cached_users(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.Accounts.User",
    default_service = "org.freedesktop.Accounts"
)]
pub trait User {
    /// HomeDirectory property
    #[zbus(property)]
    fn home_directory(&self) -> zbus::Result<String>;

    /// IconFile property
    #[zbus(property)]
    fn icon_file(&self) -> zbus::Result<String>;

    /// Language property
    #[zbus(property)]
    fn language(&self) -> zbus::Result<String>;

    /// RealName property
    #[zbus(property)]
    fn real_name(&self) -> zbus::Result<String>;

    /// Session property
    #[zbus(property)]
    fn session(&self) -> zbus::Result<String>;

    /// Uid property
    #[zbus(property)]
    fn uid(&self) -> zbus::Result<u64>;

    /// UserName property
    #[zbus(property)]
    fn user_name(&self) -> zbus::Result<String>;
}
