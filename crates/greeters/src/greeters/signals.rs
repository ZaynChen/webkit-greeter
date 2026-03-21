// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use webkit::{
    UserMessage, WebView, gio::Cancellable, glib::variant::ToVariant, prelude::WebViewExt,
};

#[derive(Debug, Clone, Copy)]
pub enum PromptType {
    Visible = 0,
    Secret = 1,
}

impl TryFrom<u32> for PromptType {
    type Error = webkit::glib::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Visible),
            1 => Ok(Self::Secret),
            _ => Err(webkit::glib::Error::new(
                webkit::glib::ConvertError::Failed,
                "invalid prompt type",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Info = 0,
    Error = 1,
}

impl TryFrom<u32> for MessageType {
    type Error = webkit::glib::Error;

    fn try_from(value: u32) -> Result<Self, <Self as TryFrom<u32>>::Error> {
        match value {
            0 => Ok(Self::Info),
            1 => Ok(Self::Error),
            _ => Err(webkit::glib::Error::new(
                webkit::glib::ConvertError::Failed,
                "invalid message type",
            )),
        }
    }
}

pub(super) fn show_prompt(webview: &WebView, text: &str, ty: PromptType) {
    let type_ = match ty {
        PromptType::Visible => "Visible",
        PromptType::Secret => "Secret",
    };
    let parameters = ["show_prompt", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
    let message = UserMessage::new("greeter", Some(&parameters));
    webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
}

pub(super) fn show_message(webview: &WebView, text: &str, ty: MessageType) {
    let type_ = match ty {
        MessageType::Info => "Info",
        MessageType::Error => "Error",
    };
    let parameters = ["show_message", &format!("[\"{text}\", \"{type_}\"]")].to_variant();
    let message = UserMessage::new("greeter", Some(&parameters));
    webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
}

pub(super) fn authentication_complete(webview: &WebView) {
    let parameters = ["authentication_complete", "[]"].to_variant();
    let message = UserMessage::new("greeter", Some(&parameters));
    webview.send_message_to_page(&message, Cancellable::NONE, |_| {});
}
