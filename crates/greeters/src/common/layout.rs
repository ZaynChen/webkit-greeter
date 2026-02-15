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
    // TODO: can this be empty ??
    pub fn layouts(&self) -> &[Layout] {
        &self.layouts
    }

    /// Get current keyboard layouts
    // TODO: can this be None ??
    pub fn layout(&self) -> &Layout {
        keyboard_layout(&self.layouts)
    }

    /// Set keyboard layout
    pub fn set_layout(&self, layout: &str) -> bool {
        switch_xkb_layout(layout, &self.layouts)
    }
}

fn keyboard_layouts() -> Vec<Layout> {
    match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
        #[cfg(feature = "hyprland")]
        Ok("Hyprland") => hyprland_kb_layouts(),
        #[cfg(feature = "sway")]
        Ok("sway") => sway_kb_layouts(),
        Ok(s) => {
            logger::warn!("WebKit Greeter does not support keyboard layouts of {s} yet");
            Vec::new()
        }
        Err(e) => {
            logger::error!("Could not get $XDG_CURRENT_DESKTOP environment variable: {e}");
            Vec::new()
        }
    }
}

fn keyboard_layout(layouts: &[Layout]) -> &Layout {
    match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
        #[cfg(feature = "hyprland")]
        Ok("Hyprland") => hyprland_kb_layout(layouts),
        #[cfg(feature = "sway")]
        Ok("sway") => sway_kb_layout(layouts),
        Ok(s) => {
            logger::warn!("WebKit Greeter does not support keyboard layout of {s} yet");
            layouts.first().unwrap()
        }
        Err(e) => {
            logger::error!("Could not get $XDG_CURRENT_DESKTOP environment variable: {e}");
            layouts.first().unwrap()
        }
    }
}

fn switch_xkb_layout(layout: &str, layouts: &[Layout]) -> bool {
    let idx = layouts
        .iter()
        .enumerate()
        .find(|(_, l)| l.name() == layout)
        .map(|(i, _)| i)
        .unwrap();
    match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
        #[cfg(feature = "hyprland")]
        Ok("Hyprland") => hyprland_switch_xkb_layout(idx as u8)
            .inspect_err(|e| logger::error!("Failed to set keyboard layout to {layout}: {e}"))
            .is_ok(),
        #[cfg(feature = "sway")]
        Ok("sway") => sway_switch_xkb_layout(idx as u8)
            .inspect_err(|e| logger::error!("Failed to set keyboard layout to {layout}: {e}"))
            .is_ok(),
        Ok(s) => {
            logger::warn!("WebKit Greeter does not support switching keyboard layout of {s} yet");
            false
        }
        Err(e) => {
            logger::error!("Could not get $XDG_CURRENT_DESKTOP environment variable: {e}");
            false
        }
    }
}

#[cfg(feature = "xkb")]
fn system_layouts() -> Vec<Layout> {
    match xkb_data::keyboard_layouts() {
        Ok(layout_list) => layout_list
            .layouts()
            .iter()
            .map(|l| {
                let layout = &l.config_item;
                Layout::new(
                    layout.name.clone(),
                    layout.short_description.clone(),
                    layout.description.clone(),
                )
            })
            .collect(),
        Err(e) => {
            logger::error!("Failed to get all keyborad layouts: {e}");
            Vec::new()
        }
    }
}

#[cfg(feature = "hyprland")]
use hyprland::shared::HyprData;

#[cfg(feature = "hyprland")]
fn hyprland_kb_layouts() -> Vec<Layout> {
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

#[cfg(feature = "hyprland")]
fn hyprland_kb_layout(layouts: &[Layout]) -> &Layout {
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

#[cfg(feature = "hyprland")]
fn hyprland_switch_xkb_layout(idx: u8) -> Result<(), hyprland::error::HyprError> {
    hyprland::ctl::switch_xkb_layout::call(
        "current",
        hyprland::ctl::switch_xkb_layout::SwitchXKBLayoutCmdTypes::Id(idx),
    )
}

#[cfg(feature = "sway")]
fn sway_kb_layouts() -> Vec<Layout> {
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

#[cfg(feature = "sway")]
fn sway_kb_layout(layouts: &[Layout]) -> &Layout {
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

#[cfg(feature = "sway")]
fn sway_switch_xkb_layout(idx: u8) -> Result<(), swayipc::Error> {
    swayipc::Connection::new()
        .expect("Failed to connect to sway")
        .run_command(format!("input type:keyboard xkb_switch_layout {idx}"))
        .expect("Failed to run sway command")
        .into_iter()
        .next()
        .unwrap()
}
