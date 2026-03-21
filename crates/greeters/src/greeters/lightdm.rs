// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use webkit::{
    WebView,
    glib::{self, clone},
};

use super::signals;

use lightdm_client::Greeter;

pub struct LightDMGreeter {
    greeter: Greeter,
}

impl LightDMGreeter {
    pub fn new(webview: &WebView) -> Self {
        let greeter = Greeter::new();

        greeter.connect_authentication_complete(clone!(
            #[strong]
            webview,
            move |_| signals::authentication_complete(&webview)
        ));
        greeter.connect_show_prompt(clone!(
            #[strong]
            webview,
            move |_, text, ty| signals::show_prompt(&webview, text, ty.try_into().unwrap())
        ));
        greeter.connect_show_message(clone!(
            #[strong]
            webview,
            move |_, text, ty| signals::show_message(&webview, text, ty.try_into().unwrap())
        ));

        // if let Err(e) = greeter.connect_to_daemon_sync() {
        if let Err(e) = glib::MainContext::default().block_on(greeter.connect_to_daemon_future()) {
            log::error!("{e}");
        }

        log::debug!("LightDM API connected");
        Self { greeter }
    }

    pub(super) fn handle(&self, method: &str, args: &[serde_json::Value]) -> String {
        if args.is_empty() {
            match method {
                "authentication_user" => self.authentication_user(),
                "in_authentication" => self.in_authentication(),
                "is_authenticated" => self.is_authenticated(),
                "cancel_authentication" => self.cancel_authentication(),
                // ----
                "autologin_guest" => self.autologin_guest_hint(),
                "autologin_timeout" => self.autologin_timeout_hint(),
                "autologin_user" => self.autologin_user_hint(),
                "default_session" => self.default_session_hint(),
                "has_guest_account" => self.has_guest_account_hint(),
                "hide_users_hint" => self.hide_users_hint(),
                "lock_hint" => self.lock_hint(),
                "select_guest_hint" => self.select_guest_hint(),
                "select_user_hint" => self.select_user_hint(),
                "show_manual_login_hint" => self.show_manual_login_hint(),
                "show_remote_login_hint" => self.show_remote_login_hint(),
                "authenticate_as_guest" => self.authenticate_as_guest(),
                "cancel_autologin" => self.cancel_autologin(),
                s => {
                    log::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        } else {
            match method {
                "authenticate" => self.authenticate(args[0].as_str().map(str::to_string)),
                "respond" => self.respond(args[0].as_str().unwrap().to_string()),
                "start_session" => self.start_session_sync(args[0].as_str().map(str::to_string)),
                s => {
                    log::warn!("{s} does not implemented");
                    "undefined".to_string()
                }
            }
        }
    }

    fn authentication_user(&self) -> String {
        self.greeter
            .authentication_user()
            .as_deref()
            .unwrap_or("null")
            .to_string()
    }

    fn in_authentication(&self) -> String {
        self.greeter.in_authentication().to_string()
    }

    fn is_authenticated(&self) -> String {
        self.greeter.is_authenticated().to_string()
    }

    fn authenticate(&self, username: Option<String>) -> String {
        if let Err(e) = self.greeter.authenticate(username) {
            log::error!("{e}");
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn cancel_authentication(&self) -> String {
        if let Err(e) = self.greeter.cancel_authentication() {
            log::error!("{e}");
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn respond(&self, response: String) -> String {
        if let Err(e) = self.greeter.respond(response) {
            log::error!("{e}");
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn start_session_sync(&self, session: Option<String>) -> String {
        if let Err(e) =
            glib::MainContext::default().block_on(self.greeter.start_session_future(session))
        {
            log::error!("{e}");
            false.to_string()
        } else {
            true.to_string()
        }
        // if let Err(e) = self.greeter.start_session_sync(session) {
        //     log::error!("{e}");
        //     false.to_string()
        // } else {
        //     true.to_string()
        // }
    }

    fn autologin_guest_hint(&self) -> String {
        self.greeter.autologin_guest_hint().to_string()
    }

    fn autologin_timeout_hint(&self) -> String {
        self.greeter.autologin_timeout_hint().to_string()
    }

    fn autologin_user_hint(&self) -> String {
        self.greeter
            .autologin_user_hint()
            .unwrap_or("null".to_string())
    }

    fn default_session_hint(&self) -> String {
        self.greeter
            .default_session_hint()
            .unwrap_or("null".to_string())
    }

    fn has_guest_account_hint(&self) -> String {
        self.greeter.has_guest_account_hint().to_string()
    }

    fn hide_users_hint(&self) -> String {
        self.greeter.hide_users_hint().to_string()
    }

    fn lock_hint(&self) -> String {
        self.greeter.lock_hint().to_string()
    }

    fn select_guest_hint(&self) -> String {
        self.greeter.select_guest_hint().to_string()
    }

    fn select_user_hint(&self) -> String {
        self.greeter
            .select_user_hint()
            .unwrap_or("null".to_string())
    }

    fn show_manual_login_hint(&self) -> String {
        self.greeter.show_manual_login_hint().to_string()
    }

    fn show_remote_login_hint(&self) -> String {
        self.greeter.show_remote_login_hint().to_string()
    }

    fn authenticate_as_guest(&self) -> String {
        if let Err(e) = self.greeter.authenticate_as_guest() {
            log::error!("{e}");
            false.to_string()
        } else {
            true.to_string()
        }
    }

    fn cancel_autologin(&self) -> String {
        self.greeter.cancel_autologin();
        true.to_string()
    }
}
