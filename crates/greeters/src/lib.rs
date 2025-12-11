// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

#[cfg(all(feature = "lightdm", not(feature = "greetd")))]
mod lightdm;
#[cfg(all(feature = "lightdm", not(feature = "greetd")))]
pub use lightdm::Greeter;

#[cfg(all(feature = "greetd", not(feature = "lightdm")))]
mod greetd;
#[cfg(all(feature = "greetd", not(feature = "lightdm")))]
pub use greetd::Greeter;

use webkit::{
    gio::{File, resources_register_include},
    gtk::prelude::*,
};

pub fn register_api_resource() {
    resources_register_include!("greeters.gresource")
        .expect("Failed to register greeters resources.");
}

pub fn greeter_api() -> String {
    #[cfg(all(feature = "greetd", not(feature = "lightdm")))]
    let uri = "resource:///com/github/zaynchen/webkit-greeter/greetd.js";
    #[cfg(all(feature = "lightdm", not(feature = "greetd")))]
    let uri = "resource:///com/github/zaynchen/webkit-greeter/lightdm.js";

    if let Ok((content, _)) = File::for_uri(uri).load_contents(webkit::gio::Cancellable::NONE) {
        String::from_utf8(content.to_vec()).unwrap()
    } else {
        "".to_string()
    }
}

#[cfg(all(feature = "lightdm", feature = "greetd"))]
compile_error!("multiple greeter features set");
