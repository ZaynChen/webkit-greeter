// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

mod greetd;
mod lightdm;
mod signals;

pub use greetd::GreetdGreeter;
pub use lightdm::LightDMGreeter;

use crate::common::{LanguageManager, LayoutManager, PowerManager, SessionManager, UserManager};

use webkit::glib::variant::ToVariant;

use webkit::glib::Variant;

use thiserror::Error as ThisError;
#[derive(Debug, ThisError)]
pub enum GreeterError {
    #[error("ipc error: {0}")]
    Ipc(String),
    #[error("i/o error: {0}")]
    Io(String),
    #[error("session state error: {0}")]
    State(String),
    #[error("unknown error: {0}")]
    __Unknown(String),
}

impl From<std::io::Error> for GreeterError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

pub struct Greeter {
    display_manager: String,
    greetd: Option<GreetdGreeter>,
    lightdm: Option<LightDMGreeter>,
}

impl Greeter {
    pub fn new(webview: &webkit::WebView, display_manager: &str) -> Self {
        match display_manager {
            "greetd" => Self {
                display_manager: display_manager.to_string(),
                greetd: Some(GreetdGreeter::new(webview)),
                lightdm: None,
            },
            "lightdm" => Self {
                display_manager: display_manager.to_string(),
                greetd: None,
                lightdm: Some(LightDMGreeter::new(webview)),
            },
            dm => unimplemented!("Unsupported display manager: {dm}"),
        }
    }

    pub fn handle(&self, method: &str, json_args: &str) -> Variant {
        let val: serde_json::Value = serde_json::from_str(json_args).unwrap();
        let args = val.as_array().expect("json_args should be array");
        let json_result = if args.is_empty() {
            match method {
                "can_hibernate" => self.can_hibernate(),
                "can_restart" => self.can_reboot(),
                "can_shutdown" => self.can_shutdown(),
                "can_suspend" => self.can_suspend(),
                "hibernate" => self.hibernate(),
                "restart" => self.reboot(),
                "shutdown" => self.shutdown(),
                "suspend" => self.suspend(),
                "language" => self.language(),
                "languages" => self.languages(),
                "sessions" => self.sessions(),
                "layout" => self.layout(),
                "layouts" => self.layouts(),
                "users" => self.users(),
                m => match self.display_manager.as_str() {
                    "lightdm" => self.lightdm.as_ref().unwrap().handle(m, &[]),
                    "greetd" => self.greetd.as_ref().unwrap().handle(m, &[]),
                    dm => unimplemented!("Unsupported display manager: {dm}"),
                },
            }
        } else {
            match method {
                "layout" => self.set_layout(args[0].as_str().unwrap()),
                m => match self.display_manager.as_str() {
                    "lightdm" => self.lightdm.as_ref().unwrap().handle(m, args),
                    "greetd" => self.greetd.as_ref().unwrap().handle(m, args),
                    dm => unimplemented!("Unsupported display manager: {dm}"),
                },
            }
        };
        json_result.to_variant()
    }

    fn can_hibernate(&self) -> String {
        PowerManager::can_hibernate().to_string()
    }

    fn can_reboot(&self) -> String {
        PowerManager::can_reboot().to_string()
    }

    fn can_shutdown(&self) -> String {
        PowerManager::can_power_off().to_string()
    }

    fn can_suspend(&self) -> String {
        PowerManager::can_suspend().to_string()
    }

    fn hibernate(&self) -> String {
        PowerManager::hibernate()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn reboot(&self) -> String {
        PowerManager::reboot()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn shutdown(&self) -> String {
        PowerManager::power_off()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn suspend(&self) -> String {
        PowerManager::suspend()
            .inspect_err(|e| log::error!("{e}"))
            .is_ok()
            .to_string()
    }

    fn languages(&self) -> String {
        serde_json::to_string(LanguageManager::languages()).unwrap()
    }

    fn language(&self) -> String {
        match LanguageManager::current() {
            Some(language) => serde_json::to_string(&language).unwrap(),
            None => match LanguageManager::languages().first() {
                Some(language) => serde_json::to_string(&language).unwrap(),
                None => "null".to_string(),
            },
        }
    }

    fn layout(&self) -> String {
        serde_json::to_string(LayoutManager::instance().layout()).unwrap()
    }

    fn set_layout(&self, layout: &str) -> String {
        LayoutManager::instance().set_layout(layout).to_string()
    }

    fn layouts(&self) -> String {
        serde_json::to_string(LayoutManager::instance().layouts()).unwrap()
    }

    fn sessions(&self) -> String {
        serde_json::to_string(&SessionManager::sessions()).unwrap()
    }

    fn users(&self) -> String {
        serde_json::to_string(UserManager::instance().list_users()).unwrap()
    }
}
