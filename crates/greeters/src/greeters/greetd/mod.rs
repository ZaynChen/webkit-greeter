// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use webkit::{
    WebView,
    glib::{Variant, clone, variant::ToVariant},
};

use std::cell::RefCell;

use crate::common::{LanguageManager, LayoutManager, PowerManager, SessionManager, UserManager};

mod client;
use client::{GreetdClient, MessageType, PromptType};

pub struct GreetdGreeter {
    greeter: RefCell<GreetdClient>,
}

impl GreetdGreeter {
    pub fn new(webview: &WebView) -> Self {
        let mut greeter = GreetdClient::new();

        greeter.connect_show_prompt(clone!(
            #[strong]
            webview,
            move |type_, msg| signals::show_prompt(&webview, type_, msg)
        ));

        greeter.connect_show_message(clone!(
            #[strong]
            webview,
            move |type_, msg| signals::show_message(&webview, type_, msg)
        ));

        greeter.connect_authentication_complete(clone!(
            #[strong]
            webview,
            move || signals::authentication_complete(&webview)
        ));

        Self {
            greeter: RefCell::new(greeter),
        }
    }

    pub fn handle(&self, name: &str, json_args: &str) -> Variant {
        let val: serde_json::Value = serde_json::from_str(json_args).unwrap();
        let args = val.as_array().expect("json_args should be array");
        let json_result = if args.is_empty() {
            match name {
                "can_hibernate" => self.can_hibernate(),
                "can_restart" => self.can_reboot(),
                "can_shutdown" => self.can_shutdown(),
                "can_suspend" => self.can_suspend(),
                "hibernate" => self.hibernate(),
                "restart" => self.reboot(),
                "shutdown" => self.shutdown(),
                "suspend" => self.suspend(),
                "language" => self.language(),
                "languages" => self.languages(),
                "layout" => self.layout(),
                "layouts" => self.layouts(),
                "sessions" => self.sessions(),
                "users" => self.users(),
                "authentication_user" => self.authentication_user(),
                "in_authentication" => self.in_authentication(),
                "is_authenticated" => self.is_authenticated(),
                "cancel_authentication" => self.cancel_authentication(),
                s => {
                    logger::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        } else {
            match name {
                "layout" => self.set_layout(args[0].as_str().unwrap()),
                "authenticate" => self.authenticate(args[0].as_str().unwrap().to_string()),
                "respond" => self.respond(args[0].as_str().map(|s| s.to_string())),
                "start_session" => self.start_session(args[0].as_str().unwrap().to_string()),
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

    fn authentication_user(&self) -> String {
        self.greeter
            .borrow()
            .authentication_user()
            .unwrap_or("null")
            .to_string()
    }

    fn in_authentication(&self) -> String {
        self.greeter.borrow().in_authentication().to_string()
    }

    fn is_authenticated(&self) -> String {
        self.greeter.borrow().is_authenticated().to_string()
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

    fn languages(&self) -> String {
        serde_json::to_string(LanguageManager::languages()).unwrap()
    }

    fn layout(&self) -> String {
        serde_json::to_string(LayoutManager::instance().layout()).unwrap()
    }

    fn set_layout(&self, layout: &str) -> String {
        LayoutManager::instance().set_layout(layout).to_string()
    }

    fn layouts(&self) -> String {
        serde_json::to_string(LayoutManager::instance().layouts()).unwrap()
    }

    fn sessions(&self) -> String {
        serde_json::to_string(&SessionManager::sessions()).unwrap()
    }

    fn users(&self) -> String {
        serde_json::to_string(UserManager::instance().list_users()).unwrap()
    }

    fn authenticate(&self, username: String) -> String {
        if let Err(e) = self.greeter.borrow_mut().create_session(username) {
            logger::error!("{e}");
            return false.to_string();
        }
        true.to_string()
    }

    fn cancel_authentication(&self) -> String {
        if let Err(e) = self.greeter.borrow_mut().cancel_session() {
            logger::error!("{e}");
            return false.to_string();
        }
        true.to_string()
    }

    fn respond(&self, response: Option<String>) -> String {
        if let Err(e) = self.greeter.borrow_mut().post_response(response) {
            logger::error!("{e}");
            return false.to_string();
        }
        true.to_string()
    }

    fn start_session(&self, session_key: String) -> String {
        let session = SessionManager::session(&session_key);
        if session.is_none() {
            logger::error!("{session_key} does not exist");
            return false.to_string();
        }
        let session = session.unwrap();
        let cmd = vec![session.exec().to_string()];
        let env = match session.type_() {
            "wayland" => vec!["XDG_SESSION_TYPE=wayland".to_string()],
            "x" => vec!["XDG_SESSION_TYPE=x11".to_string()],
            _ => vec![],
        };
        match self.greeter.borrow_mut().start_session(cmd, env) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                logger::error!("{e}");
                false.to_string()
            }
        }
    }
}

mod signals {
    use webkit::{
        UserMessage, WebView, gio::Cancellable, glib::variant::ToVariant, prelude::WebViewExt,
    };

    use super::{MessageType, PromptType};

    pub(super) fn show_prompt(webview: &WebView, text: &str, ty: PromptType) {
        let type_ = match ty {
            PromptType::Visible => "Visible",
            PromptType::Secret => "Secret",
        };
        let parameters = ["show_prompt", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn show_message(webview: &WebView, text: &str, ty: MessageType) {
        let type_ = match ty {
            MessageType::Info => "Info",
            MessageType::Error => "Error",
        };
        let parameters = ["show_message", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn authentication_complete(webview: &WebView) {
        let parameters = ["authentication_complete", "[]"].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }
}
