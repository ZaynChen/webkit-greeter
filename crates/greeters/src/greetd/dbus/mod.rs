mod accountservice;
mod logind;

pub use accountservice::AccountsService;
pub use logind::LogindManager;

use zbus::blocking::Connection;

use std::sync::OnceLock;

fn system_conn() -> &'static Connection {
    static SYSTEM_CONN: OnceLock<Connection> = OnceLock::new();
    SYSTEM_CONN.get_or_init(|| Connection::system().unwrap())
}

// fn session_conn() -> &'static Connection {
//     static SESSION_CONN: OnceLock<Connection> = OnceLock::new();
//     SESSION_CONN.get_or_init(|| Connection::session().unwrap())
// }
