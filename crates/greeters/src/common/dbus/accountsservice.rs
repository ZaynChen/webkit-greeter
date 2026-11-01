// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use std::sync::OnceLock;

use super::system_conn;

pub struct AccountsService;
impl AccountsService {
    pub fn accounts_proxy() -> &'static Option<AccountsProxy<'static>> {
        static ACCOUNTS_PROXY: OnceLock<Option<AccountsProxy>> = OnceLock::new();
        ACCOUNTS_PROXY.get_or_init(|| zbus::block_on(AccountsProxy::new(system_conn())).ok())
    }

    pub fn user_proxy(o: zbus::OwnedObjectPath) -> UserProxy<'static> {
        zbus::block_on(UserProxy::builder(system_conn()).path(o).build()).unwrap()
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.Accounts",
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts"
)]
pub trait Accounts {
    /// FindUserById method
    fn find_user_by_id(&self, id: i64) -> zbus::Result<zbus::OwnedObjectPath>;

    /// FindUserByName method
    fn find_user_by_name(&self, name: &str) -> zbus::Result<zbus::OwnedObjectPath>;

    /// ListCachedUsers method
    fn list_cached_users(&self) -> zbus::Result<Vec<zbus::OwnedObjectPath>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.Accounts.User",
    default_service = "org.freedesktop.Accounts"
)]
pub trait User {
    /// AccountType property
    #[zbus(property)]
    fn account_type(&self) -> zbus::Result<i32>;

    /// AutomaticLogin property
    #[zbus(property)]
    fn automatic_login(&self) -> zbus::Result<bool>;

    /// Email property
    #[zbus(property)]
    fn email(&self) -> zbus::Result<String>;

    /// HomeDirectory property
    #[zbus(property)]
    fn home_directory(&self) -> zbus::Result<String>;

    /// IconFile property
    #[zbus(property)]
    fn icon_file(&self) -> zbus::Result<String>;

    /// Language property
    #[zbus(property)]
    fn language(&self) -> zbus::Result<String>;

    /// Languages property
    #[zbus(property)]
    fn languages(&self) -> zbus::Result<Vec<String>>;

    /// LocalAccount property
    #[zbus(property)]
    fn local_account(&self) -> zbus::Result<bool>;

    /// Location property
    #[zbus(property)]
    fn location(&self) -> zbus::Result<String>;

    /// Locked property
    #[zbus(property)]
    fn locked(&self) -> zbus::Result<bool>;

    /// LoginFrequency property
    #[zbus(property)]
    fn login_frequency(&self) -> zbus::Result<u64>;

    /// LoginTime property
    #[zbus(property)]
    fn login_time(&self) -> zbus::Result<i64>;

    /// PasswordHint property
    #[zbus(property)]
    fn password_hint(&self) -> zbus::Result<String>;

    /// PasswordMode property
    #[zbus(property)]
    fn password_mode(&self) -> zbus::Result<i32>;

    /// RealName property
    #[zbus(property)]
    fn real_name(&self) -> zbus::Result<String>;

    /// Saved property
    #[zbus(property)]
    fn saved(&self) -> zbus::Result<bool>;

    /// Session property
    #[zbus(property)]
    fn session(&self) -> zbus::Result<String>;

    /// SessionType property
    #[zbus(property)]
    fn session_type(&self) -> zbus::Result<String>;

    /// Shell property
    #[zbus(property)]
    fn shell(&self) -> zbus::Result<String>;

    /// SystemAccount property
    #[zbus(property)]
    fn system_account(&self) -> zbus::Result<bool>;

    /// Uid property
    #[zbus(property)]
    fn uid(&self) -> zbus::Result<u64>;

    /// UserName property
    #[zbus(property)]
    fn user_name(&self) -> zbus::Result<String>;

    /// XSession property
    #[zbus(property, name = "XSession")]
    fn xsession(&self) -> zbus::Result<String>;
}
