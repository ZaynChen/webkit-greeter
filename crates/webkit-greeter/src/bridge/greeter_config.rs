// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use jsc::JSCValueExtManual;
use webkit::gtk::glib::{Variant, variant::ToVariant};

use std::ops::{Deref, DerefMut};

use crate::config::Config;

pub(super) struct GreeterConfig {
    context: jsc::Context,
    config: Config,
}

impl Deref for GreeterConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for GreeterConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl GreeterConfig {
    pub(super) fn new(context: jsc::Context, config: Config) -> Self {
        Self { context, config }
    }

    pub(super) fn handle(&self, name: &str) -> Variant {
        let context = &self.context;
        let ret = match name {
            "branding" => self.branding(),
            "greeter" => self.greeter(),
            // "layouts" => self.layouts(),
            _ => jsc::Value::new_undefined(context),
        };

        if let Some(json) = ret.to_json(0) {
            json.to_variant()
        } else {
            "undefined".to_variant()
        }
    }

    fn branding(&self) -> jsc::Value {
        let images_dir = self.background_images_dir();
        let logo_image = self.logo_image();
        let user_image = self.user_image();

        let context = &self.context;
        let value = jsc::Value::new_object(context, None, None);
        value.object_set_property(
            "background_images_dir",
            &jsc::Value::new_string(context, Some(images_dir)),
        );
        value.object_set_property(
            "logo_image",
            &jsc::Value::new_string(context, Some(logo_image)),
        );
        value.object_set_property(
            "user_image",
            &jsc::Value::new_string(context, Some(user_image)),
        );

        value
    }

    fn greeter(&self) -> jsc::Value {
        let debug_mode = self.debug_mode();
        let detect_theme_errors = self.detect_theme_errors();
        let screensaver_timeout = self.screensaver_timeout();
        let secure_mode = self.secure_mode();
        let theme = self.theme();
        let icon_theme = self.icon_theme();
        let time_language = self.time_language();

        let context = &self.context;
        let value = jsc::Value::new_object(context, None, None);
        value.object_set_property("debug_mode", &jsc::Value::new_boolean(context, debug_mode));
        value.object_set_property(
            "detect_theme_errors",
            &jsc::Value::new_boolean(context, detect_theme_errors),
        );
        value.object_set_property(
            "screensaver_timeout",
            &jsc::Value::new_number(context, screensaver_timeout as f64),
        );
        value.object_set_property(
            "secure_mode",
            &jsc::Value::new_boolean(context, secure_mode),
        );
        value.object_set_property("theme", &jsc::Value::new_string(context, Some(theme)));
        value.object_set_property("icon_theme", &jsc::Value::new_string(context, icon_theme));
        value.object_set_property(
            "time_language",
            &jsc::Value::new_string(context, time_language),
        );

        value
    }

    // fn layouts(&self) -> jsc::Value {
    //     let context = &self.context;
    //     let layouts = self
    //         .config
    //         .layouts()
    //         .iter()
    //         .map(|l| jsc::Value::new_string(context, Some(l)))
    //         .collect::<Vec<_>>();
    //
    //     jsc::Value::new_array_from_garray(context, layouts.as_slice())
    // }
}
