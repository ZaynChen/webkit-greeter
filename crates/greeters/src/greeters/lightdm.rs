// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use lightdm::prelude::*;
use webkit::{
    WebView,
    glib::{Variant, clone, variant::ToVariant},
};

use crate::common::{LanguageManager, PowerManager, SessionManager};

pub struct LightDMGreeter {
    greeter: lightdm::Greeter,
    user_list: Option<lightdm::UserList>,
    shared_data_directory: String,
}

impl LightDMGreeter {
    pub fn new(webview: &WebView) -> Self {
        let greeter = lightdm::Greeter::new();
        let user_list = lightdm::UserList::instance();

        greeter.connect_authentication_complete(clone!(
            #[strong]
            webview,
            move |_| signals::authentication_complete(&webview)
        ));
        greeter.connect_autologin_timer_expired(clone!(
            #[strong]
            webview,
            move |_| signals::autologin_timer_expired(&webview)
        ));
        greeter.connect_show_prompt(clone!(
            #[strong]
            webview,
            move |_, text, ty| signals::show_prompt(&webview, text, ty)
        ));
        greeter.connect_show_message(clone!(
            #[strong]
            webview,
            move |_, text, ty| signals::show_message(&webview, text, ty)
        ));

        if let Err(e) = greeter.connect_to_daemon_sync() {
            logger::error!("{}", e.message());
        }

        let shared_data_directory = match &user_list {
            Some(userlist) => match userlist.users().first() {
                Some(user) => {
                    match greeter.ensure_shared_data_dir_sync(
                        user.name().expect("Failed to get username").as_str(),
                    ) {
                        Ok(data_dir) => {
                            let s = data_dir.to_string();
                            let (substr, _) = s
                                .rsplit_once("/")
                                .unwrap_or_else(|| panic!("{} does not contain `/`", s));
                            substr.to_string()
                        }
                        Err(_) => "".to_string(),
                    }
                }
                None => "".to_string(),
            },
            None => "".to_string(),
        };

        logger::debug!("LightDM API connected");
        Self {
            greeter,
            user_list,
            shared_data_directory,
        }
    }

    pub fn shared_data_directory(&self) -> &str {
        &self.shared_data_directory
    }

