// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

mod accountsservice;
mod logind;

pub use accountsservice::AccountsService;
pub use logind::LogindManager;

use zbus::Connection;

use std::sync::OnceLock;

fn system_conn() -> &'static Connection {
    static SYSTEM_CONN: OnceLock<Connection> = OnceLock::new();
    SYSTEM_CONN.get_or_init(|| zbus::block_on(Connection::system()).unwrap())
}

// fn session_conn() -> &'static Connection {
//     static SESSION_CONN: OnceLock<Connection> = OnceLock::new();
//     SESSION_CONN.get_or_init(|| Connection::session().unwrap())
// }
