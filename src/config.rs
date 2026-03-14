use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub source: String,
    pub dest: String,
    pub action_type: String,
    pub file_filter: String,
    pub thread_count: i32,
    pub retry_count: i32,
    pub wait_time: i32,
    pub copy_empty_dirs: bool,
    pub restartable_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            source: "".to_string(),
            dest: "".to_string(),
            action_type: "Copy (Standard)".to_string(),
            file_filter: "*.*".to_string(),
            thread_count: 8,
            retry_count: 3,
            wait_time: 1,
            copy_empty_dirs: true,
            restartable_mode: true,
        }
    }
}

pub fn save_config(config: &AppConfig, path: &Path) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_config(path: &Path) -> std::io::Result<AppConfig> {
    let json = fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&json)?;
    Ok(config)
}
