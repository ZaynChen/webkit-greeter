use std::io::Write;

pub use log::*;

pub fn logger_init(level: LevelFilter) {
    env_logger::Builder::new()
        .filter_level(level)
        .format(|buf, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let level = record.level();
            let file = record.file().unwrap();
            let line = record.line().unwrap();
            let args = record.args();
            writeln!(buf, "{timestamp} [ {level} ] {file} {line}: {args}")
        })
        .init()
}
