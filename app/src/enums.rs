use crate::manager::{custom_effect::CustomEffect, profile::Profile};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

fn default_sensitivity() -> f32 {
    1.0
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display, EnumIter, Debug, IntoStaticStr, Default)]
pub enum Effects {
    #[default]
    Static,
    Breath,
    Smooth,
    Wave,
    Lightning,
    AmbientLight {
        fps: u8,
        saturation_boost: f32,
        #[serde(default)]
        smoothness: bool,
    },
    SmoothWave {
        mode: SwipeMode,
        clean_with_black: bool,
    },
    Swipe {
        mode: SwipeMode,
        clean_with_black: bool,
    },
    Disco,
    Christmas,
    Fade,
    Temperature {
        use_accent: bool,
        hot_color: [u8; 3],
        cool_color: [u8; 3],
    },
    Ripple,
    SystemLoad,
    AudioVisualizer {
        #[serde(default = "default_sensitivity")]
        sensitivity: f32,
    },
    SunMoon,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, EnumIter, EnumString, PartialEq)]
pub enum SwipeMode {
    #[default]
    Change,
    Fill,
}

impl PartialEq for Effects {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[allow(dead_code)]
impl Effects {
    pub fn takes_color_array(self) -> bool {
        matches!(self, Self::Static | Self::Breath | Self::Lightning | Self::SmoothWave { .. } | Self::Swipe { .. } | Self::Fade | Self::Ripple)
    }

    pub fn takes_direction(self) -> bool {
        matches!(self, Self::Wave | Self::SmoothWave { .. } | Self::Swipe { .. })
    }

    pub fn takes_speed(self) -> bool {
        matches!(
            self,
            Self::Breath | Self::Smooth | Self::Wave | Self::Lightning | Self::SmoothWave { .. } | Self::Swipe { .. } | Self::Disco | Self::Fade | Self::Ripple
        )
    }

    pub fn is_built_in(self) -> bool {
        matches!(self, Self::Static | Self::Breath | Self::Smooth | Self::Wave)
    }

    pub fn with_sensible_defaults(self) -> Self {
        match self {
            Self::AmbientLight { fps, saturation_boost, smoothness } => Self::AmbientLight {
                fps: if fps == 0 { 60 } else { fps.clamp(1, 144) },
                saturation_boost: if saturation_boost == 0.0 { 0.2 } else { saturation_boost.clamp(0.0, 1.0) },
                smoothness: if !smoothness { true } else { smoothness },
            },
            Self::AudioVisualizer { sensitivity } => Self::AudioVisualizer {
                sensitivity: if sensitivity <= 0.0 { 1.0 } else { sensitivity },
            },
            Self::Temperature { use_accent, hot_color, cool_color } => Self::Temperature {
                use_accent,
                hot_color: if hot_color == [0, 0, 0] { [255, 0, 0] } else { hot_color },
                cool_color: if cool_color == [0, 0, 0] { [0, 100, 255] } else { cool_color },
            },
            other => other,
        }
    }
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Debug, EnumIter, IntoStaticStr, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Left,
    Right,
}

#[derive(PartialEq, Eq, EnumIter, IntoStaticStr, Clone, Copy, Default, Serialize, Deserialize, Debug, Display, EnumString)]
pub enum Brightness {
    #[default]
    Low,
    High,
}

#[derive(Debug)]
pub enum Message {
    CustomEffect { effect: CustomEffect },
    Profile { profile: Profile },
    Exit,
}
