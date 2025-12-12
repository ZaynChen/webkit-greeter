// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    Application, ApplicationWindow,
    gdk::Monitor,
    gio::ActionEntry,
    glib::{self, clone},
    prelude::*,
};
use webkit::{WebView, prelude::WebViewExt};

pub fn setup_window(webview: &WebView, app: &Application, monitor: &Monitor, debug_mode: bool) {
    let window = ApplicationWindow::new(app);
    window.set_child(Some(webview));
    setup_actions(&window, webview, debug_mode);
    setup_style(&window, monitor, debug_mode);
}

fn setup_style(window: &ApplicationWindow, monitor: &Monitor, debug: bool) {
    window.set_cursor_from_name(Some("default"));
    let geometry = monitor.geometry();
    window.set_default_size(geometry.width(), geometry.height());
    window.set_show_menubar(debug);
    window.fullscreen_on_monitor(monitor);
}

fn setup_actions(window: &ApplicationWindow, webview: &WebView, debug: bool) {
    let win_entries = [
        ActionEntry::builder("undo")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_UNDO)
            ))
            .build(),
        ActionEntry::builder("redo")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_REDO)
            ))
            .build(),
        ActionEntry::builder("copy")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_COPY)
            ))
            .build(),
        ActionEntry::builder("cut")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_CUT)
            ))
            .build(),
        ActionEntry::builder("paste")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_PASTE)
            ))
            .build(),
        ActionEntry::builder("paste-plain")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview
                    .execute_editing_command(webkit::EDITING_COMMAND_PASTE_AS_PLAIN_TEXT)
            ))
            .build(),
        ActionEntry::builder("select-all")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_SELECT_ALL)
            ))
            .build(),
        ActionEntry::builder("zoom-normal")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.set_zoom_level(1f64)
            ))
            .build(),
        ActionEntry::builder("zoom-in")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.set_zoom_level(get_zoom_next_level(webview.zoom_level(), 1))
            ))
            .build(),
        ActionEntry::builder("zoom-out")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview
                    .set_zoom_level(get_zoom_next_level(webview.zoom_level(), -1))
            ))
            .build(),
        ActionEntry::builder("reload")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.reload()
            ))
            .build(),
        ActionEntry::builder("force-reload")
            .activate(clone!(
                #[weak]
                webview,
                move |_, _, _| webview.reload_bypass_cache()
            ))
            .build(),
    ];
    window.add_action_entries(win_entries);

    if debug {
        let win_debug_entries = [
            ActionEntry::builder("toggle-inspector")
                .activate(clone!(
                    #[weak]
                    webview,
                    move |_, _, _| {
                        let inspector = webview.inspector().unwrap();

                        match inspector.web_view() {
                            None => inspector.show(),
                            Some(_) => inspector.close(),
                        }
                    }
                ))
                .build(),
            ActionEntry::builder("fullscreen")
                .activate(|window: &ApplicationWindow, _, _| {
                    let is_fullscreend = window.is_fullscreen();
                    if is_fullscreend {
                        window.set_show_menubar(true);
                    } else {
                        window.set_show_menubar(false);
                    }
                    window.set_fullscreened(!is_fullscreend);
                })
                .build(),
            ActionEntry::builder("close")
                .activate(|window: &ApplicationWindow, _, _| window.close())
                .build(),
            ActionEntry::builder("minimize")
                .activate(|window: &ApplicationWindow, _, _| window.minimize())
                .build(),
        ];
        window.add_action_entries(win_debug_entries);
    }
}

const ZOOM_LEVELS: [f64; 12] = [
    0.33f64, 0.50f64, 0.75f64, 0.85f64, 1.00f64, 1.15f64, 1.25f64, 1.50f64, 1.75f64, 2.00f64,
    2.50f64, 3.00f64,
];

fn get_zoom_next_level(current_level: f64, steps: isize) -> f64 {
    let pos = ZOOM_LEVELS.iter().position(|z| &current_level <= z);
    let next_pos = match pos {
        None => (ZOOM_LEVELS.len() as isize - 1 + steps).clamp(0, 11) as usize,
        Some(idx) => (idx as isize + steps).clamp(0, 11) as usize,
    };
    ZOOM_LEVELS[next_pos]
}
