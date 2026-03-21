// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use greetd_ipc::{AuthMessageType, Request, Response, codec::SyncCodec};

use std::{env, os::unix::net::UnixStream};

use super::signals::{MessageType, PromptType};
use crate::greeters::GreeterError;

impl From<greetd_ipc::codec::Error> for GreeterError {
    fn from(value: greetd_ipc::codec::Error) -> Self {
        Self::Ipc(value.to_string())
    }
}

#[derive(Clone)]
enum AuthState {
    NotStarted,
    InAuthentication,
    Authenticated,
}

type ShowPromptFun = Box<dyn Fn(&str, PromptType)>;
type ShowMessageFunc = Box<dyn Fn(&str, MessageType)>;

/// Greetd client for communicating with greetd service
pub struct GreetdClient {
    /// Greetd socket for communicating with greetd service
    socket: Option<UnixStream>,
    /// Current user in authentication
    auth_user: Option<String>,
    /// Authentication status for login flow
    ///
    /// NotStarted -> CreateSession -> InAuthentication
    ///                             -> Authenticated
    ///            -> CancelSession -> NotStarted
    ///            -> PostResponse  -> ERROR
    ///            -> StartSession  -> ERROR
    /// InAuthentication -> PostResponse  -> InAuthentication
    ///                                   -> Authenticated
    ///                  -> CancelSession -> NotStarted
    ///                  -> CreateSession -> ERROR
    ///                  -> StartSession  -> ERROR
    /// Authenticated -> CancelSession -> NotStarted
    ///               -> StartSession  -> 5 secs for remaining successed login flow
    ///               -> CreateSession -> ERROR
    ///               -> PostResponse  -> ERROR
    auth_state: AuthState,
    /// Callback invoked when greetd has prompt to user
    show_prompt: Vec<ShowPromptFun>,
    /// Callback invoked when greetd has (error) message to user
    show_message: Vec<ShowMessageFunc>,
    /// Callback invoked when AuthStatus switch to Authenticated
    authentication_complete: Vec<Box<dyn Fn()>>,
}

impl GreetdClient {
    pub fn new() -> Self {
        Self {
            socket: None,
            auth_user: None,
            auth_state: AuthState::NotStarted,
            show_prompt: Vec::new(),
            show_message: Vec::new(),
            authentication_complete: Vec::new(),
        }
    }

