// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    Application, CssProvider,
    gdk::{Display, Monitor},
    gio::{ActionEntry, File, MenuModel},
    prelude::*,
};
use webkit::{CacheModel, WebContext, prelude::WebViewExt};

use crate::{
    config::Config,
    constants::{GREETER_RESOURCE_PREFIX, WEB_EXTENSIONS_DIR},
    service::Dispatcher,
    theme::load_theme_html,
    webview::{primary_user_message_received, secondary_user_message_received, webview_new},
    window::setup_window,
};

pub fn on_activate(app: &Application, config: &Config) {
    let dm = current_display_manager();
    {
        let webcontext = WebContext::default().expect("default web context does not exist");
        webcontext.set_cache_model(CacheModel::DocumentViewer);
        let secure_mode = config.secure_mode();
        let detect_theme_error = config.detect_theme_errors();
        let api = greeter_api(&dm);
        webcontext.connect_initialize_web_process_extensions(move |context: &WebContext| {
            let data = (secure_mode, detect_theme_error, &api).to_variant();
            log::debug!("Extension initialized");

            context.set_web_process_extensions_directory(WEB_EXTENSIONS_DIR);
            context.set_web_process_extensions_initialization_user_data(&data);
        });
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

    if display.backend().is_x11()
        && let Err(e) = x11_set_cursor()
    {
        log::error!("{e}");
    }

    // In wayland, the position of a window is determined by the compositor,
    // and the application of that window does not have the position information
    // for it to determine where it can move to.
    // Therefore, in wayland, is_primary seems only used
    // for contrainting the applicatin to construct only one primary window,
    // and does not determine which monitor it should present on,
    let (primary, secondaries) = {
        let monitors = display.monitors();
        let primary_monitor = if monitors.n_items() > 1 {
            config.primary_monitor().unwrap_or("0")
        } else {
            "0"
        };
        let (primary_monitors, secondary_monitors): (Vec<_>, Vec<_>) = monitors
            .iter::<Monitor>()
            .filter_map(|m| m.ok())
            .enumerate()
            .partition(|(idx, m)| {
                idx.to_string() == primary_monitor
                    || m.connector().as_deref() == Some(primary_monitor)
            });

        let debug = config.debug_mode();
        let (primary_html, secondary_html) = load_theme_html(config.themes_dir(), config.theme());

        let primary = webview_new(debug, &primary_html);
        let (_, primary_monitor) = primary_monitors
            .first()
            .unwrap_or_else(|| panic!("primary monitor \"{primary_monitor}\" does not exist"));
        setup_window(&primary, app, primary_monitor, debug);
        primary.grab_focus();

        let secondaries = secondary_monitors
            .iter()
            .map(|(_, monitor)| {
                let secondary = webview_new(debug, &secondary_html);
                setup_window(&secondary, app, monitor, debug);
                secondary.connect_user_message_received(secondary_user_message_received);
                secondary
            })
            .collect();

        (primary, secondaries)
    };

    let dispatcher = Dispatcher::new(config.clone(), primary.clone(), secondaries, &dm);
    primary.connect_user_message_received(move |webview, message| {
        primary_user_message_received(webview, message, &dispatcher)
    });
}

pub fn on_startup(app: &Application) {
    app.set_accels_for_action("app.quit", &["<Ctl>Q"]);
    app.add_action_entries([ActionEntry::builder("quit")
        .activate(|app: &Application, _, _| app.quit())
        .build()]);

    app.set_accels_for_action("win.undo", &["<Ctl>Z"]);
    app.set_accels_for_action("win.redo", &["<Ctl><Shift>Z"]);
    app.set_accels_for_action("win.cut", &["<Ctl>X"]);
    app.set_accels_for_action("win.copy", &["<Ctl>C"]);
    app.set_accels_for_action("win.paste", &["<Ctl>V"]);
    app.set_accels_for_action("win.paste-plain", &["<Ctl><Shift>V"]);
    app.set_accels_for_action("win.select-all", &["<Ctl>A"]);

    app.set_accels_for_action("win.reload", &["<Ctl>R", "F5", "Refresh", "Reload"]);
    app.set_accels_for_action("win.force-reload", &["<Ctl><Shift>R", "<Shift>F5"]);
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

    app.set_accels_for_action("win.toggle-inspector", &["<Ctl><Shift>I", "F12"]);
    app.set_accels_for_action("win.minimize", &["<Ctl>M"]);
    app.set_accels_for_action("win.close", &["<Ctl>W"]);

    app.set_menubar(
        gtk::Builder::from_resource(&format!("{GREETER_RESOURCE_PREFIX}/menubar.ui"))
            .object::<MenuModel>("menu")
            .as_ref(),
    );
}

fn greeter_api(dm: &str) -> String {
    let uri = format!("resource:///com/github/zaynchen/webkit-greeter/{dm}.js");

    match File::for_uri(&uri).load_contents(webkit::gio::Cancellable::NONE) {
        Ok((content, _)) => String::from_utf8(content.to_vec()).unwrap(),
        Err(e) => {
            panic!("Failed to read {uri}: {e}");
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
            log::error!("Failed to get current display manager by systemd: {e}");
            "".to_string()
        }
    }
}

fn x11_set_cursor() -> Result<(), Box<dyn std::error::Error>> {
    use x11rb::{
        connection::Connection,
        protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt},
    };

    let (conn, screen_id) = match x11rb::connect(None) {
        Ok(reply) => reply,
        Err(_) => return Ok(()),
    };
    log::debug!("Setup root window cursor: GDK backend is X11");

    let font = conn.generate_id()?;
    conn.open_font(font, b"cursor")?;

    let cursor = conn.generate_id()?;
    conn.create_glyph_cursor(
        cursor,
        font,
        font,
        68,
        68 + 1,
        0,
        0,
        0,
        u16::MAX,
        u16::MAX,
        u16::MAX,
    )?;

    let attrs = ChangeWindowAttributesAux::default().cursor(cursor);
    conn.change_window_attributes(conn.setup().roots[screen_id].root, &attrs)?;

    conn.flush()?;
    conn.free_cursor(cursor)?;
    conn.close_font(font)?;

    Ok(())
}