    pub fn handle(&self, method: &str, json_args: &str) -> Variant {
        let val: serde_json::Value = serde_json::from_str(json_args).unwrap();
        let args = val.as_array().expect("json_args should be array");
        let json_result = if args.is_empty() {
            match method {
                "authentication_user" => self.authentication_user(),
                "autologin_guest" => self.autologin_guest(),
                "autologin_timeout" => self.autologin_timeout(),
                "autologin_user" => self.autologin_user(),
                "can_hibernate" => self.can_hibernate(),
                "can_restart" => self.can_reboot(),
                "can_shutdown" => self.can_shutdown(),
                "can_suspend" => self.can_suspend(),
                "default_session" => self.default_session(),
                "has_guest_account" => self.has_guest_account(),
                "hide_users_hint" => self.hide_users_hint(),
                "hostname" => self.hostname(),
                "in_authentication" => self.in_authentication(),
                "is_authenticated" => self.is_authenticated(),
                "language" => self.language(),
                "languages" => self.languages(),
                "layout" => self.layout(),
                "layouts" => self.layouts(),
                "lock_hint" => self.lock_hint(),
                "remote_sessions" => self.remote_sessions(),
                "select_guest_hint" => self.select_guest_hint(),
                "select_user_hint" => self.select_user_hint(),
                "sessions" => self.sessions(),
                "shared_data_directory" => self.shared_data_directory_getter(),
                "show_manual_login_hint" => self.show_manual_login_hint(),
                "show_remote_login_hint" => self.show_remote_login_hint(),
                "users" => self.users(),
                "authenticate_as_guest" => self.authenticate_as_guest(),
                "cancel_authentication" => self.cancel_authentication(),
                "cancel_autologin" => self.cancel_autologin(),
                "hibernate" => self.hibernate(),
                "restart" => self.reboot(),
                "shutdown" => self.shutdown(),
                "suspend" => self.suspend(),
                s => {
                    logger::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        } else {
            match method {
                "layout" => self.set_layout(args[0].as_str().unwrap()),
                "authenticate" => self.authenticate(args[0].as_str()),
                "respond" => self.respond(args[0].as_str().unwrap()),
                "set_language" => self.set_language(args[0].as_str().unwrap()),
                "start_session" => self.start_session(args[0].as_str().unwrap()),
                s => {
                    logger::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        };
        json_result.to_variant()
    }

    fn can_hibernate(&self) -> String {
        PowerManager::can_hibernate().to_string()
    }

    fn can_reboot(&self) -> String {
        PowerManager::can_reboot().to_string()
    }

    fn can_shutdown(&self) -> String {
        PowerManager::can_power_off().to_string()
    }

    fn can_suspend(&self) -> String {
        PowerManager::can_suspend().to_string()
    }

    fn hibernate(&self) -> String {
        PowerManager::hibernate()
            .inspect_err(|e| logger::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn reboot(&self) -> String {
        PowerManager::reboot()
            .inspect_err(|e| logger::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn shutdown(&self) -> String {
        PowerManager::power_off()
            .inspect_err(|e| logger::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn suspend(&self) -> String {
        PowerManager::suspend()
            .inspect_err(|e| logger::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn languages(&self) -> String {
        serde_json::to_string(LanguageManager::languages()).unwrap()
    }

    fn language(&self) -> String {
        match LanguageManager::current() {
            Some(language) => serde_json::to_string(&language).unwrap(),
            None => match LanguageManager::languages().first() {
                Some(language) => serde_json::to_string(&language).unwrap(),
                None => "null".to_string(),
            },
        }
    }

    fn set_language(&self, language: &str) -> String {
        if let Err(e) = self.greeter.set_language(language) {
            logger::error!("{}", e.message());
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn authentication_user(&self) -> String {
        if let Some(user) = self.greeter.authentication_user() {
            user.to_string()
        } else {
            "null".to_string()
        }
    }

    fn autologin_guest(&self) -> String {
        self.greeter.is_autologin_guest_hint().to_string()
    }

    fn autologin_timeout(&self) -> String {
        self.greeter.autologin_timeout_hint().to_string()
    }

    fn autologin_user(&self) -> String {
        if let Some(value) = self.greeter.autologin_user_hint() {
            value.to_string()
        } else {
            "null".to_string()
        }
    }

    fn default_session(&self) -> String {
        if let Some(session) = self.greeter.default_session_hint() {
            session.to_string()
        } else {
            "null".to_string()
        }
    }

    fn has_guest_account(&self) -> String {
        self.greeter.has_guest_account_hint().to_string()
    }

    fn hide_users_hint(&self) -> String {
        self.greeter.hides_users_hint().to_string()
    }

    fn hostname(&self) -> String {
        if let Some(value) = lightdm::functions::hostname() {
            value.to_string()
        } else {
            "null".to_string()
        }
    }

    fn in_authentication(&self) -> String {
        self.greeter.is_in_authentication().to_string()
    }

    fn is_authenticated(&self) -> String {
        self.greeter.is_authenticated().to_string()
    }

    fn layout(&self) -> String {
        match lightdm::functions::layout() {
            Some(layout) => lightdm_layout_to_json(&layout),
            None => match lightdm::functions::layouts().first() {
                Some(layout) => lightdm_layout_to_json(layout),
                None => "null".to_string(),
            },
        }
    }

    fn set_layout(&self, name: &str) -> String {
        let layout = lightdm::functions::layouts()
            .into_iter()
            .find(|l| Some(name) == l.name().as_deref());
        lightdm::functions::set_layout(&layout.unwrap());
        true.to_string()
    }

    fn layouts(&self) -> String {
        [
            "[",
            &lightdm::functions::layouts()
                .iter()
                .map(lightdm_layout_to_json)
                .collect::<Vec<String>>()
                .join(","),
            "]",
        ]
        .join("")
    }

    fn lock_hint(&self) -> String {
        self.greeter.is_lock_hint().to_string()
    }

    fn remote_sessions(&self) -> String {
        [
            "[",
            &lightdm::functions::remote_sessions()
                .iter()
                .map(lightdm_session_to_json)
                .collect::<Vec<String>>()
                .join(","),
            "]",
        ]
        .join("")
    }

    fn select_guest_hint(&self) -> String {
        self.greeter.selects_guest_hint().to_string()
    }

    fn select_user_hint(&self) -> String {
        match self.greeter.select_user_hint() {
            Some(value) => value.to_string(),
            None => "null".to_string(),
        }
    }

    fn sessions(&self) -> String {
        serde_json::to_string(&SessionManager::sessions()).unwrap()
    }

    fn shared_data_directory_getter(&self) -> String {
        let dir = &self.shared_data_directory;
        if dir.is_empty() {
            "null".to_string()
        } else {
            dir.clone()
        }
    }

    fn show_manual_login_hint(&self) -> String {
        self.greeter.shows_manual_login_hint().to_string()
    }

    fn show_remote_login_hint(&self) -> String {
        self.greeter.shows_remote_login_hint().to_string()
    }

    fn users(&self) -> String {
        match &self.user_list {
            Some(userlist) => [
                "[",
                &userlist
                    .users()
                    .iter()
                    .map(lightdm_user_to_json)
                    .collect::<Vec<String>>()
                    .join(","),
                "]",
            ]
            .join(""),
            None => "[]".to_string(),
        }
    }

    fn authenticate(&self, username: Option<&str>) -> String {
        if let Err(e) = self.greeter.authenticate(username) {
            logger::error!("{}", e.message());
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn authenticate_as_guest(&self) -> String {
        if let Err(e) = self.greeter.authenticate_as_guest() {
            logger::error!("{}", e.message());
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn cancel_authentication(&self) -> String {
        if let Err(e) = self.greeter.cancel_authentication() {
            logger::error!("{}", e.message());
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn cancel_autologin(&self) -> String {
        self.greeter.cancel_autologin();
        true.to_string()
    }

    fn respond(&self, response: &str) -> String {
        if let Err(e) = self.greeter.respond(response) {
            logger::error!("{}", e.message());
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn start_session(&self, session: &str) -> String {
        if let Err(e) = self.greeter.start_session_sync(Some(session)) {
            logger::error!("{}", e.message());
            false.to_string()
        } else {
            true.to_string()
        }
    }
}

fn lightdm_user_to_json(user: &lightdm::User) -> String {
    let background = user.background().map(|s| s.to_string());
    let display_name = user.display_name().map(|s| s.to_string());
    let home_directory = user.home_directory().map(|s| s.to_string());
    let image = user.image().map(|s| s.to_string());
    let language = user.language().map(|s| s.to_string());
    let layout = user.layout().map(|s| s.to_string());
    let layouts: Vec<String> = user.layouts().iter().map(|s| s.to_string()).collect();
    let logged_in = user.is_logged_in();
    let session = user.session().map(|s| s.to_string());
    let username = user.name().map(|s| s.to_string());
    serde_json::json!({
        "background": background,
        "display_name": display_name,
        "home_directory": home_directory,
        "image": image,
        "language": language,
        "layout": layout,
        "layouts": layouts,
        "logged_in": logged_in,
        "session": session,
        "username": username,
    })
    .to_string()
}

fn lightdm_layout_to_json(layout: &lightdm::Layout) -> String {
    let name = layout.name().map(|s| s.to_string());
    let description = layout.description().map(|s| s.to_string());
    let short_description = layout.short_description().map(|s| s.to_string());
    serde_json::json!({
        "name": name,
        "description": description,
        "short_description": short_description
    })
    .to_string()
}

fn lightdm_session_to_json(session: &lightdm::Session) -> String {
    let comment = session.comment().map(|s| s.to_string());
    let key = session.key().map(|s| s.to_string());
    let name = session.name().map(|s| s.to_string());
    let session_type = session.session_type().map(|s| s.to_string());
    serde_json::json!({
        "comment": comment,
        "key": key,
        "name": name,
        "type": session_type
    })
    .to_string()
}

mod signals {
    use webkit::{
        UserMessage, WebView, gio::Cancellable, glib::variant::ToVariant, prelude::WebViewExt,
    };

    pub(super) fn authentication_complete(webview: &WebView) {
        let parameters = ["authentication_complete", "[]"].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn autologin_timer_expired(webview: &WebView) {
        let parameters = ["autologin_timer_expired", "[]"].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn show_prompt(webview: &WebView, text: &str, ty: lightdm::PromptType) {
        let type_ = match ty {
            lightdm::PromptType::Question => "Visible",
            lightdm::PromptType::Secret => "Secret",
            _ => "Unknown",
        };
        let parameters = ["show_prompt", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn show_message(webview: &WebView, text: &str, ty: lightdm::MessageType) {
        let type_ = match ty {
            lightdm::MessageType::Info => "Info",
            lightdm::MessageType::Error => "Error",
            _ => "Unknown",
        };
        let parameters = ["show_message", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }
}
