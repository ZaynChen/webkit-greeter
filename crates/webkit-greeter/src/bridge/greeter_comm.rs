// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    gio::Cancellable,
    glib::{Variant, variant::ToVariant},
};
use webkit::{UserMessage, WebView, prelude::WebViewExt};

pub(super) struct GreeterComm {
    primary: WebView,
    secondaries: Vec<WebView>,
}

impl GreeterComm {
    pub(super) fn new(primary: WebView, secondaries: Vec<WebView>) -> Self {
        Self {
            primary,
            secondaries,
        }
    }

    pub(super) fn handle(&self, name: &str, json_params: &str) -> Variant {
        let json_result = if "broadcast" == name && json_params != "[]" {
            self.broadcast(json_params)
        } else {
            "undefined"
        };
        json_result.to_variant()
    }

    pub(super) fn primary(&self) -> &WebView {
        &self.primary
    }

    pub(super) fn secondaries(&self) -> &[WebView] {
        &self.secondaries
    }

    fn broadcast(&self, json_params: &str) -> &str {
        [&self.primary]
            .into_iter()
            .chain(&self.secondaries)
            .for_each(|webview| {
                let parameters = ["_emit", json_params].to_variant();
                let message = UserMessage::new("greeter_comm", Some(&parameters));
                webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
            });
        "null"
    }
}
