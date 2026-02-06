// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use greetd_ipc::{AuthMessageType, ErrorType, Request, Response, codec::SyncCodec};
use std::{env, os::unix::net::UnixStream};

enum AuthStatus {
    NotStarted,
    InAuthentication,
    Authenticated,
}

pub type GreetdResult = Result<Response, greetd_ipc::codec::Error>;
type Function2StrArg = Box<dyn Fn(&str, &str)>;
type Function0Arg = Box<dyn Fn()>;

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
    auth_status: AuthStatus,
    /// Callback invoked when greetd has prompt to user
    show_prompt: Vec<Function2StrArg>,
    /// Callback invoked when greetd has (error) message to user
    show_message: Vec<Function2StrArg>,
    /// Callback invoked when AuthStatus switch to Authenticated
    authentication_complete: Vec<Function0Arg>,
}

impl GreetdClient {
    pub fn new() -> Self {
        let socket = match env::var("GREETD_SOCK") {
            Ok(path) => UnixStream::connect(path)
                .inspect_err(|e| logger::error!("Unable to determine socket to greetd server: {e}"))
                .ok(),
            Err(_) => {
                logger::error!("environment variable 'GREETD_SOCK' not found");
                None
            }
        };

        Self {
            socket,
            auth_user: None,
            auth_status: AuthStatus::NotStarted,
            show_prompt: Vec::new(),
            show_message: Vec::new(),
            authentication_complete: Vec::new(),
        }
    }

    pub fn connect_show_prompt<F>(&mut self, f: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        self.show_prompt.push(Box::new(f));
    }

    fn emit_show_prompt(&self, type_: &str, text: &str) {
        self.show_prompt.iter().for_each(|f| f(type_, text));
    }

    pub fn connect_show_message<F>(&mut self, f: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        self.show_message.push(Box::new(f));
    }

    fn emit_show_message(&self, type_: &str, text: &str) {
        self.show_message.iter().for_each(|f| f(type_, text));
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

    fn set_auth_status(&mut self, status: AuthStatus) {
        self.auth_status = status;
        if self.is_authenticated() && !self.authentication_complete.is_empty() {
            self.authentication_complete
                .iter()
                .for_each(|callback| callback());
        }
    }

    pub fn authentication_user(&self) -> Option<&str> {
        self.auth_user.as_deref()
    }

    pub fn in_authentication(&self) -> bool {
        matches!(self.auth_status, AuthStatus::InAuthentication)
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self.auth_status, AuthStatus::Authenticated)
    }

    fn socket(&mut self) -> Result<&mut UnixStream, std::io::Error> {
        self.socket.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "connect to greetd service failed",
        ))
    }

    /// create_session initiates a login attempt for the given user and
    /// returns either a Response::AuthMessage, Response::Success or Response::Failure.
    ///
    /// If an auth message is returned, it should be answered with post_response.
    /// If a success is returned, the session can then be started with start_session
    ///
    /// If a login flow needs to be aborted at any point, call cancel_session.
    /// Note that the session is cancelled automatically on error.
    pub fn create_session(&mut self, username: String) -> GreetdResult {
        logger::debug!("Creating session for user '{username}'");
        let socket = self.socket()?;
        let auth_user = username.clone();
        let request = Request::CreateSession { username };
        request.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| match resp {
            Response::Success => {
                self.auth_user = Some(auth_user);
                self.set_auth_status(AuthStatus::Authenticated);
            }
            Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => {
                self.auth_user = Some(auth_user);
                self.set_auth_status(AuthStatus::InAuthentication);
                let type_ = match auth_message_type {
                    AuthMessageType::Visible => "Visible",
                    AuthMessageType::Secret => "Secret",
                    AuthMessageType::Info => "Info",
                    AuthMessageType::Error => "Error",
                };
                logger::debug!("AuthMessage: ({type_}, {auth_message})");
                self.emit_show_prompt(type_, auth_message);
            }
            Response::Error {
                error_type,
                description,
            } => {
                let type_ = match error_type {
                    ErrorType::AuthError => "AuthError",
                    ErrorType::Error => "Error",
                };
                self.emit_show_message(type_, description);
            }
        })
    }

    /// post_response responds to the last auth message, and returns
    /// either a Response::AuthMessage, Response::Success or Response::Failure.
    ///
    /// If an auth message is returned, it should be answered with post_response.
    /// If a success is returned, the session can then be started with start_session
    pub fn post_response(&mut self, response: Option<String>) -> GreetdResult {
        logger::debug!("Sending response to greetd");
        let socket = self.socket()?;
        let request = Request::PostAuthMessageResponse { response };
        request.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| match resp {
            Response::Success => self.set_auth_status(AuthStatus::Authenticated),
            Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => {
                let type_ = match auth_message_type {
                    AuthMessageType::Visible => "Visible",
                    AuthMessageType::Secret => "Secret",
                    AuthMessageType::Info => "Info",
                    AuthMessageType::Error => "Error",
                };
                logger::info!("[UNEXPECTED]AuthMessage(response): {type_}, {auth_message}");
                self.emit_show_prompt(type_, auth_message);
            }
            Response::Error {
                error_type,
                description,
            } => {
                let type_ = match error_type {
                    ErrorType::AuthError => "AuthError",
                    ErrorType::Error => "Error",
                };
                self.emit_show_message(type_, description);
            }
        })
    }

    /// Start a successfully logged in session. This will fail if the session
    /// has pending messages or has encountered an error.
    ///
    /// When the start_session success, greetd gives the greeter 5 seconds
    /// to prove itself well-behaved. During this time, call create_session,
    /// post_resonse and cancel_session will response error.
    /// After 5 secs, greetd lose patience and shoot it in the back repeatedly.
    pub fn start_session(&mut self, cmd: Vec<String>, env: Vec<String>) -> GreetdResult {
        logger::debug!("Starting session: cmd: {cmd:?}, env: {env:?}");
        let socket = self.socket()?;
        let request = Request::StartSession { cmd, env };
        request.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| match resp {
            Response::Success => {}
            Response::AuthMessage { .. } => {
                unimplemented!("greetd responded with auth request after requesting session start.")
            }
            Response::Error {
                error_type,
                description,
            } => {
                let type_ = match error_type {
                    ErrorType::AuthError => "AuthError",
                    ErrorType::Error => "Error",
                };
                self.emit_show_message(type_, description);
            }
        })
    }

    /// Cancel a session.
    /// This can only be done if the session has not been started,
    ///   after start_session(), this should not be called.
    /// Cancel does not have to be called if an error has been encountered in
    ///   its setup or login flow.
    /// TODO: Deal with the response from greetd
    pub fn cancel_session(&mut self) -> GreetdResult {
        logger::debug!("Cancelling session");
        self.set_auth_status(AuthStatus::NotStarted);
        self.auth_user = None;
        let socket = self.socket()?;
        Request::CancelSession.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| match resp {
            Response::Success => {}
            Response::AuthMessage { .. } => {
                unimplemented!(
                    "greetd resonded with auth request after requesting session cancellation."
                )
            }
            Response::Error {
                error_type,
                description,
            } => {
                let type_ = match error_type {
                    ErrorType::AuthError => "AuthError",
                    ErrorType::Error => "Error",
                };
                self.emit_show_message(type_, description);
            }
        })
    }
}
