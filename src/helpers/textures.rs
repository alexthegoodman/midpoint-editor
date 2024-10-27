use std::{fs, path::Path};

use base64::decode;

use super::utilities::get_common_os_dir;

fn save_texture(projectId: String, textureBase64: String, textureFilename: String) -> String {
    let sync_dir = get_common_os_dir().expect("Couldn't get CommonOS directory");

    let textures_dir = sync_dir.join(format!("midpoint/projects/{}/textures", projectId));

    // Check if the concepts directory exists, create if it doesn't
    if !Path::new(&textures_dir).exists() {
        fs::create_dir_all(&textures_dir).expect("Couldn't create textures directory");
    }

    let texture_path = textures_dir.join(textureFilename);

    // Strip the "data:image/png;base64," prefix
    let base64_data = textureBase64
        .strip_prefix("data:image/png;base64,")
        .ok_or("Invalid base64 image string")
        .expect("Couldn't get base64 string");

    // Decode the base64 string
    let image_data = decode(base64_data)
        .map_err(|e| format!("Couldn't decode base64 string: {}", e))
        .expect("Couldn't decode base64 string");

    // Save the decoded image data to a file
    fs::write(texture_path, image_data)
        .map_err(|e| format!("Couldn't save texture file: {}", e))
        .expect("Couldn't save texture file");

    "success".to_string()
}
