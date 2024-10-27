#[tauri::command]
fn save_model(
    state: tauri::State<'_, AppState>,
    projectId: String,
    modelBase64: String,
    modelFilename: String,
) -> String {
    let handle = &state.handle;
    let config = handle.config();
    let package_info = handle.package_info();
    let env = handle.env();

    let sync_dir = PathBuf::from("C:/Users/alext/CommonOSFiles");
    let models_dir = sync_dir.join(format!("midpoint/projects/{}/models", projectId));

    // Check if the concepts directory exists, create if it doesn't
    if !Path::new(&models_dir).exists() {
        fs::create_dir_all(&models_dir).expect("Couldn't create models directory");
    }

    let model_path = models_dir.join(modelFilename);

    // Strip the "data:image/png;base64," prefix
    let base64_data = modelBase64
        .strip_prefix("data:model/gltf-binary;base64,")
        .ok_or("Invalid base64 model string")
        .expect("Couldn't get base64 string for model");

    // Decode the base64 string
    let model_data = decode(base64_data)
        .map_err(|e| format!("Couldn't decode base64 string for model: {}", e))
        .expect("Couldn't decode base64 string for model");

    // Save the decoded image data to a file
    fs::write(model_path, model_data)
        .map_err(|e| format!("Couldn't save model file: {}", e))
        .expect("Couldn't save model file");

    "success".to_string()
}

#[tauri::command]
async fn read_model(
    state: tauri::State<'_, AppState>,
    projectId: String,
    modelFilename: String,
) -> Result<Vec<u8>, String> {
    let handle = &state.handle;
    let config = handle.config();
    let package_info = handle.package_info();
    let env = handle.env();

    let sync_dir = PathBuf::from("C:/Users/alext/CommonOSFiles");
    let model_path = sync_dir.join(format!(
        "midpoint/projects/{}/models/{}",
        projectId, modelFilename
    ));

    let mut file = File::open(&model_path).map_err(|e| format!("Failed to open model: {}", e))?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read model: {}", e))?;

    Ok(bytes)
}
