// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{AlertDialog, ApplicationWindow, gdk, gio::Cancellable};
use webkit::{HardwareAccelerationPolicy, Settings, UserMessage, WebView, prelude::*};

use crate::{bridge::Dispatcher, constants::DEFAULT_THEME, theme::load_theme_html};

pub fn webview_new(debug: bool, theme_file: &str) -> WebView {
    let settings = Settings::builder()
        .allow_file_access_from_file_urls(true)
        .allow_universal_access_from_file_urls(true)
        .enable_page_cache(true)
        .enable_html5_local_storage(true)
        .enable_webgl(true)
        .hardware_acceleration_policy(HardwareAccelerationPolicy::Always)
        .enable_developer_extras(debug)
        .build();

    let webview = WebView::builder().settings(&settings).build();

    let rgba = gdk::RGBA::parse("#000000").unwrap();
    webview.set_background_color(&rgba);

    let uri = "file://".to_string() + theme_file;
    webview.load_uri(&uri);
    logger::debug!("Theme loaded");

    webview
}

pub fn primary_user_message_received(
    webview: &WebView,
    message: &UserMessage,
    dispatcher: &Dispatcher,
) -> bool {
    match message.name().as_deref() {
        Some("ready-to-show") => {
            let root = webview.root().expect("webview.root is None");
            let window = root
                .downcast_ref::<ApplicationWindow>()
                .expect("webview.root is not a ApplicationWindow");
            window.present();
            logger::debug!("WebKit Greeter started win: {}", window.id());
            true
        }
        Some("console") => {
            show_console_error_prompt(message.clone(), dispatcher);
            true
        }
        Some(_) => {
            dispatcher.send(message);
            true
        }
        None => false,
    }
}

pub fn secondary_user_message_received(webview: &WebView, message: &UserMessage) -> bool {
    if !matches!(message.name().as_deref(), Some("ready-to-show")) {
        return false;
    }
    let root = webview.root().expect("webview.root is None");
    let window = root
        .downcast_ref::<ApplicationWindow>()
        .expect("webview.root is not a ApplicationWindow");
    window.present();
    true
}

pub fn show_console_error_prompt(message: UserMessage, dispatcher: &Dispatcher) {
    let params = message.parameters().unwrap();

    let msg_var = params.child_value(1);
    let source_id_var = params.child_value(2);
    let msg = msg_var.str().unwrap();
    let source_id = source_id_var.str().unwrap();
    let line = u32::from_variant(&params.child_value(3)).unwrap();

    let primary = dispatcher.primary().clone();
    let secondaries = dispatcher.secondaries().to_vec();
    let root = primary.root().expect("webview.root is None");
    let window = root
        .downcast_ref::<ApplicationWindow>()
        .expect("webview.root is not a ApplicationWindow");
    let themes_dir = dispatcher.themes_dir();
    AlertDialog::builder()
        .message("An error ocurred. Change to default theme? (litarvan)")
        .detail(format!("{source_id} {line}: {msg}"))
        .buttons(["Cancel", "Use default theme", "Reload theme"])
        .build()
        .choose(Some(window), Some(&Cancellable::new()), move |res| {
            if let Ok(stop_prompts) = res.map(|response| match response {
                1 => {
                    let (phtml, shtml) = load_theme_html(&themes_dir, DEFAULT_THEME);
                    primary.load_uri(&phtml);
                    secondaries
                        .iter()
                        .for_each(|webview| webview.load_uri(&shtml));
                    true
                }
                2 => {
                    primary.reload();
                    secondaries.iter().for_each(|webview| webview.reload());
                    true
                }
                _ => false,
            }) {
                message.send_reply(&UserMessage::new(
                    "console-done",
                    Some(&stop_prompts.to_variant()),
                ))
            }
        });
}