    pub fn connect_to_daemon(&mut self) -> Result<(), std::io::Error> {
        match env::var("GREETD_SOCK") {
            Ok(path) => match UnixStream::connect(path) {
                Ok(sock) => {
                    self.socket = Some(sock);
                    Ok(())
                }
                Err(e) => Err(std::io::Error::new(
                    e.kind(),
                    "Unable to determine socket to greetd server: {e}",
                )),
            },
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("environment variable 'GREETD_SOCK' not found: {e}"),
            )),
        }
    }

    pub fn connect_show_prompt<F>(&mut self, f: F)
    where
        F: Fn(&str, PromptType) + 'static,
    {
        self.show_prompt.push(Box::new(f));
    }

    fn emit_show_prompt(&self, text: &str, type_: PromptType) {
        self.show_prompt.iter().for_each(|f| f(text, type_));
    }

    pub fn connect_show_message<F>(&mut self, f: F)
    where
        F: Fn(&str, MessageType) + 'static,
    {
        self.show_message.push(Box::new(f));
    }

    fn emit_show_message(&self, text: &str, type_: MessageType) {
        self.show_message.iter().for_each(|f| f(text, type_));
    }

    pub fn connect_authentication_complete<F>(&mut self, f: F)
    where
        F: Fn() + 'static,
    {
        self.authentication_complete.push(Box::new(f));
    }

    fn emit_authentication_complete(&self) {
        self.authentication_complete.iter().for_each(|f| f())
    }

    fn set_auth_state(&mut self, status: AuthState) {
        self.auth_state = status;
        if self.is_authenticated() && !self.authentication_complete.is_empty() {
            self.emit_authentication_complete()
        }
    }

    pub fn authentication_user(&self) -> Option<&str> {
        self.auth_user.as_deref()
    }

    pub fn in_authentication(&self) -> bool {
        matches!(self.auth_state, AuthState::InAuthentication)
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self.auth_state, AuthState::Authenticated)
    }

    fn socket(&mut self) -> Result<&mut UnixStream, std::io::Error> {
        self.socket.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "connect to greetd service failed",
        ))
    }

    /// return false if response is Response::Error
    fn handle_greetd_response(&mut self, response: Response) -> bool {
        match response {
            Response::Success => self.set_auth_state(AuthState::Authenticated),
            Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => {
                self.set_auth_state(AuthState::InAuthentication);
                log::debug!("AuthMessage: {auth_message_type:?}, {auth_message}");
                match auth_message_type {
                    AuthMessageType::Visible => {
                        self.emit_show_prompt(&auth_message, PromptType::Visible)
                    }
                    AuthMessageType::Secret => {
                        self.emit_show_prompt(&auth_message, PromptType::Secret)
                    }
                    AuthMessageType::Info => {
                        self.emit_show_message(&auth_message, MessageType::Info)
                    }
                    AuthMessageType::Error => {
                        self.emit_show_message(&auth_message, MessageType::Error)
                    }
                }
            }
            Response::Error {
                error_type,
                description,
            } => {
                log::error!("Greetd response error: {description}, {error_type:?}");
                self.emit_show_message(&description, MessageType::Error);
                return false;
            }
        }
        true
    }

    /// create_session initiates a login attempt for the given user and
    /// returns either a Response::AuthMessage, Response::Success or Response::Failure.
    ///
    /// If an auth message is returned, it should be answered with post_response.
    /// If a success is returned, the session can then be started with start_session
    ///
    /// If a login flow needs to be aborted at any point, call cancel_session.
    /// Note that the session is cancelled automatically on error.
    pub fn create_session(&mut self, username: String) -> Result<(), GreeterError> {
        log::debug!("Creating session for user '{username}'");
        if !matches!(self.auth_state, AuthState::NotStarted) {
            let description = "a session is already in authentication";
            self.emit_show_message(description, MessageType::Info);
            return Err(GreeterError::State(description.to_string()));
        }
        let auth_user = Some(username.clone());
        let response = {
            let socket = self.socket()?;
            Request::CreateSession { username }.write_to(socket)?;
            Response::read_from(socket)?
        };
        if self.handle_greetd_response(response) {
            self.auth_user = auth_user;
            Ok(())
        } else {
            self.cancel_session()
        }
    }

    /// post_response responds to the last auth message, and returns
    /// either a Response::AuthMessage, Response::Success or Response::Failure.
    ///
    /// If an auth message is returned, it should be answered with post_response.
    /// If a success is returned, the session can then be started with start_session
    pub fn post_response(&mut self, response: Option<String>) -> Result<(), GreeterError> {
        log::debug!("Sending response to greetd");
        match self.auth_state.clone() {
            AuthState::NotStarted => {
                let description = "no session under authentication";
                self.emit_show_message(description, MessageType::Info);
                return Err(GreeterError::State(description.to_string()));
            }
            AuthState::Authenticated => {
                let description = "session is already authenticated".to_string();
                self.emit_show_message(&description, MessageType::Info);
                return Err(GreeterError::State(description.to_string()));
            }
            _ => {}
        }
        let response = {
            let socket = self.socket()?;
            Request::PostAuthMessageResponse { response }.write_to(socket)?;
            Response::read_from(socket)?
        };
        if self.handle_greetd_response(response) {
            Ok(())
        } else {
            self.cancel_session()
        }
    }

    /// Start a successfully logged in session. This will fail if the session
    /// has pending messages or has encountered an error.
    ///
    /// When the start_session success, greetd gives the greeter 5 seconds
    /// to prove itself well-behaved. During this time, call create_session,
    /// post_resonse and cancel_session will response error.
    /// After 5 secs, greetd lose patience and shoot it in the back repeatedly.
    pub fn start_session(
        &mut self,
        cmd: Vec<String>,
        env: Vec<String>,
    ) -> Result<(), GreeterError> {
        log::debug!("Starting session: cmd: {cmd:?}, env: {env:?}");
        if !self.is_authenticated() {
            let description = "session is not ready";
            self.emit_show_message(description, MessageType::Info);
            return Err(GreeterError::State(description.to_string()));
        }
        let response = {
            let socket = self.socket()?;
            Request::StartSession { cmd, env }.write_to(socket)?;
            Response::read_from(socket)?
        };
        if !self.handle_greetd_response(response) {
            self.auth_user = None;
            self.set_auth_state(AuthState::NotStarted);
        }
        Ok(())
    }

    /// Cancel a session.
    /// This can only be done if the session has not been started,
    ///   after start_session(), this should not be called.
    /// Cancel does not have to be called if an error has been encountered in
    ///   its setup or login flow.
    pub fn cancel_session(&mut self) -> Result<(), GreeterError> {
        log::debug!("Cancelling session");
        self.auth_user = None;
        self.set_auth_state(AuthState::NotStarted);
        let socket = self.socket()?;
        Request::CancelSession.write_to(socket)?;
        if let Response::AuthMessage { .. } = Response::read_from(socket)? {
            unimplemented!(
                "greetd responded with auth request after requesting session cancellation."
            );
        }
        Ok(())
    }
}
