use std::{convert::TryInto, path::Path};

use crate::{
    enums::{Brightness, Direction, Effects},
    util::StorageTrait,
};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct KeyboardZone {
    pub rgb: [u8; 3],
    pub enabled: bool,
}

impl Default for KeyboardZone {
    fn default() -> Self {
        Self {
            rgb: Default::default(),
            enabled: true,
        }
    }
}

type Zones = [KeyboardZone; 4];

#[derive(Clone, Copy, Debug)]
pub struct PresetPalette {
    pub name: &'static str,
    pub colors: [u8; 12],
}

pub const PRESET_PALETTES: &[PresetPalette] = &[
    PresetPalette {
        name: "Default",
        colors: [255, 0, 0, 255, 185, 0, 179, 181, 237, 76, 0, 255],
    },
    PresetPalette {
        name: "Cyberpunk",
        colors: [255, 0, 128, 140, 0, 255, 0, 220, 255, 255, 0, 200],
    },
    PresetPalette {
        name: "Ice & Fire",
        colors: [0, 150, 255, 0, 220, 255, 255, 120, 0, 255, 30, 0],
    },
    PresetPalette {
        name: "Aurora",
        colors: [0, 255, 150, 0, 200, 255, 120, 0, 255, 255, 0, 120],
    },
    PresetPalette {
        name: "Pure White",
        colors: [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
    },
];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Profile {
    pub name: Option<String>,
    pub rgb_zones: Zones,
    pub effect: Effects,
    pub direction: Direction,
    pub speed: u8,
    pub brightness: Brightness,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: None,
            rgb_zones: arr_to_zones([
                255, 0, 0,      // Zone 1: Red (#FF0000)
                255, 185, 0,    // Zone 2: Amber/Gold (#FFB900)
                179, 181, 237,  // Zone 3: Soft Lavender (#B3B5ED)
                76, 0, 255,     // Zone 4: Neon Violet/Blue (#4C00FF)
            ]),
            effect: Effects::default(),
            direction: Direction::default(),
            speed: 1,
            brightness: Brightness::default(),
        }
    }
}

#[derive(Debug, Error)]
#[error("Could not load profile")]
pub struct LoadProfileError;

#[derive(Debug, Error)]
#[error("Could not save profile")]
pub struct SaveProfileError;

impl Profile {
    pub fn load_profile(path: &Path) -> Result<Self, LoadProfileError> {
        Self::load(path).change_context(LoadProfileError)
    }

    pub fn save_profile(&mut self, path: &Path) -> Result<(), SaveProfileError> {
        if self.name.is_none() {
            self.name = Some("Untitled".to_string());
        }
        self.save(path).change_context(SaveProfileError)
    }

    pub fn rgb_array(&self) -> [u8; 12] {
        self.rgb_zones.map(|zone| if zone.enabled { zone.rgb } else { [0; 3] }).concat().try_into().unwrap()
    }
}

pub fn arr_to_zones(arr: [u8; 12]) -> Zones {
    [
        KeyboardZone {
            rgb: arr[0..3].try_into().unwrap(),
            enabled: true,
        },
        KeyboardZone {
            rgb: arr[3..6].try_into().unwrap(),
            enabled: true,
        },
        KeyboardZone {
            rgb: arr[6..9].try_into().unwrap(),
            enabled: true,
        },
        KeyboardZone {
            rgb: arr[9..12].try_into().unwrap(),
            enabled: true,
        },
    ]
}

impl StorageTrait<'_> for Profile {}
