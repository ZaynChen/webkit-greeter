// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod application;
mod bridge;
mod config;
mod constants;
mod theme;
mod webview;
mod window;

use gtk::{gio, glib, prelude::*};

use crate::{
    application::{on_activate, on_startup},
    config::Config,
    constants::{APPLICATION_ID, WEBKIT_APPLICATION_INFO},
    theme::print_themes,
};

fn main() -> glib::ExitCode {
    // WebKitGTK 2.41.1 is the first unstable release of this cycle
    // and already includes the DMABUF support that is used by default.
    // We encourage everybody to try it out and provide feedback or report any issue.
    // Please, export the contents of webkit://gpu and attach it to the bug report
    // when reporting any problem related to graphics.
    // To check if the issue is a regression of the DMABUF implementation you can
    // use WEBKIT_DISABLE_DMABUF_RENDERER=1 to use the WPE renderer or X11 instead.
    // This environment variable and the WPE render/X11 code will be eventually removed
    // if DMABUF works fine. -- https://planet.webkitgtk.org/
    //
    // HACK: disable webkitgtk DMABUF renderer
    unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };

    logger::logger_init(logger::LevelFilter::Debug);
    eprintln!("username={:?}", glib::user_name());
    eprintln!("cache={:?}", glib::user_cache_dir());
    eprintln!("config={:?}", glib::user_config_dir());
    eprintln!("data={:?}", glib::user_data_dir());
    eprintln!("runtime={:?}", glib::user_runtime_dir());
    eprintln!("sys-config={:?}", glib::system_config_dirs());
    eprintln!("sys-data={:?}", glib::system_data_dirs());
    eprintln!("{:?}", std::env::vars());

    let args = CliArgs::parse();
    let config = Config::new(args.debug_mode(), args.theme());
    eprintln!("{config:?}");

    if args.list {
        print_themes(config.themes_dir());
        return glib::ExitCode::SUCCESS;
    }

    register_resources();

    let webinfo = webkit::ApplicationInfo::new();
    webinfo.set_name(WEBKIT_APPLICATION_INFO);

    let app = gtk::Application::builder()
        .application_id(APPLICATION_ID)
        .flags(Default::default())
        .build();

    app.connect_activate(move |app| on_activate(app, &config));
    app.connect_startup(on_startup);

    let exit_code = app.run_with_args::<glib::GString>(&[]);
    logger::debug!("WebKit Greeter stopped");
    exit_code
}

// before Application created
fn register_resources() {
    gio::resources_register_include!("greeter.gresource").expect("Failed to register resources.");
    greeters::register_api_resource();
}

use clap::{Parser, ValueEnum};
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Debug,
    Normal,
}

/// A modern, visually appealing greeter.
#[derive(Debug, Parser)]
#[command(version, about)]
struct CliArgs {
    /// Debug mode
    #[arg(short, long, group = "debug_mode")]
    debug: bool,
    /// Normal mode
    #[arg(short, long, group = "debug_mode")]
    normal: bool,
    /// Mode
    #[arg(long, group = "debug_mode")]
    mode: Option<Mode>,
    /// Theme
    #[arg(long)]
    theme: Option<String>,
    /// List installed themes
    #[arg(long)]
    list: bool,
}

impl CliArgs {
    fn debug_mode(&self) -> bool {
        self.debug || self.mode == Some(Mode::Debug)
    }

    fn theme(&self) -> Option<&str> {
        self.theme.as_deref()
    }
}
