// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use webkit::gtk::{
    self, Application, CssProvider,
    gdk::{Display, Monitor},
    gio::{ActionEntry, MenuModel},
    glib::{self, translate::*},
    prelude::*,
};

use std::rc::Rc;

use crate::{
    bridge::Dispatcher,
    browser::Browser,
    config::Config,
    constants::{GREETER_RESOURCE_PREFIX, WEB_EXTENSIONS_DIR},
    webview::webview_new,
};

pub fn on_activate(app: &Application, config: &Config) {
    {
        let webcontext = webkit::WebContext::default().expect("default web context does not exist");
        webcontext.set_cache_model(webkit::CacheModel::DocumentViewer);
        let secure_mode = config.secure_mode();
        let detect_theme_error = config.detect_theme_errors();
        let api = greeters::greeter_api();
        webcontext.connect_initialize_web_process_extensions(
            move |context: &webkit::WebContext| {
                let data = (secure_mode, detect_theme_error, &api).to_variant();
                logger::debug!("Extension initialized");

                context.set_web_process_extensions_directory(*WEB_EXTENSIONS_DIR);
                context.set_web_process_extensions_initialization_user_data(&data);
            },
        );
    }

    let display = Display::default().expect("Default display does not exist");

    {
        let provider = CssProvider::new();
        provider.load_from_resource(&format!("{GREETER_RESOURCE_PREFIX}/style.css"));
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    #[cfg(feature = "x11")]
    set_cursor(&display);

    // In wayland, the position of a window is determined by the compositor,
    // and the application of that window does not have the position information
    // for it to determine where it can move to.
    // Therefore, in wayland, is_primary seems only used
    // for contrainting the applicatin to construct only one primary window,
    // and does not determine which monitor it should present on,
    let browsers: Vec<Browser> = {
        let monitors = display.monitors();
        let only_one_monitor = monitors.n_items() == 1u32;
        let primary_monitor = config.primary_monitor();
        let debug = config.debug_mode();
        let (primary, secondary) = config.theme_file();
        monitors
            .iter::<Monitor>()
            .filter_map(|m| m.ok())
            .enumerate()
            .map(|(idx, m)| {
                let id = gen_id(&m);
                let geometry = m.geometry();
                let is_primary = only_one_monitor
                    || Some(idx.to_string().as_ref()) == primary_monitor
                    || m.connector().as_deref() == primary_monitor;
                let theme_file = if is_primary { &primary } else { &secondary };
                Browser::builder()
                    .debug_mode(debug)
                    .id(id)
                    .geometry(geometry)
                    .primary(is_primary)
                    .application(app)
                    .webview(webview_new(debug, theme_file))
                    .build()
            })
            .collect()
    };
    let browsers = Rc::new(browsers);
    let dispatcher = Rc::new(Dispatcher::new(
        config.clone(),
        jsc::Context::default(),
        browsers.clone(),
    ));
    browsers.iter().for_each(|browser| {
        browser.connect_user_message_received(dispatcher.clone());
    })
}

pub fn on_startup(app: &Application) {
    app.set_accels_for_action("app.quit", &["<Ctl>Q"]);
    app.set_accels_for_action("win.toggle-inspector", &["<Ctl><Shift>I", "F12"]);

    app.set_accels_for_action("win.undo", &["<Ctl>Z"]);
    app.set_accels_for_action("win.redo", &["<Ctl><Shift>Z"]);
    app.set_accels_for_action("win.cut", &["<Ctl>X"]);
    app.set_accels_for_action("win.copy", &["<Ctl>C"]);
    app.set_accels_for_action("win.paste", &["<Ctl>V"]);
    app.set_accels_for_action("win.paste-plain", &["<Ctl><Shift>V"]);
    app.set_accels_for_action("win.select-all", &["<Ctl>A"]);

    app.set_accels_for_action("win.zoom-normal", &["<Ctl>0", "<Ctl>KP_0"]);
    app.set_accels_for_action(
        "win.zoom-in",
        &["<Ctl>plus", "<Ctl>equal", "<Ctl>KP_Add", "ZoomIn"],
    );
    app.set_accels_for_action(
        "win.zoom-out",
        &["<Ctl>minus", "<Ctl>KP_Subtract", "ZoomOut"],
    );
    app.set_accels_for_action("win.fullscreen", &["F11"]);
    app.set_accels_for_action("win.reload", &["<Ctl>R", "F5", "Refresh", "Reload"]);
    app.set_accels_for_action("win.force-reload", &["<Ctl><Shift>R", "<Shift>F5"]);

    app.set_accels_for_action("win.close", &["<Ctl>W"]);
    app.set_accels_for_action("win.minimize", &["<Ctl>M"]);

    app.add_action_entries([ActionEntry::builder("quit")
        .activate(|app: &Application, _, _| app.quit())
        .build()]);

    app.set_menubar(
        gtk::Builder::from_resource(&format!("{GREETER_RESOURCE_PREFIX}/menubar.ui"))
            .object::<MenuModel>("menu")
            .as_ref(),
    );
}

#[cfg(feature = "x11")]
fn set_cursor(display: &gtk::gdk::Display) {
    if display.backend().is_x11() {
        logger::debug!("Setup root window cursor: GDK backend is X11");
        let display = display
            .downcast_ref::<gdkx::X11Display>()
            .expect("the display should be x11");
        let root_window = display.xrootwindow();
        unsafe {
            let cursor = gdkx::x11::xlib::XCreateFontCursor(display.xdisplay(), 68);
            gdkx::x11::xlib::XDefineCursor(display.xdisplay(), root_window, cursor);
        }
    }
}

fn gen_id(monitor: &Monitor) -> u64 {
    let manufacture = monitor.manufacturer();
    let model = monitor.model();
    let manufacture_hash = manufacture.map_or(0, |m| unsafe {
        glib::ffi::g_str_hash(m.into_glib_ptr() as glib::ffi::gconstpointer)
    }) as u64;
    let model_hash = model.map_or(0, |m| unsafe {
        glib::ffi::g_str_hash(m.into_glib_ptr() as glib::ffi::gconstpointer)
    }) as u64;

    (manufacture_hash << 24) | (model_hash << 8)
}
