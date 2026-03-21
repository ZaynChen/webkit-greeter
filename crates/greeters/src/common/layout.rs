// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use serde::Serialize;

use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize)]
pub struct Layout {
    name: String,
    short_description: Option<String>,
    description: String,
}

impl Layout {
    pub fn new(name: String, short_description: Option<String>, description: String) -> Self {
        Self {
            name,
            short_description,
            description,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn short_description(&self) -> Option<&str> {
        self.short_description.as_deref()
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

pub struct LayoutManager {
    layouts: Vec<Layout>,
}

impl LayoutManager {
    pub fn instance() -> &'static LayoutManager {
        static MANAGER: OnceLock<LayoutManager> = OnceLock::new();
        MANAGER.get_or_init(|| Self {
            layouts: keyboard_layouts(),
        })
    }

    /// Get keyboard layouts
    pub fn layouts(&self) -> &[Layout] {
        &self.layouts
    }

    /// Get current keyboard layouts
    pub fn layout(&self) -> &Layout {
        keyboard_layout(&self.layouts)
    }

    /// Set keyboard layout
    pub fn set_layout(&self, layout: &str) -> bool {
        switch_keyboard_layout(layout, &self.layouts)
    }
}

fn keyboard_layouts() -> Vec<Layout> {
    match std::env::var("XDG_SESSION_TYPE").as_deref() {
        Ok("wayland") => match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
            Ok("Hyprland") => hyprland::xkb_layouts(),
            Ok("sway") => sway::xkb_layouts(),
            Ok(s) => {
                log::warn!("WebKit Greeter does not support keyboard layouts of {s} yet");
                vec![]
            }
            Err(e) => {
                log::error!("Could not get $XDG_CURRENT_DESKTOP environment variable: {e}");
                vec![]
            }
        },
        Ok("x11") => system_layouts(),
        _ => {
            log::error!("Could not get $XDG_SESSION_TYPE environment variable");
            vec![]
        }
    }
}

fn keyboard_layout(layouts: &[Layout]) -> &Layout {
    match std::env::var("XDG_SESSION_TYPE").as_deref() {
        Ok("wayland") => match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
            Ok("Hyprland") => hyprland::xkb_layout(layouts),
            Ok("sway") => sway::xkb_layout(layouts),
            Ok(s) => {
                log::warn!("WebKit Greeter does not support keyboard layout of {s} yet");
                layouts.first().unwrap()
            }
            Err(e) => {
                log::error!("Could not get $XDG_CURRENT_DESKTOP environment variable: {e}");
                layouts.first().unwrap()
            }
        },
        Ok("x11") => x11::xkb_layout(layouts),
        _ => {
            log::error!("Could not get $XDG_SESSION_TYPE environment variable");
            layouts.first().unwrap()
        }
    }
}

fn switch_keyboard_layout(layout: &str, layouts: &[Layout]) -> bool {
    let idx = layouts
        .iter()
        .enumerate()
        .find(|(_, l)| l.name() == layout)
        .map(|(i, _)| i)
        .unwrap();
    match std::env::var("XDG_SESSION_TYPE").as_deref() {
        Ok("wayland") => match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
            Ok("Hyprland") => hyprland::switch_xkb_layout(idx as u8)
                .inspect_err(|e| log::error!("Failed to set keyboard layout to {layout}: {e}"))
                .is_ok(),
            Ok("sway") => hyprland::switch_xkb_layout(idx as u8)
                .inspect_err(|e| log::error!("Failed to set keyboard layout to {layout}: {e}"))
                .is_ok(),
            Ok(s) => {
                log::warn!("WebKit Greeter does not support switching keyboard layout of {s} yet");
                false
            }
            Err(e) => {
                log::error!("Could not get $XDG_CURRENT_DESKTOP environment variable: {e}");
                false
            }
        },
        Ok("x11") => x11::switch_xkb_layout(&layouts[idx]),
        _ => {
            log::error!("Could not get $XDG_SESSION_TYPE environment variable");
            false
        }
    }
}

fn system_layouts() -> Vec<Layout> {
    match xkb_data::keyboard_layouts() {
        Ok(layout_list) => layout_list
            .layouts()
            .iter()
            .flat_map(|l| {
                let layout = &l.config_item;
                let mut layouts = vec![Layout::new(
                    layout.name.clone(),
                    layout.short_description.clone(),
                    layout.description.clone(),
                )];
                if let Some(v) = l.variants() {
                    layouts.extend(
                        v.iter()
                            .map(|v| {
                                Layout::new(
                                    [&layout.name, v.name()].join("@"),
                                    layout.short_description.clone(),
                                    layout.description.clone(),
                                )
                            })
                            .collect::<Vec<Layout>>(),
                    );
                }
                layouts
            })
            .collect(),
        Err(e) => {
            log::error!("Failed to get all keyborad layouts: {e}");
            Vec::new()
        }
    }
}

mod hyprland {
    use hyprland::shared::HyprData;

    use super::{Layout, system_layouts};

    pub(super) fn xkb_layouts() -> Vec<Layout> {
        let sys_layouts = system_layouts();
        let keyboard = hyprland::data::Devices::get()
            .expect("Failed to get hyprland devices")
            .keyboards
            .into_iter()
            .find(|kb| kb.main)
            .expect("No main keyboard found");
        let layout_names = keyboard.layout;
        sys_layouts
            .into_iter()
            .filter(|l| layout_names.split(',').any(|name| name == l.name()))
            .collect()
    }

