use std::{fs, path::Path};

use base64::decode;
use uuid::Uuid;

use super::utilities::get_common_os_dir;

fn save_landscape(
    // state: tauri::State<'_, AppState>,
    projectId: String,
    landscapeBase64: String,
    landscapeFilename: String,
    rockmapFilename: String,
    rockmapBase64: String,
    soilFilename: String,
    soilBase64: String,
) -> String {
    // let handle = &state.handle;
    // let config = handle.config();
    // let package_info = handle.package_info();
    // let env = handle.env();

    let landscape_id = Uuid::new_v4();

    // let sync_dir = PathBuf::from("C:/Users/alext/CommonOSFiles");
    let sync_dir = get_common_os_dir().expect("Couldn't get CommonOS directory");

    let heightmaps_dir = sync_dir.join(format!(
        "midpoint/projects/{}/landscapes/{}/heightmaps",
        projectId, landscape_id,
    ));
    let rockmaps_dir = sync_dir.join(format!(
        "midpoint/projects/{}/landscapes/{}/rockmaps",
        projectId, landscape_id
    ));
    let soils_dir = sync_dir.join(format!(
        "midpoint/projects/{}/landscapes/{}/soils",
        projectId, landscape_id
    ));

    // Check if the concepts directory exists, create if it doesn't
    if !Path::new(&heightmaps_dir).exists() {
        fs::create_dir_all(&heightmaps_dir).expect("Couldn't create heightmaps directory");
    }
    if !Path::new(&rockmaps_dir).exists() {
        fs::create_dir_all(&rockmaps_dir).expect("Couldn't create rockmaps directory");
    }
    if !Path::new(&soils_dir).exists() {
        fs::create_dir_all(&soils_dir).expect("Couldn't create soils directory");
    }

    let heightmap_path = heightmaps_dir.join(landscapeFilename);
    let rockmap_path = rockmaps_dir.join(rockmapFilename);
    let soil_path = soils_dir.join(soilFilename);

    // prefix is pre-stripped on frontend
    let base64_data = landscapeBase64;
    let heightmap_data = decode(base64_data)
        .map_err(|e| format!("Couldn't decode base64 string for heightmap: {}", e))
        .expect("Couldn't decode base64 string for heightmap");
    fs::write(heightmap_path, heightmap_data)
        .map_err(|e| format!("Couldn't save heightmap file: {}", e))
        .expect("Couldn't save heightmap file");

    let base64_data = rockmapBase64;
    let rockmap_data = decode(base64_data)
        .map_err(|e| format!("Couldn't decode base64 string for rockmap: {}", e))
        .expect("Couldn't decode base64 string for rockmap");
    fs::write(rockmap_path, rockmap_data)
        .map_err(|e| format!("Couldn't save rockmap file: {}", e))
        .expect("Couldn't save rockmap file");

    let base64_data = soilBase64;
    let soil_data = decode(base64_data)
        .map_err(|e| format!("Couldn't decode base64 string for soil: {}", e))
        .expect("Couldn't decode base64 string for soil");
    fs::write(soil_path, soil_data)
        .map_err(|e| format!("Couldn't save soil file: {}", e))
        .expect("Couldn't save soil file");

    "success".to_string()
}
