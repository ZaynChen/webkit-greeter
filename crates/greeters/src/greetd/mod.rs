// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

mod accounts;
mod client;
mod dbus;
mod language;
mod power;
mod session;

use greetd_ipc::Response;
use jsc::JSCValueExtManual;
use webkit::{
    WebView,
    glib::{Variant, clone, variant::ToVariant},
};

use std::cell::RefCell;

use accounts::{User, UserManager};
use client::GreetdClient;
use language::{Language, LanguageManager};
use power::PowerManager;
use session::{Session, SessionManager};

pub struct Greeter {
    context: jsc::Context,
    greeter: RefCell<GreetdClient>,
}

impl Greeter {
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
                // "autologin_guest" => self.autologin_guest(),
                // "autologin_timeout" => self.autologin_timeout(),
                // "autologin_user" => self.autologin_user(),
                "can_hibernate" => self.can_hibernate(),
                "can_restart" => self.can_reboot(),
                "can_shutdown" => self.can_shutdown(),
                "can_suspend" => self.can_suspend(),
                // "default_session" => self.default_session(),
                // "has_guest_account" => self.has_guest_account(),
                // "hide_users_hint" => self.hide_users_hint(),
                // "hostname" => self.hostname(),
                "authentication_user" => self.authentication_user(),
                "in_authentication" => self.in_authentication(),
                "is_authenticated" => self.is_authenticated(),
                "language" => self.language(),
                "languages" => self.languages(),
                // "layout" => self.layout(),
                // "layouts" => self.layouts(),
                // "lock_hint" => self.lock_hint(),
                // "remote_sessions" => self.remote_sessions(),
                // "select_guest_hint" => self.select_guest_hint(),
                // "select_user_hint" => self.select_user_hint(),
                "sessions" => self.sessions(),
                // "show_manual_login_hint" => self.show_manual_login_hint(),
                // "show_remote_login_hint" => self.show_remote_login_hint(),
                "users" => self.users(),
                // "authenticate_as_guest" => self.authenticate_as_guest(),
                // "cancel_autologin" => self.cancel_autologin(),
                "hibernate" => self.hibernate(),
                "restart" => self.reboot(),
                "shutdown" => self.shutdown(),
                "suspend" => self.suspend(),
                "cancel_authentication" => self.cancel_authentication(),
                s => {
                    logger::warn!("{s} does not implemented");
                    jsc::Value::new_undefined(context)
                }
            }
        } else {
            match name {
                // "layout" => self.set_layout(params[0].clone()),
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
            Some(language) => language_to_jsc_value(context, &language),
            None => match LanguageManager::languages().first() {
                Some(language) => language_to_jsc_value(context, language),
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
            .map(|language| language_to_jsc_value(context, language))
            .collect();
        jsc::Value::new_array_from_garray(context, &languages)
    }

    fn sessions(&self) -> jsc::Value {
        let context = &self.context;
        let sessions: Vec<_> = SessionManager::sessions()
            .iter()
            .map(|session| session_to_jsc_value(context, session))
            .collect();
        jsc::Value::new_array_from_garray(context, &sessions)
    }

    fn users(&self) -> jsc::Value {
        let context = &self.context;
        let users: Vec<_> = UserManager::instance()
            .list_users()
            .iter()
            .map(|user| user_to_jsc_value(context, user))
            .collect();
        jsc::Value::new_array_from_garray(context, &users)
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

    fn authenticate(&self, username: String) -> jsc::Value {
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

    fn cancel_authentication(&self) -> jsc::Value {
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

    fn respond(&self, response: String) -> jsc::Value {
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

fn session_to_jsc_value(context: &jsc::Context, session: &Session) -> jsc::Value {
    let value = jsc::Value::new_object(context, None, None);

    let comment = session.comment();
    let key = session.key();
    let name = session.name();
    let session_type = session.type_();

    value.object_set_property("comment", &jsc::Value::new_string(context, Some(comment)));
    value.object_set_property("key", &jsc::Value::new_string(context, Some(key)));
    value.object_set_property("name", &jsc::Value::new_string(context, Some(name)));
    value.object_set_property("type", &jsc::Value::new_string(context, Some(session_type)));

    value
}

fn user_to_jsc_value(context: &jsc::Context, user: &User) -> jsc::Value {
    let value = jsc::Value::new_object(context, None, None);

    let username = user.user_name();
    let real_name = user.real_name().filter(|n| !n.is_empty());
    let display_name = if real_name.is_some() {
        real_name
    } else {
        user.user_name()
    };
    let home_directory = user.home_directory();
    let image = user.icon_file();
    let language = user.language();
    let logged_in = user
        .uid()
        .map(|uid| SessionManager::is_logged_in(uid as u32))
        .unwrap_or_default();
    let session = user.session();

    value.object_set_property(
        "display_name",
        &jsc::Value::new_string(context, display_name),
    );
    value.object_set_property(
        "home_directory",
        &jsc::Value::new_string(context, home_directory),
    );
    value.object_set_property("image", &jsc::Value::new_string(context, image));
    value.object_set_property("language", &jsc::Value::new_string(context, language));
    value.object_set_property("logged_in", &jsc::Value::new_boolean(context, logged_in));
    value.object_set_property("session", &jsc::Value::new_string(context, session));
    value.object_set_property("username", &jsc::Value::new_string(context, username));

    value
}

fn language_to_jsc_value(context: &jsc::Context, language: &Language) -> jsc::Value {
    let value = jsc::Value::new_object(context, None, None);

    let code = language.code();
    let name = language.name();
    let territory = language.territory();

    value.object_set_property("code", &jsc::Value::new_string(context, Some(code)));
    value.object_set_property("name", &jsc::Value::new_string(context, Some(name)));
    value.object_set_property(
        "territory",
        &jsc::Value::new_string(context, Some(territory)),
    );

    value
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
