// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use webkit::{WebView, glib::Variant};

#[cfg(feature = "greetd")]
use greeters::GreetdGreeter;
#[cfg(feature = "lightdm")]
use greeters::LightDMGreeter;

pub(super) struct Greeter {
    display_manager: String,
    #[cfg(feature = "greetd")]
    greetd: Option<GreetdGreeter>,
    #[cfg(feature = "lightdm")]
    lightdm: Option<LightDMGreeter>,
}

impl Greeter {
    pub fn new(context: jsc::Context, webview: &WebView, display_manager: &str) -> Self {
        logger::info!("greeter new");
        match display_manager {
            #[cfg(feature = "greetd")]
            "greetd" => Self {
                display_manager: display_manager.to_string(),
                greetd: Some(GreetdGreeter::new(context, webview)),
                #[cfg(feature = "lightdm")]
                lightdm: None,
            },
            #[cfg(feature = "lightdm")]
            "lightdm" => Self {
                display_manager: display_manager.to_string(),
                #[cfg(feature = "greetd")]
                greetd: None,
                lightdm: Some(LightDMGreeter::new(context, webview)),
            },
            dm => unimplemented!("Unsupported display manager: {dm}"),
        }
    }

    pub fn handle(&self, name: &str, json_params: &str) -> Variant {
        match self.display_manager.as_str() {
            #[cfg(feature = "greetd")]
            "greetd" => self.greetd.as_ref().unwrap().handle(name, json_params),
            #[cfg(feature = "lightdm")]
            "lightdm" => self.lightdm.as_ref().unwrap().handle(name, json_params),
            dm => unreachable!("Unsupported display manager: {dm}"),
        }
    }
}

#[cfg(all(not(feature = "greetd"), not(feature = "lightdm")))]
compile_error!("feature \"greetd\" and feature \"lightdm\" cannot be enabled at the same time");
