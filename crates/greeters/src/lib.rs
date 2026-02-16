// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

mod common;

mod greeters;
#[cfg(feature = "greetd")]
pub use greeters::GreetdGreeter;
#[cfg(feature = "lightdm")]
pub use greeters::LightDMGreeter;

use webkit::{
    gio::{File, resources_register_include},
    gtk::prelude::*,
};

pub fn register_api_resource() {
    resources_register_include!("greeters.gresource")
        .expect("Failed to register greeters resources.");
}

pub fn greeter_api(dm: &str) -> String {
    let uri = format!("resource:///com/github/zaynchen/webkit-greeter/{dm}.js");

    match File::for_uri(&uri).load_contents(webkit::gio::Cancellable::NONE) {
        Ok((content, _)) => String::from_utf8(content.to_vec()).unwrap(),
        Err(e) => {
            logger::error!("Failed to read {uri}: {e}");
            "".to_string()
        }
    }
}
