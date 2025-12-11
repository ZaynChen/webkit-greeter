// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use lightdm::prelude::*;
use webkit::{
    WebView,
    glib::{self, Variant, clone, variant::ToVariant},
};

use std::rc::Rc;

pub struct Greeter {
    context: jsc::Context,
    greeter: lightdm::Greeter,
    user_list: Option<lightdm::UserList>,
    shared_data_directory: String,
}

impl Greeter {
    pub fn new(context: jsc::Context, webviews: Vec<WebView>) -> Self {
        let greeter = lightdm::Greeter::new();
        let user_list = lightdm::UserList::instance();

        let webviews = Rc::new(webviews);
        greeter.connect_authentication_complete(clone!(
            #[weak]
            webviews,
            move |_| signals::authentication_complete(&webviews)
        ));
        greeter.connect_autologin_timer_expired(clone!(
            #[weak]
            webviews,
            move |_| signals::autologin_timer_expired(&webviews)
        ));
        greeter.connect_show_prompt(clone!(
            #[weak]
            context,
            #[weak]
            webviews,
            move |_, text, ty| signals::show_prompt(&webviews, &context, text, ty)
        ));
        greeter.connect_show_message(clone!(
            #[weak]
            context,
            #[weak]
            webviews,
            move |_, text, ty| signals::show_message(&webviews, &context, text, ty)
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
            context,
            greeter,
            user_list,
            shared_data_directory,
        }
    }

    pub fn shared_data_directory(&self) -> &str {
        &self.shared_data_directory
    }

    pub fn handle(&self, name: &str, json_params: &str) -> Variant {
        let context = &self.context;
        let params = jsc::Value::from_json(context, json_params).to_vec();
        let ret = if params.is_empty() {
            match name {
                "authentication_user" => self.authentication_user(),
                "autologin_guest" => self.autologin_guest(),
                "autologin_timeout" => self.autologin_timeout(),
                "autologin_user" => self.autologin_user(),
                "can_hibernate" => self.can_hibernate(),
                "can_restart" => self.can_restart(),
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
                "restart" => self.restart(),
                "shutdown" => self.shutdown(),
                "suspend" => self.suspend(),
                s => {
                    logger::warn!("{s} does not implemented");
                    jsc::Value::new_undefined(context)
                }
            }
        } else {
            match name {
                "layout" => self.set_layout(params[0].clone()),
                "authenticate" => self.authenticate(Some(&params[0].to_string())),
                "respond" => self.respond(&params[0].to_string()),
                "set_language" => self.set_language(&params[0].to_string()),
                "start_session" => self.start_session(Some(&params[0].to_string())),
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

    fn authentication_user(&self) -> jsc::Value {
        let context = &self.context;
        if let Some(user) = self.greeter.authentication_user() {
            jsc::Value::new_string(context, Some(user.as_str()))
        } else {
            jsc::Value::new_null(context)
        }
    }

    fn autologin_guest(&self) -> jsc::Value {
        let value = self.greeter.is_autologin_guest_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn autologin_timeout(&self) -> jsc::Value {
        let value = self.greeter.autologin_timeout_hint();
        jsc::Value::new_number(&self.context, value as f64)
    }

    fn autologin_user(&self) -> jsc::Value {
        let context = &self.context;
        if let Some(value) = self.greeter.autologin_user_hint() {
            jsc::Value::new_string(context, Some(value.as_str()))
        } else {
            jsc::Value::new_null(context)
        }
    }

    fn can_hibernate(&self) -> jsc::Value {
        let value = lightdm::functions::can_hibernate();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn can_restart(&self) -> jsc::Value {
        let value = lightdm::functions::can_restart();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn can_shutdown(&self) -> jsc::Value {
        let value = lightdm::functions::can_shutdown();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn can_suspend(&self) -> jsc::Value {
        let value = lightdm::functions::can_suspend();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn default_session(&self) -> jsc::Value {
        if let Some(session) = self.greeter.default_session_hint() {
            jsc::Value::new_string(&self.context, Some(session.as_str()))
        } else {
            jsc::Value::new_null(&self.context)
        }
    }

    fn has_guest_account(&self) -> jsc::Value {
        let value = self.greeter.has_guest_account_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn hide_users_hint(&self) -> jsc::Value {
        let value = self.greeter.hides_users_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn hostname(&self) -> jsc::Value {
        let context = &self.context;
        if let Some(value) = lightdm::functions::hostname() {
            jsc::Value::new_string(context, Some(value.as_str()))
        } else {
            jsc::Value::new_null(context)
        }
    }

    fn in_authentication(&self) -> jsc::Value {
        let value = self.greeter.is_in_authentication();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn is_authenticated(&self) -> jsc::Value {
        let value = self.greeter.is_authenticated();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn language(&self) -> jsc::Value {
        let context = &self.context;
        match lightdm::functions::language() {
            Some(language) => language.to_jscvalue(context),
            None => match lightdm::functions::languages().first() {
                Some(language) => language.to_jscvalue(context),
                None => jsc::Value::new_undefined(context),
            },
        }
    }

    fn languages(&self) -> jsc::Value {
        let context = &self.context;
        let languages: Vec<jsc::Value> = lightdm::functions::languages()
            .iter()
            .map(|language| language.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &languages)
    }

    fn layout(&self) -> jsc::Value {
        let context = &self.context;
        match lightdm::functions::layout() {
            Some(layout) => layout.to_jscvalue(context),
            None => match lightdm::functions::layouts().first() {
                Some(layout) => layout.to_jscvalue(context),
                None => jsc::Value::new_undefined(context),
            },
        }
    }

    fn set_layout(&self, value: jsc::Value) -> jsc::Value {
        let context = &self.context;
        if !value.object_has_property("name")
            || !value.object_has_property("description")
            || !value.object_has_property("short_description")
        {
            context.throw("Invalid LightDMLayout");
        }

        let name = value.object_get_property("name").and_then(|s| {
            if s.is_string() {
                Some(s.to_string())
            } else {
                None
            }
        });

        let layout = lightdm::functions::layouts()
            .into_iter()
            .find(|l| name == l.name().map(|s| s.to_string()));
        lightdm::functions::set_layout(&layout.unwrap());
        jsc::Value::new_boolean(context, true)
    }

    fn layouts(&self) -> jsc::Value {
        let context = &self.context;
        let layouts: Vec<jsc::Value> = lightdm::functions::layouts()
            .iter()
            .map(|layout| layout.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &layouts)
    }

    fn lock_hint(&self) -> jsc::Value {
        let value = self.greeter.is_lock_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn remote_sessions(&self) -> jsc::Value {
        let context = &self.context;
        let sessions: Vec<jsc::Value> = lightdm::functions::remote_sessions()
            .iter()
            .map(|session| session.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &sessions)
    }

    fn select_guest_hint(&self) -> jsc::Value {
        let value = self.greeter.selects_guest_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn select_user_hint(&self) -> jsc::Value {
        let context = &self.context;
        match self.greeter.select_user_hint() {
            Some(value) => jsc::Value::new_string(context, Some(value.as_str())),
            None => jsc::Value::new_null(context),
        }
    }

    fn sessions(&self) -> jsc::Value {
        let context = &self.context;
        let sessions: Vec<jsc::Value> = lightdm::functions::sessions()
            .iter()
            .map(|session| session.to_jscvalue(context))
            .collect();
        jsc::Value::new_array_from_garray(context, &sessions)
    }

    fn shared_data_directory_getter(&self) -> jsc::Value {
        let context = &self.context;
        let dir = &self.shared_data_directory;
        if dir.is_empty() {
            jsc::Value::new_null(context)
        } else {
            jsc::Value::new_string(context, Some(dir))
        }
    }

    fn show_manual_login_hint(&self) -> jsc::Value {
        let value = self.greeter.shows_manual_login_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn show_remote_login_hint(&self) -> jsc::Value {
        let value = self.greeter.shows_remote_login_hint();
        jsc::Value::new_boolean(&self.context, value)
    }

    fn users(&self) -> jsc::Value {
        let context = &self.context;
        let users = match &self.user_list {
            Some(userlist) => userlist
                .users()
                .iter()
                .map(|user| user.to_jscvalue(context))
                .collect::<Vec<jsc::Value>>(),
            None => vec![],
        };
        jsc::Value::new_array_from_garray(context, &users)
    }

    fn authenticate(&self, username: Option<&str>) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.authenticate(username) {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn authenticate_as_guest(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.authenticate_as_guest() {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn cancel_authentication(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.cancel_authentication() {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn cancel_autologin(&self) -> jsc::Value {
        self.greeter.cancel_autologin();
        jsc::Value::new_boolean(&self.context, true)
    }

    fn hibernate(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::hibernate() {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn respond(&self, response: &str) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.respond(response) {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn restart(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::restart() {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn set_language(&self, language: &str) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.set_language(language) {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn shutdown(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::shutdown() {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn suspend(&self) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = lightdm::functions::suspend() {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }

    fn start_session(&self, session: Option<&str>) -> jsc::Value {
        let context = &self.context;
        if let Err(e) = self.greeter.start_session_sync(session) {
            logger::error!("{}", e.message());
            jsc::Value::new_boolean(context, false)
        } else {
            jsc::Value::new_boolean(context, true)
        }
    }
}

mod signals {
    use webkit::{
        UserMessage, WebView,
        gio::Cancellable,
        glib::{translate::IntoGlib, variant::ToVariant},
        prelude::WebViewExt,
    };

    pub(super) fn authentication_complete(webviews: &[WebView]) {
        webviews.iter().for_each(|webview| {
            let parameters = ["authentication_complete", "[]"].to_variant();
            let message = UserMessage::new("greeter", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }

    pub(super) fn autologin_timer_expired(webviews: &[WebView]) {
        webviews.iter().for_each(|webview| {
            let parameters = ["autologin_timer_expired", "[]"].to_variant();
            let message = UserMessage::new("greeter", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }

    pub(super) fn show_prompt(
        webviews: &[WebView],
        context: &jsc::Context,
        text: &str,
        ty: lightdm::PromptType,
    ) {
        webviews.iter().for_each(|webview| {
            let param = jsc::Value::new_array_from_garray(
                context,
                &[
                    jsc::Value::new_string(context, Some(text)),
                    jsc::Value::new_number(context, ty.into_glib() as f64),
                ],
            )
            .to_json(0)
            .expect("param parse to json failed");
            let parameters = ["show_prompt", &param].to_variant();
            let message = UserMessage::new("greeter", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }

    pub(super) fn show_message(
        webviews: &[WebView],
        context: &jsc::Context,
        text: &str,
        ty: lightdm::MessageType,
    ) {
        webviews.iter().for_each(|webview| {
            let param = jsc::Value::new_array_from_garray(
                context,
                &[
                    jsc::Value::new_string(context, Some(text)),
                    jsc::Value::new_number(context, ty.into_glib() as f64),
                ],
            )
            .to_json(0)
            .expect("param parse to json failed");

            let parameters = ["show_message", &param].to_variant();
            let message = UserMessage::new("greeter", Some(&parameters));
            webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
        });
    }
}

use jsc::JSCValueExtManual;

trait ToJSCValue {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value;
}

impl ToJSCValue for lightdm::User {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let background = self.background();
        let display_name = self.display_name();
        let home_directory = self.home_directory();
        let image = self.image();
        let language = self.language();
        let layout = self.layout();
        let layouts: Vec<jsc::Value> = self
            .layouts()
            .iter()
            .map(|l| jsc::Value::new_string(context, Some(l.as_str())))
            .collect();

        let logged_in = self.is_logged_in();
        let session = self.session();
        let username = self.name();

        value.object_set_property(
            "background",
            &jsc::Value::new_string(context, background.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "display_name",
            &jsc::Value::new_string(context, display_name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "home_directory",
            &jsc::Value::new_string(context, home_directory.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "image",
            &jsc::Value::new_string(context, image.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "language",
            &jsc::Value::new_string(context, language.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "layout",
            &jsc::Value::new_string(context, layout.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "layouts",
            &jsc::Value::new_array_from_garray(context, &layouts),
        );
        value.object_set_property("logged_in", &jsc::Value::new_boolean(context, logged_in));
        value.object_set_property(
            "session",
            &jsc::Value::new_string(context, session.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "username",
            &jsc::Value::new_string(context, username.as_ref().map(|s| s.as_str())),
        );

        value
    }
}

impl ToJSCValue for lightdm::Session {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let comment = self.comment();
        let key = self.key();
        let name = self.name();
        let session_type = self.session_type();

        value.object_set_property(
            "comment",
            &jsc::Value::new_string(context, comment.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "key",
            &jsc::Value::new_string(context, key.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "name",
            &jsc::Value::new_string(context, name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "type",
            &jsc::Value::new_string(context, session_type.as_ref().map(|s| s.as_str())),
        );

        value
    }
}

impl ToJSCValue for lightdm::Language {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let code = self.code();
        let name = self.name();
        let territory = self.territory();

        value.object_set_property(
            "code",
            &jsc::Value::new_string(context, code.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "name",
            &jsc::Value::new_string(context, name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "territory",
            &jsc::Value::new_string(context, territory.as_ref().map(|s| s.as_str())),
        );

        value
    }
}

impl ToJSCValue for lightdm::Layout {
    fn to_jscvalue(&self, context: &jsc::Context) -> jsc::Value {
        let value = jsc::Value::new_object(context, None, None);

        let name = self.name();
        let description = self.description();
        let short_description = self.short_description();

        value.object_set_property(
            "name",
            &jsc::Value::new_string(context, name.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "description",
            &jsc::Value::new_string(context, description.as_ref().map(|s| s.as_str())),
        );
        value.object_set_property(
            "short_description",
            &jsc::Value::new_string(context, short_description.as_ref().map(|s| s.as_str())),
        );

        value
    }
}
