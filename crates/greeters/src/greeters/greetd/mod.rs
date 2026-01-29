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
    common::{LanguageManager, PowerManager, SessionManager, UserManager},
    jscvalue::ToJSCValue,
};

pub struct GreetdGreeter {
    pub(super) context: jsc::Context,
    greeter: RefCell<GreetdClient>,
}

impl GreetdGreeter {
    pub fn new(context: jsc::Context, webview: &WebView) -> Self {
        let mut greeter = GreetdClient::new();

        greeter.connect_authentication_complete(clone!(
            #[strong]
            webview,
            move |_| signals::authentication_complete(&webview)
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
                "set_language" => self.set_language(&params[0].to_string()),
                "authenticate" => self.authenticate(params[0].to_string()),
                "respond" => self.respond(params[0].to_string()),
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

    pub(super) fn can_hibernate(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_hibernate())
    }

    pub(super) fn can_reboot(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_reboot())
    }

    pub(super) fn can_shutdown(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_power_off())
    }

    pub(super) fn can_suspend(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, PowerManager::can_suspend())
    }

    pub(super) fn hibernate(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::hibernate()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    pub(super) fn reboot(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::reboot()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    pub(super) fn shutdown(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::power_off()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    pub(super) fn suspend(&self) -> jsc::Value {
        jsc::Value::new_boolean(
            &self.context,
            PowerManager::suspend()
                .inspect_err(|e| logger::error!("{e}"))
                .is_ok(),
        )
    }

    pub(super) fn authentication_user(&self) -> jsc::Value {
        jsc::Value::new_string(&self.context, self.greeter.borrow().authentication_user())
    }

    pub(super) fn in_authentication(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, self.greeter.borrow().in_authentication())
    }

    pub(super) fn is_authenticated(&self) -> jsc::Value {
        jsc::Value::new_boolean(&self.context, self.greeter.borrow().is_authenticated())
    }

    pub(super) fn language(&self) -> jsc::Value {
        let context = &self.context;
        match LanguageManager::current() {
            Some(language) => language.to_jscvalue(context),
            None => match LanguageManager::languages().first() {
                Some(language) => language.to_jscvalue(context),
                None => jsc::Value::new_undefined(context),
            },
        }
    }

    pub(super) fn set_language(&self, language: &str) -> jsc::Value {
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

    pub(super) fn languages(&self) -> jsc::Value {
        let context = &self.context;
        let languages: Vec<_> = LanguageManager::languages()
            .iter()
            .map(|language| language.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &languages)
    }

    pub(super) fn sessions(&self) -> jsc::Value {
        let context = &self.context;
        let sessions: Vec<_> = SessionManager::sessions()
            .iter()
            .map(|session| session.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &sessions)
    }

    pub(super) fn users(&self) -> jsc::Value {
        let context = &self.context;
        let users: Vec<_> = UserManager::instance()
            .list_users()
            .iter()
            .map(|user| user.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &users)
    }

    pub(super) fn authenticate(&self, username: String) -> jsc::Value {
        let context = &self.context;
        match self.greeter.borrow_mut().create_session(username) {
            Ok(Response::Success) | Ok(Response::AuthMessage { .. }) => {
                return jsc::Value::new_boolean(context, true);
            }
            Ok(Response::Error { description, .. }) => logger::error!("{description}"),
            Err(e) => logger::error!("{e}"),
        }
        jsc::Value::new_boolean(context, false)
    }

    pub(super) fn cancel_authentication(&self) -> jsc::Value {
        let context = &self.context;
        match self.greeter.borrow_mut().cancel_session() {
            Ok(Response::Success) => {
                return jsc::Value::new_boolean(context, true);
            }
            Ok(Response::Error { description, .. }) => logger::error!("{description}"),
            Err(e) => logger::error!("{e}"),
            _ => {}
        }
        jsc::Value::new_boolean(context, false)
    }

    pub(super) fn respond(&self, response: String) -> jsc::Value {
        let context = &self.context;
        match self.greeter.borrow_mut().post_response(Some(response)) {
            Ok(Response::Success) => {
                return jsc::Value::new_boolean(context, true);
            }
            Ok(Response::Error { description, .. }) => logger::error!("{description}"),
            Err(e) => logger::error!("{e}"),
            _ => {}
        }

        jsc::Value::new_boolean(context, false)
    }

    pub(super) fn start_session(&self, session_key: String) -> jsc::Value {
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
        logger::info!("{cmd:?}-{env:?}");
        match self.greeter.borrow_mut().start_session(cmd, env) {
            Ok(Response::Success) => std::process::exit(0),
            Ok(Response::Error { description, .. }) => logger::error!("{description}"),
            Err(e) => logger::error!("{e}"),
            _ => {}
        }
        jsc::Value::new_boolean(&self.context, false)
    }
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
}
