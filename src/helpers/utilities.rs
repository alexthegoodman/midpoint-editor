use std::{fs, path::PathBuf};

use directories::{BaseDirs, UserDirs};

pub fn get_common_os_dir() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| {
        let common_os = user_dirs
            .document_dir()
            .expect("Couldn't find Documents directory")
            .join("CommonOS");
        fs::create_dir_all(&common_os)
            .ok()
            .expect("Couldn't check or create CommonOS directory");
        common_os
    })
}
