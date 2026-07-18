use error_stack::{Result, ResultExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs::File, io::Write, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to load file")]
pub struct LoadFileError;

#[derive(Debug, Error)]
#[error("Failed to save file")]
pub struct SaveFileError;

pub(super) trait StorageTrait<'a>
where
    Self: DeserializeOwned + Serialize + Sized,
    for<'de> Self: Deserialize<'de> + 'a,
{
    fn load(path: &Path) -> Result<Self, LoadFileError> {
        let file = std::fs::File::open(path).change_context(LoadFileError)?;

        let reader = std::io::BufReader::new(file);

        serde_json::de::from_reader(reader).change_context(LoadFileError)
    }

    fn save(&self, path: &Path) -> Result<(), SaveFileError> {
        let mut file = File::create(path).change_context(SaveFileError)?;

        let stringified_json = serde_json::to_string(&self).change_context(SaveFileError)?;

        file.write_all(stringified_json.as_bytes()).change_context(SaveFileError)?;

        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn get_windows_accent_color() -> Option<[u8; 3]> {
    use std::os::windows::process::CommandExt;
    
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\DWM').ColorizationColor"
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(line) = stdout.lines().next() {
        if let Ok(color_val) = line.trim().parse::<u32>() {
            let r = ((color_val >> 16) & 0xFF) as u8;
            let g = ((color_val >> 8) & 0xFF) as u8;
            let b = (color_val & 0xFF) as u8;
            return Some([r, g, b]);
        }
    }
    None
}
