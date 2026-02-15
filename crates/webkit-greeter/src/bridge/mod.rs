// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod greeter;
mod greeter_comm;
mod greeter_config;
mod theme_utils;

pub use dispatcher::Dispatcher;

mod dispatcher {
    use webkit::{UserMessage, WebView, gtk::glib::VariantTy};

    use crate::config::Config;

    use super::{
        greeter::Greeter, greeter_comm::GreeterComm, greeter_config::GreeterConfig,
        theme_utils::ThemeUtils,
    };

    pub struct Dispatcher {
        greeter: Greeter,
        greeter_config: GreeterConfig,
        greeter_comm: GreeterComm,
        theme_utils: ThemeUtils,
    }

    impl Dispatcher {
        pub fn new(
            config: Config,
            context: jsc::Context,
            primary: WebView,
            secondaries: Vec<WebView>,
            display_manager: &str,
        ) -> Self {
            let allowed_dirs = [
                config.themes_dir().to_string(),
                config.background_images_dir().to_string(),
            ];
            Self {
                theme_utils: ThemeUtils::new(context.clone(), &allowed_dirs, config.theme()),
                greeter: Greeter::new(context.clone(), &primary, display_manager),
                greeter_config: GreeterConfig::new(context.clone(), config),
                greeter_comm: GreeterComm::new(context.clone(), primary, secondaries),
            }
        }

        pub fn primary(&self) -> &WebView {
            self.greeter_comm.primary()
        }

        pub fn secondaries(&self) -> &[WebView] {
            self.greeter_comm.secondaries()
        }

        pub fn themes_dir(&self) -> String {
            self.greeter_config.themes_dir().to_string()
        }

        pub fn send(&self, message: &UserMessage) {
            let reply = match parse(message) {
                Message::GreeterConfig((method, _)) => {
                    // logger::debug!("greeter_config.{method}({json_params})");
                    let reply = self.greeter_config.handle(&method);
                    UserMessage::new("reply", Some(&reply))
                }
                Message::GreeterComm((method, json_params)) => {
                    // logger::debug!("greeter_comm.{method}({json_params})");
                    let reply = self.greeter_comm.handle(&method, &json_params);
                    UserMessage::new("reply", Some(&reply))
                }
                Message::Greeter((method, json_params)) => {
                    // logger::debug!("greeter.{method}({json_params})");
                    let reply = self.greeter.handle(&method, &json_params);
                    UserMessage::new("reply", Some(&reply))
                }
                Message::ThemeUtils((method, json_params)) => {
                    // logger::debug!("theme_utils.{method}({json_params})");
                    let reply = self.theme_utils.handle(&method, &json_params);
                    UserMessage::new("reply", Some(&reply))
                }
                _ => {
                    logger::warn!("{:?}-{:?}", message.name(), message.parameters());
                    UserMessage::new("", None)
                }
            };
            // logger::warn!("{:?}-{:?}", reply.name(), reply.parameters());
            message.send_reply(&reply);
        }
    }

    enum Message {
        GreeterConfig((String, String)),
        Greeter((String, String)),
        GreeterComm((String, String)),
        ThemeUtils((String, String)),
        Unknown,
    }

    fn parse(message: &UserMessage) -> Message {
        let (method, json_params) = if let Some(msg_param) = message.parameters()
            && msg_param.is_type(VariantTy::ARRAY)
            && msg_param.n_children() == 2
            && let method = msg_param.child_value(0).str()
            && method.is_some_and(|m| !m.is_empty())
        {
            (
                method.unwrap().to_string(),
                msg_param.child_value(1).str().unwrap().to_string(),
            )
        } else {
            return Message::Unknown;
        };

        match message.name().as_deref() {
            Some("greeter") => Message::Greeter((method, json_params)),
            Some("greeter_config") => Message::GreeterConfig((method, json_params)),
            Some("greeter_comm") => Message::GreeterComm((method, json_params)),
            Some("theme_utils") => Message::ThemeUtils((method, json_params)),
            _ => Message::Unknown,
        }
    }
}
