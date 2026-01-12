use serde::{Deserialize, Serialize};
use zbus::{
    proxy,
    zvariant::{OwnedObjectPath, OwnedValue, Type, Value, as_value},
};

use std::{collections::HashMap, sync::OnceLock};

use super::system_conn;

pub struct AccountsService;
impl AccountsService {
    pub fn accounts_proxy() -> &'static AccountsProxyBlocking<'static> {
        static ACCOUNTS_PROXY: OnceLock<AccountsProxyBlocking> = OnceLock::new();
        ACCOUNTS_PROXY.get_or_init(|| AccountsProxyBlocking::new(system_conn()).unwrap())
    }

    pub fn user_proxy(o: OwnedObjectPath) -> UserProxyBlocking<'static> {
        UserProxyBlocking::builder(system_conn())
            .path(o)
            .unwrap()
            .build()
            .unwrap()
    }
}

#[proxy(
    interface = "org.freedesktop.Accounts",
    default_service = "org.freedesktop.Accounts",
    default_path = "/org/freedesktop/Accounts"
)]
pub trait Accounts {
    /// CacheUser method
    fn cache_user(&self, name: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// CreateUser method
    fn create_user(
        &self,
        name: &str,
        fullname: &str,
        account_type: i32,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// DeleteUser method
    fn delete_user(&self, id: i64, remove_files: bool) -> zbus::Result<()>;

    /// FindUserById method
    fn find_user_by_id(&self, id: i64) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// FindUserByName method
    fn find_user_by_name(&self, name: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// GetUsersLanguages method
    fn get_users_languages(&self) -> zbus::Result<Vec<String>>;

    /// ListCachedUsers method
    fn list_cached_users(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// UncacheUser method
    fn uncache_user(&self, name: &str) -> zbus::Result<()>;

    /// UserAdded signal
    #[zbus(signal)]
    fn user_added(&self, user: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// UserDeleted signal
    #[zbus(signal)]
    fn user_deleted(&self, user: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// AutomaticLoginUsers property
    #[zbus(property)]
    fn automatic_login_users(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// DaemonVersion property
    #[zbus(property)]
    fn daemon_version(&self) -> zbus::Result<String>;

    /// HasMultipleUsers property
    #[zbus(property)]
    fn has_multiple_users(&self) -> zbus::Result<bool>;

    /// HasNoUsers property
    #[zbus(property)]
    fn has_no_users(&self) -> zbus::Result<bool>;
}

#[derive(Debug, Type, Value, OwnedValue, Deserialize, Serialize)]
#[zvariant(signature = "dict")]
struct LoginHistory {
    #[serde(with = "as_value")]
    login_time: i64,
    #[serde(with = "as_value")]
    logout_time: i64,
    #[serde(flatten)]
    extra: HashMap<String, OwnedValue>,
}

#[proxy(
    interface = "org.freedesktop.Accounts.User",
    default_service = "org.freedesktop.Accounts"
)]
pub trait User {
    /// GetPasswordExpirationPolicy method
    fn get_password_expiration_policy(&self) -> zbus::Result<(i64, i64, i64, i64, i64, i64)>;

    /// SetAccountType method
    fn set_account_type(&self, account_type: i32) -> zbus::Result<()>;

    /// SetAutomaticLogin method
    fn set_automatic_login(&self, enabled: bool) -> zbus::Result<()>;

    /// SetEmail method
    fn set_email(&self, email: &str) -> zbus::Result<()>;

    /// SetHomeDirectory method
    fn set_home_directory(&self, homedir: &str) -> zbus::Result<()>;

    /// SetIconFile method
    fn set_icon_file(&self, filename: &str) -> zbus::Result<()>;

    /// SetLanguage method
    fn set_language(&self, language: &str) -> zbus::Result<()>;

    /// SetLanguages method
    fn set_languages(&self, languages: &[&str]) -> zbus::Result<()>;

    /// SetLocation method
    fn set_location(&self, location: &str) -> zbus::Result<()>;

    /// SetLocked method
    fn set_locked(&self, locked: bool) -> zbus::Result<()>;

    /// SetPassword method
    fn set_password(&self, password: &str, hint: &str) -> zbus::Result<()>;

    /// SetPasswordExpirationPolicy method
    fn set_password_expiration_policy(
        &self,
        min_days_between_changes: i64,
        max_days_between_changes: i64,
        days_to_warn: i64,
        days_after_expiration_until_lock: i64,
    ) -> zbus::Result<()>;

    /// SetPasswordHint method
    fn set_password_hint(&self, hint: &str) -> zbus::Result<()>;

    /// SetPasswordMode method
    fn set_password_mode(&self, mode: i32) -> zbus::Result<()>;

    /// SetRealName method
    fn set_real_name(&self, name: &str) -> zbus::Result<()>;

    /// SetSession method
    fn set_session(&self, session: &str) -> zbus::Result<()>;

    /// SetSessionType method
    fn set_session_type(&self, session_type: &str) -> zbus::Result<()>;

    /// SetShell method
    fn set_shell(&self, shell: &str) -> zbus::Result<()>;

    /// SetUserExpirationPolicy method
    fn set_user_expiration_policy(&self, expiration_time: i64) -> zbus::Result<()>;

    /// SetUserName method
    fn set_user_name(&self, name: &str) -> zbus::Result<()>;

    /// SetXSession method
    #[zbus(name = "SetXSession")]
    fn set_xsession(&self, x_session: &str) -> zbus::Result<()>;

    /// Changed signal
    #[zbus(signal)]
    fn changed(&self) -> zbus::Result<()>;

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

    /// LoginHistory property
    #[zbus(property)]
    fn login_history(&self) -> zbus::Result<Vec<LoginHistory>>;

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
