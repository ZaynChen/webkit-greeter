lazy_static::lazy_static! {
    pub static ref LOGIN_UID_MINMAX: (u64, u64) = {
        const LOGIN_DEFS_PATHS: [&str; 2] = ["/etc/login.defs", "/usr/local/etc/login.defs"];

        const DEFAULT_UID_MIN: u64 = 1000;
        const DEFAULT_UID_MAX: u64 = 60000;
        match LOGIN_DEFS_PATHS
            .iter()
            .find(|path| std::path::Path::new(path).is_file())
        {
            None => {
                logger::warn!("`login.defs` file not found in these paths: {LOGIN_DEFS_PATHS:?}");
                (DEFAULT_UID_MIN, DEFAULT_UID_MAX)
            }
            Some(path) => std::fs::read_to_string(path)
                .map(|s| {
                    let mut uid_min = None;
                    let mut uid_max = None;
                    for line in s.lines().map(str::trim) {
                        if uid_min.is_none() && line.starts_with("UID_MIN") {
                            uid_min = Some(
                                line.trim_start_matches("UID_MIN")
                                    .trim()
                                    .parse()
                                    .ok()
                                    .unwrap_or(DEFAULT_UID_MIN),
                            );
                        } else if uid_max.is_none() && line.starts_with("UID_MAX") {
                            uid_max = Some(
                                line.trim_start_matches("UID_MAX")
                                    .trim()
                                    .parse()
                                    .ok()
                                    .unwrap_or(DEFAULT_UID_MAX),
                            )
                        } else if uid_min.is_some() && uid_max.is_some() {
                            break;
                        }
                    }
                    (
                        uid_min.unwrap_or(DEFAULT_UID_MIN),
                        uid_max.unwrap_or(DEFAULT_UID_MAX),
                    )
                })
                .map_err(|e| {
                    logger::warn!(
                        "Failed to read login.defs from '{path}', using default values: {e}"
                    )
                })
                .unwrap_or((DEFAULT_UID_MIN, DEFAULT_UID_MAX)),
        }
    };
}
