// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use greetd_ipc::{ErrorType, Request, Response, codec::SyncCodec};
use std::{env, os::unix::net::UnixStream};

enum AuthStatus {
    /// no session active
    NotStarted,
    /// session has pending questions
    Authenticated,
    /// session is not ready
    InAuthentication,
}

type AuthenticationComplete = Option<Box<dyn Fn(&GreetdClient)>>;
pub type GreetdResult = Result<Response, greetd_ipc::codec::Error>;

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
    /// Callback invoked when AuthStatus switch to Authenticated
    authentication_complete: AuthenticationComplete,
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
            authentication_complete: None,
        }
    }

    pub fn connect_authentication_complete<F>(&mut self, f: F)
    where
        F: Fn(&Self) + 'static,
    {
        self.authentication_complete = Some(Box::new(f));
    }

    fn set_auth_status(&mut self, status: AuthStatus) {
        self.auth_status = status;
        if self.is_authenticated()
            && let Some(callback) = &self.authentication_complete
        {
            callback(self);
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
        if !matches!(self.auth_status, AuthStatus::NotStarted) {
            return Ok(Response::Error {
                error_type: ErrorType::Error,
                description: "a session is already under authentication".into(),
            });
        }

        self.auth_user = Some(username.clone());
        let socket = self.socket()?;
        let request = Request::CreateSession { username };
        request.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| match resp {
            Response::Success => self.set_auth_status(AuthStatus::Authenticated),
            Response::AuthMessage { .. } => self.set_auth_status(AuthStatus::InAuthentication),
            _ => self.auth_user = None,
        })
    }

    /// post_response responds to the last auth message, and returns
    /// either a Response::AuthMessage, Response::Success or Response::Failure.
    ///
    /// If an auth message is returned, it should be answered with post_response.
    /// If a success is returned, the session can then be started with start_session
    pub fn post_response(&mut self, response: Option<String>) -> GreetdResult {
        match self.auth_status {
            AuthStatus::NotStarted => {
                return Ok(Response::Error {
                    error_type: ErrorType::Error,
                    description: "no session under authentication".into(),
                });
            }
            AuthStatus::Authenticated => {
                return Ok(Response::Error {
                    error_type: ErrorType::Error,
                    description: "current session is already authenticated".into(),
                });
            }
            _ => {}
        }

        let socket = self.socket()?;
        let request = Request::PostAuthMessageResponse { response };
        request.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| {
            if let Response::Success = resp {
                self.set_auth_status(AuthStatus::Authenticated);
            }
        })
    }

    /// Start a successfully logged in session. This will fail if the session
    /// has pending messages or has encountered an error.
    pub fn start_session(&mut self, cmd: Vec<String>, env: Vec<String>) -> GreetdResult {
        if !self.is_authenticated() {
            return Ok(Response::Error {
                error_type: ErrorType::AuthError,
                description: "current session is not authenticated".to_string(),
            });
        }

        // When the start_session success, greetd gives the greeter 5 seconds
        // to prove itself well-behaved. During this time, call create_session,
        // post_resonse and cancel_session will response error.
        // After 5 secs, greetd lose patience and shoot it in the back repeatedly.
        let socket = self.socket()?;
        let request = Request::StartSession { cmd, env };
        request.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| {
            if let Response::AuthMessage { .. } = resp {
                unimplemented!(
                    "greetd responded with auth request after requesting session start."
                );
            }
        })
    }

    /// Cancel a session.
    /// This can only be done if the session has not been started.
    pub fn cancel_session(&mut self) -> GreetdResult {
        let socket = self.socket()?;
        Request::CancelSession.write_to(socket)?;
        Response::read_from(socket).inspect(|resp| {
            match resp {
                Response::Success => {
                    self.set_auth_status(AuthStatus::NotStarted);
                    self.auth_user = None;
                }
                Response::AuthMessage { .. } => {
                    unimplemented!(
                        "greetd resonded with auth request after requesting session cancellation."
                    )
                }
                _ => {}
            };
        })
    }
}
