// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use std::sync::{OnceLock, RwLock};

#[derive(Debug)]
pub struct Layout {
    name: String,
}

impl Layout {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct LayoutManager {
    active_layout_idx: RwLock<usize>,
    layouts: Vec<Layout>,
}
impl LayoutManager {
    pub fn instance() -> &'static LayoutManager {
        static MANAGER: OnceLock<LayoutManager> = OnceLock::new();
        MANAGER.get_or_init(|| {
            let (active_layout_idx, layouts) = keyboard_layouts();
            Self {
                active_layout_idx: RwLock::new(active_layout_idx),
                layouts,
            }
        })
    }

    /// Get keyboard layouts
    pub fn layouts(&self) -> &[Layout] {
        &self.layouts
    }

    /// Get current keyboard layouts
    pub fn layout(&self) -> &Layout {
        &self.layouts[*self.active_layout_idx.read().unwrap()]
    }

    /// Set keyboard layout
    // HACK: making this more general
    pub fn set_layout(&self, layout: &str) -> bool {
        let layouts = &self.layouts;
        let idx = layouts
            .iter()
            .enumerate()
            .find(|(_, l)| l.name() == layout)
            .map(|(i, _)| i)
            .unwrap_or_default();
        let mut active_idx = self.active_layout_idx.write().unwrap();
        *active_idx = idx;
        switch_xkb_layout(idx, layouts)
    }
}

fn keyboard_layouts() -> (usize, Vec<Layout>) {
    match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
        #[cfg(feature = "hyprland")]
        Ok("Hyprland") => hyprland_kb_layouts(),
        Ok(s) => {
            logger::warn!("WebKit Greeter does not support keyboard layouts of {s} yet");
            (0, Vec::new())
        }
        Err(e) => {
            logger::error!("Could not get $XDG_CURRENT_DESKTOP environment variable: {e}");
            (0, Vec::new())
        }
    }
}

fn switch_xkb_layout(idx: usize, layouts: &[Layout]) -> bool {
    let layout_name = &layouts[idx].name;
    match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
        #[cfg(feature = "hyprland")]
        Ok("Hyprland") => hyprland_switch_xkb_layout(idx as u8)
            .inspect_err(|e| logger::error!("Failed to set keyboard layout to {layout_name}: {e}"))
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

#[cfg(feature = "hyprland")]
fn hyprland_kb_layouts() -> (usize, Vec<Layout>) {
    use hyprland::shared::HyprData;
    let layouts = hyprland::data::Devices::get()
        .expect("Failed to get hyprland devices")
        .keyboards
        .into_iter()
        .find(|kb| kb.main)
        .expect("No main keyboard found")
        .layout
        .split(',')
        .map(|l| Layout::new(l.to_string()))
        .collect();
    (0, layouts)
}

#[cfg(feature = "hyprland")]
fn hyprland_switch_xkb_layout(idx: u8) -> Result<(), hyprland::error::HyprError> {
    hyprland::ctl::switch_xkb_layout::call(
        "current",
        hyprland::ctl::switch_xkb_layout::SwitchXKBLayoutCmdTypes::Id(idx),
    )
}
