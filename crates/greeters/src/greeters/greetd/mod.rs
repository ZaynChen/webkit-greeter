// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use webkit::{WebView, glib::clone};

use std::cell::RefCell;

use crate::common::SessionManager;

use super::signals;

mod client;
use client::GreetdClient;

pub struct GreetdGreeter {
    greeter: RefCell<GreetdClient>,
}

impl GreetdGreeter {
    pub fn new(webview: &WebView) -> Self {
        let mut greeter = GreetdClient::new();

        greeter.connect_authentication_complete(clone!(
            #[strong]
            webview,
            move || signals::authentication_complete(&webview)
        ));
        greeter.connect_show_prompt(clone!(
            #[strong]
            webview,
            move |msg, type_| signals::show_prompt(&webview, msg, type_)
        ));
        greeter.connect_show_message(clone!(
            #[strong]
            webview,
            move |msg, type_| signals::show_message(&webview, msg, type_)
        ));

        if let Err(e) = greeter.connect_to_daemon() {
            log::error!("{e}");
        }

        Self {
            greeter: RefCell::new(greeter),
        }
    }

    pub(super) fn handle(&self, method: &str, args: &[serde_json::Value]) -> String {
        if args.is_empty() {
            match method {
                "authentication_user" => self.authentication_user(),
                "in_authentication" => self.in_authentication(),
                "is_authenticated" => self.is_authenticated(),
                "cancel_authentication" => self.cancel_session(),
                s => {
                    log::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        } else {
            match method {
                "authenticate" => self.create_session(args[0].as_str()),
                "respond" => self.respond(args[0].as_str()),
                "start_session" => self.start_session(args[0].as_str().as_ref().unwrap()),
                s => {
                    log::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        }
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

    fn create_session(&self, username: Option<&str>) -> String {
        if let Err(e) = self
            .greeter
            .borrow_mut()
            .create_session(username.unwrap().to_string())
        {
            log::error!("{e}");
            return false.to_string();
        }
        true.to_string()
    }

    fn cancel_session(&self) -> String {
        if let Err(e) = self.greeter.borrow_mut().cancel_session() {
            log::error!("{e}");
            return false.to_string();
        }
        true.to_string()
    }

    fn respond(&self, response: Option<&str>) -> String {
        if let Err(e) = self
            .greeter
            .borrow_mut()
            .post_response(response.map(|s| s.to_string()))
        {
            log::error!("{e}");
            return false.to_string();
        }
        true.to_string()
    }

    fn start_session(&self, session_key: &str) -> String {
        let session = SessionManager::session(session_key);
        if session.is_none() {
            log::error!("{session_key} does not exist");
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
                log::error!("{e}");
                false.to_string()
            }
        }
    }
}
