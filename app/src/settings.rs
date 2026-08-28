use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::manager::profile::Profile;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub last_profile: Option<Profile>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            start_with_windows: false,
            start_minimized: false,
            last_profile: None,
        }
    }
}

impl Settings {
    pub fn get_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        let base = PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string()));
        #[cfg(not(target_os = "windows"))]
        let base = PathBuf::from(std::env::var("HOME").map(|h| format!("{}/.config", h)).unwrap_or_else(|_| ".".to_string()));
        
        let dir = base.join("legion-kb-rgb");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir.join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&data) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::get_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, data);
        }
    }

    pub fn delete() {
        let path = Self::get_path();
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
}
