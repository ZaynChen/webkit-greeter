// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    gdk::Rectangle,
    gio::Cancellable,
    glib::{Variant, variant::ToVariant},
};
use jsc::JSCValueExtManual;
use webkit::prelude::WebViewExt;

use std::rc::Rc;

use crate::browser::{Browser, BrowserProperties};

pub(super) struct GreeterComm {
    context: jsc::Context,
    browsers: Rc<Vec<Browser>>,
}

impl GreeterComm {
    pub(super) fn new(context: jsc::Context, browsers: Rc<Vec<Browser>>) -> Self {
        Self { context, browsers }
    }

    pub(super) fn handle(
        &self,
        name: &str,
        json_params: &str,
        props: &BrowserProperties,
    ) -> Variant {
        let context = &self.context;
        let params = jsc::Value::from_json(context, json_params).to_vec();
        let ret = if "window_metadata" == name && params.is_empty() {
            self.window_metadata(props)
        } else if "broadcast" == name && !params.is_empty() {
            self.greeter_comm_broadcast_cb(&params)
        } else {
            jsc::Value::new_undefined(context)
        };

        if let Some(json) = ret.to_json(0) {
            json.to_variant()
        } else {
            "undefined".to_variant()
        }
    }

    pub(super) fn load_theme(&self, primary: String, secondary: String) {
        self.browsers.iter().for_each(|browser| {
            let theme_file = if browser.primary() {
                &primary
            } else {
                &secondary
            };
            let uri = "file://".to_string() + theme_file;
            browser.webview().load_uri(&uri);
        });
    }

    fn greeter_comm_broadcast_cb(&self, args: &[jsc::Value]) -> jsc::Value {
        let context = &self.context;
        self.browsers
            .iter()
            .map(|b| b.webview())
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

    fn window_metadata(&self, props: &BrowserProperties) -> jsc::Value {
        let id = props.id;
        let is_primary = props.is_primary;
        let geometry = props.geometry;
        let (x, y, width, height) = (
            geometry.x() as f64,
            geometry.y() as f64,
            geometry.width() as f64,
            geometry.height() as f64,
        );

        let (min_x, min_y, max_x, max_y) = overall_boundary(&self.browsers);

        let context = &self.context;
        let position = jsc::Value::new_object(context, None, None);
        position.object_set_property("x", &jsc::Value::new_number(context, x));
        position.object_set_property("y", &jsc::Value::new_number(context, y));

        let size = jsc::Value::new_object(context, None, None);
        size.object_set_property("width", &jsc::Value::new_number(context, width));
        size.object_set_property("height", &jsc::Value::new_number(context, height));

        let overall_boundary = jsc::Value::new_object(context, None, None);
        overall_boundary.object_set_property("minX", &jsc::Value::new_number(context, min_x));
        overall_boundary.object_set_property("minY", &jsc::Value::new_number(context, min_y));
        overall_boundary.object_set_property("maxX", &jsc::Value::new_number(context, max_x));
        overall_boundary.object_set_property("maxY", &jsc::Value::new_number(context, max_y));

        let value = jsc::Value::new_object(context, None, None);
        value.object_set_property("id", &jsc::Value::new_number(context, id as f64));
        value.object_set_property("is_primary", &jsc::Value::new_boolean(context, is_primary));
        value.object_set_property("position", &position);
        value.object_set_property("size", &size);
        value.object_set_property("overallBoundary", &overall_boundary);

        value
    }
}

fn overall_boundary(browsers: &[Browser]) -> (f64, f64, f64, f64) {
    let geometries: Vec<&Rectangle> = browsers.iter().map(|browser| browser.geometry()).collect();
    let min_x = geometries.iter().map(|g| g.x()).min().unwrap() as f64;
    let min_y = geometries.iter().map(|g| g.y()).min().unwrap() as f64;
    let max_x = geometries.iter().map(|g| g.x() + g.width()).max().unwrap() as f64;
    let max_y = geometries.iter().map(|g| g.y() + g.height()).max().unwrap() as f64;
    (min_x, min_y, max_x, max_y)
}
