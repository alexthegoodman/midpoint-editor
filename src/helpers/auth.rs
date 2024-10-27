use directories::ProjectDirs;

pub fn read_token(// state: tauri::State<'_, AppState>
) -> String {
    // let handle = &state.handle;
    // let config = handle.config();
    // let package_info = handle.package_info();
    // let env = handle.env();

    read_auth_token()
}

pub fn read_auth_token(// config: &Arc<tauri::Config>
) -> String {
    println!("read_auth_token");

    // let app_data_path = app_data_dir(config)
    //     .ok_or("Failed to get AppData directory (1)")
    //     .expect("Failed to get AppData directory (2)");
    // let app_data_path = app_data_path
    //     .parent()
    //     .expect("Failed to get AppData directory (3)")
    //     .join("com.common.commonosfiles");

    if let Some(proj_dirs) = ProjectDirs::from("com", "common", "commonosfiles") {
        // App data directory
        let data_dir = proj_dirs.data_dir();

        let read_path = data_dir.join("auth");

        // pull String content from read_path
        let auth_data =
            String::from_utf8_lossy(&std::fs::read(read_path).unwrap_or_default()).to_string();

        auth_data
    } else {
        "".to_string()
    }
}
