use std::path::Path;

pub const LEGACY_HELPER_PATH: &str = "/usr/local/bin/smc-helper";

pub fn remove_legacy_helper() -> Result<(), String> {
    remove_legacy_helper_at(Path::new(LEGACY_HELPER_PATH))
}

fn remove_legacy_helper_at(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => std::fs::remove_file(path)
            .map_err(|error| format!("could not remove legacy setuid helper: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("could not inspect legacy setuid helper: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::remove_legacy_helper_at;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn removes_only_the_named_legacy_path() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("superfan-migration-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let legacy = directory.join("smc-helper");
        let neighbor = directory.join("keep-me");
        std::fs::write(&legacy, b"legacy").unwrap();
        std::fs::write(&neighbor, b"neighbor").unwrap();

        remove_legacy_helper_at(&legacy).unwrap();

        assert!(!legacy.exists());
        assert!(neighbor.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn missing_legacy_path_is_already_migrated() {
        let path = std::env::temp_dir().join("superfan-missing-legacy-helper");
        let _ = std::fs::remove_file(&path);

        remove_legacy_helper_at(&path).unwrap();
    }
}
