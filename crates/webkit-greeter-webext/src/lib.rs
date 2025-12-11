// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: Apache-2.0

mod extension;

use wwpe::{
    ConsoleMessageLevel, ConsoleMessageSource, UserMessage, WebPage, WebProcessExtension,
    ffi::WebKitWebProcessExtension,
    gio::Cancellable,
    glib::{
        self, DateTime, MainContext, Variant, clone,
        ffi::GVariant,
        translate::*,
        variant::{FromVariant, ToVariant},
    },
};

use std::{cell::Cell, rc::Rc};

fn web_page_created(page: &WebPage, secure_mode: bool, detect_theme_errors: bool) {
    let stop_prompts = Rc::new(Cell::new(false));

    page.connect_document_loaded(clone!(
        #[strong]
        stop_prompts,
        move |page| {
            stop_prompts.set(false);
            let message = wwpe::UserMessage::new("ready-to-show", None);
            page.send_message_to_view(&message, Cancellable::NONE, |_| {});
        }
    ));

    page.connect_console_message_sent(clone!(
        #[strong]
        stop_prompts,
        move |page, message| {
            let message = &mut message.clone();
            let text = message.text().unwrap().to_string();
            let source = message.source();
            let source_id = message.source_id().unwrap().to_string();
            let line = message.line();

            let timestamp = DateTime::now_local()
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .unwrap();
            match message.level() {
                ConsoleMessageLevel::Error => {
                    eprintln!("{timestamp} [ ERROR ] {source_id} {line}: {text}");
                    if !stop_prompts.get()
                        && detect_theme_errors
                        && source != ConsoleMessageSource::Network
                    {
                        let params = ("ERROR", text, source_id, line).to_variant();
                        let message = UserMessage::new("console", Some(&params));
                        if let Ok(reply) = MainContext::default()
                            .block_on(page.send_message_to_view_future(&message))
                        {
                            stop_prompts.set(
                                reply
                                    .parameters()
                                    .as_ref()
                                    .map(bool::from_variant)
                                    .unwrap_or(None)
                                    .unwrap_or(false),
                            );
                        }
                    }
                }
                ConsoleMessageLevel::Warning => {
                    eprintln!("{timestamp} [ WARNING ] {source_id} {line}: {text}");
                }
                _ => {}
            }
        }
    ));

    if secure_mode {
        page.connect_send_request(|_, request, _| {
            let uri = request.uri().unwrap();
            let scheme = glib::uri_parse_scheme(uri).unwrap();
            !matches!(scheme.as_str(), "file" | "data" | "webkit-greeter")
        });
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn webkit_web_process_extension_initialize_with_user_data(
    extension: *mut WebKitWebProcessExtension,
    user_data: *const GVariant,
) {
    logger::logger_init(logger::LevelFilter::Debug);

    let user_data: Variant = unsafe { from_glib_none(user_data) };
    let secure_mode =
        bool::from_variant(&user_data.child_value(0)).expect("secure_mode is not a bool");
    let detect_theme_errors =
        bool::from_variant(&user_data.child_value(1)).expect("detect_theme_errors is not a bool");

    let extention: WebProcessExtension = unsafe { from_glib_none(extension) };
    extention.connect_page_created(move |_, page| {
        web_page_created(page, secure_mode, detect_theme_errors)
    });

    let greeter_api_script = String::from_variant(&user_data.child_value(2))
        .expect("greeter_api_script is not a String");
    crate::extension::web_page_initialize(greeter_api_script);
}
