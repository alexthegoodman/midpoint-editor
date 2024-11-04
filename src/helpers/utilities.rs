use std::{fs, path::PathBuf};

use directories::{BaseDirs, UserDirs};
use regex::Regex;
use uuid::Uuid;

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

pub fn get_filename(concept_prompt_str: String) -> String {
    let concept_filename: String = concept_prompt_str.chars().skip(0).take(20).collect();

    let re = Regex::new(r"[^a-zA-Z0-9.]").unwrap();
    let concept_filename = re.replace_all(concept_filename.as_str(), "_").to_string();
    // let concept_filename =
    //     std::str::from_utf8(&concept_filename).expect("Couldn't convert filename");

    let concept_filename = format!("{}-{}", concept_filename, Uuid::new_v4());

    concept_filename
}

use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Command {
    pub command: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "newId")]
    pub new_id: String,
    #[serde(rename = "fileName")]
    pub filename: String,
    #[serde(rename = "cloudfrontUrl")]
    pub cloudfront_url: String,
    #[serde(rename = "normalFilePath")]
    pub normal_file_path: String,
}

pub fn parse_ws_command(json: &str) -> Result<Command, serde_json::Error> {
    match serde_json::from_str::<Command>(json) {
        Ok(cmd) => {
            println!("Successfully parsed command: {:?}", cmd);
            Ok(cmd)
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
            println!("Failed JSON: {}", json);
            Err(e)
        }
    }
}

pub fn change_extension_to_glb(filename: &str) -> String {
    let mut path = PathBuf::from(filename);
    path.set_extension("glb");
    path.to_string_lossy().into_owned()
}

pub fn parse_string_to_float(input: &str) -> Option<f32> {
    // First trim any whitespace
    let trimmed = input.trim();

    // Check if string is empty after trimming
    if trimmed.is_empty() {
        return None;
    }

    // Attempt to parse and handle errors gracefully
    match trimmed.parse::<f32>() {
        Ok(value) => {
            // Check if value is finite (not NaN or infinite)
            if value.is_finite() {
                Some(value)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
