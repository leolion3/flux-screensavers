use crate::config;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    pub version: semver::Version,
    pub log_level: log::Level,
    pub flux: FluxSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: semver::Version::parse("0.1.0").unwrap(),
            log_level: log::Level::Warn,
            flux: Default::default(),
        }
    }
}

impl config::UpgradableConfig for Config {
    type UpgradedConfig = config::Config;

    fn upgrade(&self) -> Self::UpgradedConfig {
        let color_mode = match self.flux.color_mode {
            ColorMode::Preset(preset) => config::ColorMode::Preset {
                preset_name: preset,
            },
            ColorMode::DesktopImage => config::ColorMode::DesktopImage,
        };

        config::Config {
            version: config::LATEST_VERSION,
            log_level: self.log_level,
            flux: config::FluxSettings { color_mode },
            platform: Default::default(),
            location: None,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct FluxSettings {
    pub color_mode: ColorMode,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum ColorMode {
    Preset(flux::settings::ColorPreset),
    DesktopImage,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Preset(Default::default())
    }
}
