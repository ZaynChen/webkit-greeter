// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use webkit::gtk::glib::{Variant, variant::ToVariant};

use std::ops::{Deref, DerefMut};

use crate::config::Config;

pub(super) struct GreeterConfig {
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
    pub(super) fn new(config: Config) -> Self {
        Self { config }
    }

    pub(super) fn handle(&self, method: &str) -> Variant {
        let json_result = match method {
            "branding" => serde_json::to_string(self.branding()).unwrap(),
            "greeter" => serde_json::to_string(self.greeter()).unwrap(),
            _ => "undefined".to_string(),
        };
        json_result.to_variant()
    }
}
