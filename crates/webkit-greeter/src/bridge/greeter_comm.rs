// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    gio::Cancellable,
    glib::{Variant, variant::ToVariant},
};
use webkit::{UserMessage, WebView, prelude::WebViewExt};

pub(super) struct GreeterComm {
    context: jsc::Context,
    primary: WebView,
    secondaries: Vec<WebView>,
}

impl GreeterComm {
    pub(super) fn new(context: jsc::Context, primary: WebView, secondaries: Vec<WebView>) -> Self {
        Self {
            context,
            primary,
            secondaries,
        }
    }

    pub(super) fn handle(&self, name: &str, json_params: &str) -> Variant {
        let context = &self.context;
        let ret = if "broadcast" == name && json_params != "[]" {
            self.broadcast(json_params)
        } else {
            jsc::Value::new_undefined(context)
        };
        ret.to_json(0).unwrap_or("undefined".into()).to_variant()
    }

    pub(super) fn primary(&self) -> &WebView {
        &self.primary
    }

    pub(super) fn secondaries(&self) -> &[WebView] {
        &self.secondaries
    }

    fn broadcast(&self, json_params: &str) -> jsc::Value {
        let context = &self.context;
        [&self.primary]
            .into_iter()
            .chain(&self.secondaries)
            .for_each(|webview| {
                let parameters = ["_emit", json_params].to_variant();
                let message = UserMessage::new("greeter_comm", Some(&parameters));
                webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
            });
        jsc::Value::new_null(context)
    }
}
