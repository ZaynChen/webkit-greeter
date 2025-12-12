// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    gio::Cancellable,
    glib::{Variant, variant::ToVariant},
};
use jsc::JSCValueExtManual;
use webkit::{WebView, prelude::WebViewExt};

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
        let params = jsc::Value::from_json(context, json_params).to_vec();
        let ret = if "broadcast" == name && !params.is_empty() {
            self.broadcast(&params)
        } else {
            jsc::Value::new_undefined(context)
        };

        if let Some(json) = ret.to_json(0) {
            json.to_variant()
        } else {
            "undefined".to_variant()
        }
    }

    pub(super) fn primary(&self) -> &WebView {
        &self.primary
    }

    pub(super) fn secondaries(&self) -> &[WebView] {
        &self.secondaries
    }

    fn broadcast(&self, args: &[jsc::Value]) -> jsc::Value {
        let context = &self.context;
        [&self.primary]
            .into_iter()
            .chain(&self.secondaries)
            .for_each(|webview| {
                let parameters = [
                    "_emit",
                    jsc::Value::new_array_from_garray(context, args)
                        .to_json(0)
                        .expect("greeter_comm._emit paramerter errors")
                        .as_str(),
                ]
                .to_variant();
                let message = webkit::UserMessage::new("greeter_comm", Some(&parameters));
                webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
            });
        jsc::Value::new_null(context)
    }
}
