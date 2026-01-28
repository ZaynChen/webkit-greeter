// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later
mod common;
mod jscvalue;

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
    let dm = current_display_manager();
    let uri = format!("resource:///com/github/zaynchen/webkit-greeter/{dm}.js");

    match File::for_uri(&uri).load_contents(webkit::gio::Cancellable::NONE) {
        Ok((content, _)) => String::from_utf8(content.to_vec()).unwrap(),
        Err(e) => {
            logger::error!("Failed to read {uri}: {e}");
            "".to_string()
        }
    }
}

// Get current displaymanager managed by systemd.
fn current_display_manager() -> String {
    match std::process::Command::new("systemctl")
        .arg("--property=Id")
        .arg("show")
        .arg("display-manager")
        .output()
    {
        Ok(output) => String::from_utf8(output.stdout)
            .expect("The output of 'systemctl show display-manager' is not encoded as utf8")
            .trim()
            .strip_prefix("Id=")
            .unwrap()
            .strip_suffix(".service")
            .unwrap()
            .to_string(),
        Err(e) => {
            logger::error!("Failed to get current display manager by systemd: {e}");
            "".to_string()
        }
    }
}

#[cfg(all(feature = "lightdm", feature = "greetd"))]
compile_error!("multiple greeter features set");
