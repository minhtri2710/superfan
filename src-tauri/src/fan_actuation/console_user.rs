use std::os::unix::fs::MetadataExt;

pub fn active_console_uid() -> Result<u32, String> {
    // Apple documents /dev/console as owned by the active console user.
    // Source: https://developer.apple.com/library/archive/technotes/tn2083/_index.html
    let uid = std::fs::metadata("/dev/console")
        .map_err(|error| format!("cannot determine active console user: {error}"))?
        .uid();

    if uid == 0 {
        Err("no active console user is available".into())
    } else {
        Ok(uid)
    }
}

#[cfg(test)]
mod tests {
    use super::active_console_uid;

    #[test]
    fn active_console_uid_never_authorizes_root() {
        if let Ok(uid) = active_console_uid() {
            assert_ne!(uid, 0);
        }
    }
}
