// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{
    Application, ApplicationWindow,
    gdk::Rectangle,
    gio::ActionEntry,
    glib::{self, clone},
    prelude::*,
};
use webkit::{WebView, prelude::WebViewExt};

use std::{cell::Cell, rc::Rc};

use crate::bridge::Dispatcher;

pub struct BrowserProperties {
    pub id: u64,
    pub geometry: Rectangle,
    pub is_primary: bool,
}

pub struct Browser {
    webview: WebView,
    properties: Rc<BrowserProperties>,
    loaded: Rc<Cell<bool>>,
}

impl Browser {
    pub fn builder() -> BrowserBuilder {
        BrowserBuilder::new()
    }

    pub fn geometry(&self) -> &Rectangle {
        &self.properties.geometry
    }

    pub fn webview(&self) -> &WebView {
        &self.webview
    }

    pub fn primary(&self) -> bool {
        self.properties.is_primary
    }

    pub fn connect_user_message_received(&self, dispatcher: Rc<Dispatcher>) {
        let win_props = &self.properties;
        let loaded = &self.loaded;
        self.webview.connect_user_message_received(clone!(
            #[strong]
            loaded,
            #[strong]
            dispatcher,
            #[strong]
            win_props,
            move |webview, message| {
                crate::webview::user_message_received(
                    webview,
                    message,
                    &dispatcher,
                    &loaded,
                    &win_props,
                )
            }
        ));
    }
}

#[must_use = "The builder must be built to be used"]
pub struct BrowserBuilder {
    id: u64,
    window: Option<ApplicationWindow>,
    webview: Option<WebView>,
    geometry: Option<Rectangle>,
    debug_mode: bool,
    is_primary: bool,
}

impl BrowserBuilder {
    fn new() -> Self {
        Self {
            id: 0,
            window: None,
            webview: None,
            geometry: None,
            debug_mode: false,
            is_primary: false,
        }
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    pub fn application(mut self, app: &Application) -> Self {
        self.window.replace(ApplicationWindow::new(app));
        self
    }

    pub fn webview(mut self, webview: WebView) -> Self {
        self.webview.replace(webview);
        self
    }

    pub fn geometry(mut self, geometry: Rectangle) -> Self {
        self.geometry.replace(geometry);
        self
    }

    pub fn primary(mut self, is_primary: bool) -> Self {
        self.is_primary = is_primary;
        self
    }

    pub fn debug_mode(mut self, debug_mode: bool) -> Self {
        self.debug_mode = debug_mode;
        self
    }

    #[must_use = "Building the object from the builder is usually expensive and is not expected to have side effects"]
    pub fn build(self) -> Browser {
        if self.window.is_none() || self.webview.is_none() {
            panic!("application and webview should both be set to build a Browser");
        }
        let id = self.id;
        let webview = self.webview.unwrap();
        let window = self.window.unwrap();
        let geometry = self.geometry.unwrap_or_else(|| Rectangle::new(0, 0, 0, 0));
        let debug_mode = self.debug_mode;
        let is_primary = self.is_primary;

        setup_style(&window, geometry, debug_mode);
        setup_actions(&window, &webview, debug_mode);
        window.set_child(Some(&webview));
        Browser {
            webview,
            properties: Rc::new(BrowserProperties {
                id,
                geometry,
                is_primary,
            }),
            loaded: Default::default(),
        }
    }
}

fn setup_style(window: &ApplicationWindow, geometry: Rectangle, debug: bool) {
    window.set_cursor_from_name(Some("defualt"));
    window.set_default_size(geometry.width(), geometry.height());
    window.set_show_menubar(debug);
    window.set_fullscreened(!debug);
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