    pub(super) fn xkb_layout(layouts: &[Layout]) -> &Layout {
        let active_keymap = hyprland::data::Devices::get()
            .expect("Failed to get hyprland devices")
            .keyboards
            .into_iter()
            .find(|kb| kb.main)
            .expect("No main keyboard found")
            .active_keymap;
        layouts
            .iter()
            .find(|l| l.description() == active_keymap)
            .unwrap()
    }

    pub(super) fn switch_xkb_layout(idx: u8) -> Result<(), hyprland::error::HyprError> {
        hyprland::ctl::switch_xkb_layout::call(
            "current",
            hyprland::ctl::switch_xkb_layout::SwitchXKBLayoutCmdTypes::Id(idx),
        )
    }
}

mod sway {
    use super::{Layout, system_layouts};

    pub(super) fn xkb_layouts() -> Vec<Layout> {
        let sys_layouts = system_layouts();
        let keyboard = swayipc::Connection::new()
            .expect("Failed to connect to sway")
            .get_inputs()
            .expect("Failed to get inputs")
            .into_iter()
            .find(|input| input.input_type == "keyboard")
            .expect("No main keyboard found");
        sys_layouts
            .into_iter()
            .filter(|l| {
                keyboard
                    .xkb_layout_names
                    .contains(&l.description().to_string())
            })
            .collect()
    }

    pub(super) fn xkb_layout(layouts: &[Layout]) -> &Layout {
        let xkb_active_layout_name = swayipc::Connection::new()
            .expect("Failed to connect to sway")
            .get_inputs()
            .expect("Failed to get inputs")
            .into_iter()
            .find(|input| input.input_type == "keyboard")
            .expect("No main keyboard found")
            .xkb_active_layout_name
            .unwrap();
        layouts
            .iter()
            .find(|l| l.description() == xkb_active_layout_name)
            .unwrap()
    }

    pub(super) fn switch_xkb_layout(idx: u8) -> Result<(), swayipc::Error> {
        swayipc::Connection::new()
            .expect("Failed to connect to sway")
            .run_command(format!("input type:keyboard xkb_switch_layout {idx}"))
            .expect("Failed to run sway command")
            .into_iter()
            .next()
            .unwrap()
    }
}

mod x11 {
    use x11rb::{
        connection::Connection,
        protocol::xproto::{AtomEnum, ConnectionExt, PropMode, change_property, get_property},
    };

    use super::Layout;

    const LAYOUT_VARIANT_SEP: &str = "@";

    fn xkb_rmlvo() -> Vec<String> {
        if let Ok((conn, screen_id)) = x11rb::connect(None)
            && let Ok(request) = conn.intern_atom(false, b"_XKB_RULES_NAMES")
            && let Ok(atom_reply) = request.reply()
        {
            let reply = get_property(
                &conn,
                false,
                conn.setup().roots[screen_id].root,
                atom_reply.atom,
                AtomEnum::STRING,
                0,
                1024,
            )
            .unwrap()
            .reply()
            .unwrap();
            let prop = String::from_utf8_lossy(&reply.value);
            let rmlvo: Vec<_> = prop.split_terminator('\0').map(str::to_string).collect();
            if rmlvo.len() == 5 {
                return rmlvo;
            } else {
                log::error!("Target window property is not an valid rmlvo: {rmlvo:?}");
            }
        }

        log::error!("Failed to get root_window keyboard layout");
        vec!["".to_string(); 5]
    }

    pub(super) fn xkb_layout(layouts: &[Layout]) -> &Layout {
        if let Some(lv) = xkb_rmlvo().get(2..4) {
            let layout = if lv.iter().any(|s| s.is_empty()) {
                lv.first().unwrap().to_string()
            } else {
                lv.join(LAYOUT_VARIANT_SEP)
            };
            if let Some(l) = layouts.iter().find(|l| l.name == layout) {
                return l;
            };
        }

        layouts.first().unwrap()
    }

    pub(super) fn switch_xkb_layout(layout: &Layout) -> bool {
        let old = xkb_rmlvo();
        let rmlvo = if old.iter().all(String::is_empty) {
            log::error!("old keyboard layout is not an valid rmlvo: {old:?}");
            return false;
        } else {
            let lv: Vec<_> = layout.name().split(LAYOUT_VARIANT_SEP).collect();
            let (l, v) = if lv.len() == 2 {
                (lv[0], lv[1])
            } else {
                (lv[0], "")
            };
            [&old[0], &old[1], l, v, &old[4], ""].join("\0")
        };
        if let Ok((conn, screen_id)) = x11rb::connect(None)
            && let Ok(request) = conn.intern_atom(false, b"_XKB_RULES_NAMES")
            && let Ok(atom_reply) = request.reply()
        {
            change_property(
                &conn,
                PropMode::REPLACE,
                conn.setup().roots[screen_id].root,
                atom_reply.atom,
                AtomEnum::STRING,
                8,
                rmlvo.len() as _,
                rmlvo.as_bytes(),
            )
            .is_ok_and(|reply| reply.check().is_ok())
        } else {
            false
        }
    }
}
