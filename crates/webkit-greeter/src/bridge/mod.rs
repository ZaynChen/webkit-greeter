// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod greeter_comm;
mod greeter_config;
mod theme_utils;

pub use dispatcher::Dispatcher;

mod dispatcher {
    use greeters::Greeter;
    use webkit::{UserMessage, WebView, gtk::glib::VariantTy};

    use crate::config::Config;

    use super::{
        greeter_comm::GreeterComm, greeter_config::GreeterConfig, theme_utils::ThemeUtils,
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
        ) -> Self {
            let allowed_dirs = [
                config.themes_dir().to_string(),
                config.background_images_dir().to_string(),
            ];
            Self {
                theme_utils: ThemeUtils::new(context.clone(), &allowed_dirs, config.theme()),
                greeter: Greeter::new(context.clone(), &primary),
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
                    logger::debug!("greeter.{method}({json_params})");
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
        let msg_param = message.parameters();
        if msg_param.is_none() {
            return Message::Unknown;
        }

        let msg_param = msg_param.unwrap();
        if msg_param.is_type(VariantTy::ARRAY) {
            let p_len = msg_param.n_children();
            if p_len == 0 || p_len > 2 {
                return Message::Unknown;
            }
        } else {
            return Message::Unknown;
        }

        let method_var = msg_param.child_value(0);
        let params_var = msg_param.child_value(1);

        let method = method_var.str().unwrap().to_string();
        let json_params = params_var.str().unwrap().to_string();

        if method.is_empty() {
            return Message::Unknown;
        }

        match message.name().as_deref() {
            Some("greeter") => Message::Greeter((method, json_params)),
            Some("greeter_config") => Message::GreeterConfig((method, json_params)),
            Some("greeter_comm") => Message::GreeterComm((method, json_params)),
            Some("theme_utils") => Message::ThemeUtils((method, json_params)),
            _ => Message::Unknown,
        }
    }
}
