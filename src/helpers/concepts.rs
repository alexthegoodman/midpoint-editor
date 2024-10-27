use std::{fs, path::Path};

use base64::decode;
use directories::BaseDirs;

use super::utilities::get_common_os_dir;

fn save_concept(
    // state: tauri::State<'_, AppState>,
    projectId: String,
    conceptBase64: String,
    conceptFilename: String,
) -> String {
    // let handle = &state.handle;
    // let config = handle.config();
    // let package_info = handle.package_info();
    // let env = handle.env();

    // let sync_dir = PathBuf::from("C:/Users/alext/CommonOSFiles");
    let sync_dir = get_common_os_dir().expect("Couldn't get CommonOS directory");
    let concepts_dir = sync_dir.join(format!("midpoint/projects/{}/concepts", projectId));

    // Check if the concepts directory exists, create if it doesn't
    if !Path::new(&concepts_dir).exists() {
        fs::create_dir_all(&concepts_dir).expect("Couldn't create concepts directory");
    }

    let concept_path = concepts_dir.join(conceptFilename);

    // Strip the "data:image/png;base64," prefix
    let base64_data = conceptBase64
        .strip_prefix("data:image/png;base64,")
        .ok_or("Invalid base64 image string")
        .expect("Couldn't get base64 string");

    // Decode the base64 string
    let image_data = decode(base64_data)
        .map_err(|e| format!("Couldn't decode base64 string: {}", e))
        .expect("Couldn't decode base64 string");

    // Save the decoded image data to a file
    fs::write(concept_path, image_data)
        .map_err(|e| format!("Couldn't save concept file: {}", e))
        .expect("Couldn't save concept file");

    "success".to_string()
}
