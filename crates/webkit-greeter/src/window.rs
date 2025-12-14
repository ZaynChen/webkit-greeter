// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    Application, ApplicationWindow,
    gdk::Monitor,
    gio::{ActionEntry, SimpleAction},
    glib::{self, Variant, clone},
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
        new_action_entry(
            "undo",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_UNDO)
            ),
        ),
        new_action_entry(
            "redo",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_REDO)
            ),
        ),
        new_action_entry(
            "copy",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_COPY)
            ),
        ),
        new_action_entry(
            "cut",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_CUT)
            ),
        ),
        new_action_entry(
            "paste",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_PASTE)
            ),
        ),
        new_action_entry(
            "paste-plain",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview
                    .execute_editing_command(webkit::EDITING_COMMAND_PASTE_AS_PLAIN_TEXT)
            ),
        ),
        new_action_entry(
            "select-all",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.execute_editing_command(webkit::EDITING_COMMAND_SELECT_ALL)
            ),
        ),
        new_action_entry(
            "zoom-normal",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.set_zoom_level(1f64)
            ),
        ),
        new_action_entry(
            "zoom-in",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.set_zoom_level(next_zoom_level(webview.zoom_level(), 1))
            ),
        ),
        new_action_entry(
            "zoom-out",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.set_zoom_level(next_zoom_level(webview.zoom_level(), -1))
            ),
        ),
        new_action_entry(
            "reload",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.reload()
            ),
        ),
        new_action_entry(
            "force-reload",
            clone!(
                #[weak]
                webview,
                move |_, _, _| webview.reload_bypass_cache()
            ),
        ),
    ];
    window.add_action_entries(win_entries);

    if debug {
        let win_debug_entries = [
            new_action_entry(
                "toggle-inspector",
                clone!(
                    #[weak]
                    webview,
                    move |_, _, _| {
                        let inspector = webview.inspector().unwrap();

                        match inspector.web_view() {
                            None => inspector.show(),
                            Some(_) => inspector.close(),
                        }
                    }
                ),
            ),
            new_action_entry("fullscreen", |window: &ApplicationWindow, _, _| {
                let is_fullscreend = window.is_fullscreen();
                window.set_show_menubar(is_fullscreend);
                window.set_fullscreened(!is_fullscreend);
            }),
            new_action_entry("close", |window: &ApplicationWindow, _, _| window.close()),
            new_action_entry("minimize", |window: &ApplicationWindow, _, _| {
                window.minimize()
            }),
        ];
        window.add_action_entries(win_debug_entries);
    }
}

fn new_action_entry<F>(name: &str, callback: F) -> ActionEntry<ApplicationWindow>
where
    F: Fn(&ApplicationWindow, &SimpleAction, Option<&Variant>) + 'static,
{
    ActionEntry::builder(name).activate(callback).build()
}

const ZOOM_LEVELS: [f64; 12] = [
    0.33f64, 0.50f64, 0.75f64, 0.85f64, 1.00f64, 1.15f64, 1.25f64, 1.50f64, 1.75f64, 2.00f64,
    2.50f64, 3.00f64,
];

fn next_zoom_level(current_level: f64, steps: isize) -> f64 {
    let pos = ZOOM_LEVELS.iter().position(|z| &current_level <= z);
    let next_pos = match pos {
        None => (ZOOM_LEVELS.len() as isize - 1 + steps).clamp(0, 11) as usize,
        Some(idx) => (idx as isize + steps).clamp(0, 11) as usize,
    };
    ZOOM_LEVELS[next_pos]
}
