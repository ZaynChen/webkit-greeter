// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

mod client;
use client::GreetdClient;

use greetd_ipc::Response;
use jsc::JSCValueExtManual;
use webkit::{
    WebView,
    glib::{Variant, clone, variant::ToVariant},
};

use std::cell::RefCell;

use crate::{
    common::{LanguageManager, LayoutManager, PowerManager, SessionManager, UserManager},
    jscvalue::ToJSCValue,
};

pub struct GreetdGreeter {
    pub(super) context: jsc::Context,
    greeter: RefCell<GreetdClient>,
}

impl GreetdGreeter {
    pub fn new(context: jsc::Context, webview: &WebView) -> Self {
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
            context,
            greeter: RefCell::new(greeter),
        }
    }

    pub fn handle(&self, name: &str, json_params: &str) -> Variant {
        let context = &self.context;
        let params = jsc::Value::from_json(context, json_params).to_vec();
        let ret = if params.is_empty() {
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
                    jsc::Value::new_undefined(context)
                }
            }
        } else {
            match name {
                "layout" => self.set_layout(&params[0].to_string()),
                "set_language" => self.set_language(&params[0].to_string()),
                "authenticate" => self.authenticate(params[0].to_string()),
                "respond" => {
                    let param = &params[0];
                    let response = if param.is_string() {
                        Some(param.to_string())
                    } else {
                        None
                    };
                    self.respond(response)
                }
                "start_session" => self.start_session(params[0].to_string()),
                s => {
                    logger::warn!("{s} does not implemented");
                    jsc::Value::new_undefined(context)
                }
            }
        };

        if let Some(json) = ret.to_json(0) {
            json.to_variant()
        } else {
            "undefined".to_variant()
        }
    }

    fn can_hibernate(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_hibernate())
    }

    fn can_reboot(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_reboot())
    }

    fn can_shutdown(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_power_off())
    }

    fn can_suspend(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_suspend())
    }

    fn hibernate(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::hibernate()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    fn reboot(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::reboot()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    fn shutdown(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::power_off()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    fn suspend(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::suspend()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    fn authentication_user(&self) -> jsc::Value {
        jsc::Value::new_string(&self.context, self.greeter.borrow().authentication_user())
    }

    fn in_authentication(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, self.greeter.borrow().in_authentication())
    }

    fn is_authenticated(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, self.greeter.borrow().is_authenticated())
    }

    fn language(&self) -> jsc::Value {
        let context = &self.context;
        match LanguageManager::current() {
            Some(language) => language.to_jscvalue(context),
            None => match LanguageManager::languages().first() {
                Some(language) => language.to_jscvalue(context),
                None => jsc::Value::new_undefined(context),
            },
        }
    }

    fn set_language(&self, language: &str) -> jsc::Value {
        let context = &self.context;
        if let Some(user) = self.greeter.borrow().authentication_user() {
            if let Err(e) = UserManager::set_language(user, language) {
                logger::error!("{e}");
                jsc::Value::new_boolean(context, false)
            } else {
                jsc::Value::new_boolean(context, true)
            }
        } else {
            logger::error!("No user is in authentication");
            jsc::Value::new_boolean(context, false)
        }
    }

    fn languages(&self) -> jsc::Value {
        let context = &self.context;
        let languages: Vec<_> = LanguageManager::languages()
            .iter()
            .map(|language| language.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &languages)
    }

    fn layout(&self) -> jsc::Value {
        LayoutManager::instance()
            .layout()
            .to_jscvalue(&self.context)
    }

    fn set_layout(&self, layout: &str) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, LayoutManager::instance().set_layout(layout))
    }

    fn layouts(&self) -> jsc::Value {
        let context = &self.context;
        let layouts: Vec<_> = LayoutManager::instance()
            .layouts()
            .iter()
            .map(|l| l.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(&self.context, &layouts)
    }

    fn sessions(&self) -> jsc::Value {
        let context = &self.context;
        let sessions: Vec<_> = SessionManager::sessions()
            .iter()
            .map(|session| session.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &sessions)
    }

    fn users(&self) -> jsc::Value {
        let context = &self.context;
        let users: Vec<_> = UserManager::instance()
            .list_users()
            .iter()
            .map(|user| user.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &users)
    }

    fn authenticate(&self, username: String) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.borrow_mut().create_session(username) {
            logger::error!("{e}");
            return jsc::Value::new_boolean(context, false);
        }
        jsc::Value::new_boolean(context, true)
    }

    fn cancel_authentication(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.borrow_mut().cancel_session() {
            logger::error!("{e}");
            return jsc::Value::new_boolean(context, false);
        }
        jsc::Value::new_boolean(context, true)
    }

    fn respond(&self, response: Option<String>) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.borrow_mut().post_response(response) {
            logger::error!("{e}");
            return jsc::Value::new_boolean(context, false);
        }
        jsc::Value::new_boolean(context, true)
    }

    fn start_session(&self, session_key: String) -> jsc::Value {
        let session = SessionManager::session(&session_key);
        if session.is_none() {
            logger::error!("{session_key} does not exist");
            return jsc::Value::new_boolean(&self.context, false);
        }
        let session = session.unwrap();
        let cmd = vec![session.exec().to_string()];
        let env = match session.type_() {
            "wayland" => vec!["XDG_SESSION_TYPE=wayland".to_string()],
            "x" => vec!["XDG_SESSION_TYPE=x11".to_string()],
            _ => vec![],
        };
        let context = &self.context;
        match self.greeter.borrow_mut().start_session(cmd, env) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                logger::error!("{e}");
                jsc::Value::new_boolean(context, false)
            }
        }
    }
}

mod signals {
    use webkit::{
        UserMessage, WebView, gio::Cancellable, glib::variant::ToVariant, prelude::WebViewExt,
    };

    pub(super) fn show_prompt(webview: &WebView, type_: &str, text: &str) {
        let parameters = ["show_prompt", &format!("[\"{type_}\", \"{text}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn show_message(webview: &WebView, type_: &str, text: &str) {
        let parameters = ["show_message", &format!("[\"{type_}\", \"{text}\"]")].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }

    pub(super) fn authentication_complete(webview: &WebView) {
        let parameters = ["authentication_complete", "[]"].to_variant();
        let message = UserMessage::new("greeter", Some(&parameters));
        webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
    }
}
