mod v1;

use log::Level;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, fs, io, path};

const LATEST_VERSION: u8 = 2;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub version: u8,
    #[serde(with = "LogLevelDef")]
    pub log_level: log::Level,
    pub flux: FluxSettings,
    pub platform: PlatformConfig,

    // An optional path to the location of this config
    #[serde(skip)]
    location: Option<path::PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(remote = "Level", rename_all = "camelCase")]
enum LogLevelDef {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Latest version of the config
            version: LATEST_VERSION,
            log_level: log::Level::Warn,
            flux: Default::default(),
            platform: Default::default(),
            location: None,
        }
    }
}

impl Config {
    pub fn load(optional_config_dir: Option<&path::Path>) -> Self {
        match optional_config_dir {
            None => Self::default(),

            Some(config_dir) => {
                let config_path = config_dir.join("settings.json");
                let config = Self::load_existing_config(config_path.as_path());
                if let Err(err) = &config {
                    match err {
                        Problem::ReadSettings { err, path }
                            if err.kind() == io::ErrorKind::NotFound =>
                        {
                            log::info!(
                                "No settings file found at {}. Using defaults.",
                                path.display()
                            )
                        }
                        _ => log::error!("{}", err),
                    }
                }

                config.unwrap_or_default().attach_location(&config_path)
            }
        }
    }

    // Attach the config's location
    fn attach_location(mut self, path: &path::Path) -> Self {
        self.location = Some(path.to_owned());

        self
    }

    fn load_existing_config(config_path: &path::Path) -> Result<Self, Problem> {
        let config_string =
            fs::read_to_string(config_path).map_err(|err| Problem::ReadSettings {
                path: config_path.to_owned(),
                err,
            })?;

        Self::from_string(&config_string, Some(config_path))
    }

    fn from_string(config_string: &str, config_path: Option<&path::Path>) -> Result<Self, Problem> {
        let to_decode_error = |err| Problem::DecodeSettings {
            path: config_path
                .unwrap_or_else(|| path::Path::new(""))
                .to_owned(),
            err,
        };

        let config_ast: serde_json::Value =
            serde_json::from_str(config_string).map_err(to_decode_error)?;
        let version: Cow<'_, str> =
            serde_json::from_value(config_ast["version"].clone()).map_err(to_decode_error)?;

        match version.as_ref() {
            "0.1.0" => serde_json::from_value::<v1::Config>(config_ast)
                .map(|config| config.upgrade())
                .map_err(to_decode_error),
            "2" => serde_json::from_value(config_ast).map_err(to_decode_error),
            _ => Err(Problem::UnsupportedVersion {
                version: version.to_string(),
            }),
        }
    }

    pub fn save(&self) -> Result<(), Problem> {
        match &self.location {
            None => Err(Problem::NoSaveLocation),
            Some(config_path) => {
                if let Some(config_dir) = config_path.parent() {
                    fs::create_dir_all(config_dir).map_err(Problem::IO)?
                }
                let config = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(config_path)
                    .map_err(Problem::IO)?;

                serde_json::to_writer_pretty(config, self).map_err(|err| Problem::Save {
                    path: config_path.clone(),
                    err,
                })
            }
        }
    }

    pub fn to_settings(&self, wallpaper: Option<path::PathBuf>) -> flux::settings::Settings {
        use flux::settings;

        let color_mode = match &self.flux.color_mode {
            ColorMode::Preset { preset_name } => settings::ColorMode::Preset(*preset_name),
            ColorMode::ImageFile { image_path } => image_path.clone().map_or(
                settings::ColorMode::default(),
                settings::ColorMode::ImageFile,
            ),
            ColorMode::DesktopImage => wallpaper.map_or(
                settings::ColorMode::default(),
                settings::ColorMode::ImageFile,
            ),
        };
        flux::settings::Settings {
            color_mode,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FluxSettings {
    #[serde(flatten)]
    pub color_mode: ColorMode,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "colorMode", rename_all = "camelCase")]
pub enum ColorMode {
    Preset {
        #[serde(rename = "presetName")]
        preset_name: flux::settings::ColorPreset,
    },
    ImageFile {
        #[serde(rename = "imagePath")]
        image_path: Option<path::PathBuf>,
    },
    DesktopImage,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Preset {
            preset_name: Default::default(),
        }
    }
}

use flux::settings::ColorPreset;
impl ColorMode {
    pub const ALL: [ColorMode; 5] = [
        ColorMode::Preset {
            preset_name: ColorPreset::Original,
        },
        ColorMode::Preset {
            preset_name: ColorPreset::Plasma,
        },
        ColorMode::Preset {
            preset_name: ColorPreset::Poolside,
        },
        ColorMode::DesktopImage,
        ColorMode::ImageFile { image_path: None },
    ];
}

impl std::fmt::Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ColorMode::Preset { preset_name } => {
                    use flux::settings::ColorPreset::*;
                    match preset_name {
                        Original => "Original",
                        Plasma => "Plasma",
                        Poolside => "Poolside",
                        Freedom => "Freedom",
                    }
                }
                ColorMode::DesktopImage => "From wallpaper",
                ColorMode::ImageFile { .. } => "From image",
            }
        )
    }
}

#[derive(Default, Deserialize, Serialize, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
// Platform-specific configuration
pub struct PlatformConfig {
    #[cfg(windows)]
    pub windows: WindowsConfig,
}

#[derive(Default, Deserialize, Serialize, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
// Windows-specific configuration
pub struct WindowsConfig {
    pub fill_mode: FillMode,
}

#[derive(Default, Deserialize, Serialize, Copy, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
// Configures how Flux works with multiple displays.
pub enum FillMode {
    // Display a separate instance on each display
    None,
    // Span across and up to adjacent displays with matching dimensions
    #[default]
    Span,
    // Fill all displays with a single surface
    Fill,
}

#[cfg(windows)]
impl FillMode {
    pub const ALL: [FillMode; 3] = [FillMode::None, FillMode::Span, FillMode::Fill];
}

impl fmt::Display for FillMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FillMode::None => "None",
                FillMode::Span => "Span",
                FillMode::Fill => "Fill",
            }
        )
    }
}

#[derive(Debug)]
pub enum Problem {
    GetProjectDir,
    CreateProjectDir {
        path: path::PathBuf,
        err: io::Error,
    },
    ReadSettings {
        path: path::PathBuf,
        err: io::Error,
    },
    DecodeSettings {
        path: path::PathBuf,
        err: serde_json::Error,
    },
    UnsupportedVersion {
        version: String,
    },
    NoSaveLocation,
    Save {
        path: path::PathBuf,
        err: serde_json::Error,
    },
    IO(io::Error),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::GetProjectDir => write!(
                f,
                "Failed to find a suitable project directory to store settings"
            ),
            Problem::CreateProjectDir { path, err } => write!(
                f,
                "Failed to create the project directory at {}: {}",
                path.display(),
                err
            ),
            Problem::ReadSettings { path, err } => {
                write!(
                    f,
                    "Failed to read the settings file at {}: {}",
                    path.display(),
                    err
                )
            }
            Problem::DecodeSettings { path, err } => {
                write!(
                    f,
                    "Failed to decode settings file at {}: {}",
                    path.display(),
                    err
                )
            }
            Problem::UnsupportedVersion { version } => {
                write!(f, "Unsupported settings version {}.", version)
            }
            Problem::NoSaveLocation => write!(f, "No location available to save the settings"),
            Problem::Save { path, err } => {
                write!(
                    f,
                    "Failed to save the settings to {}: {}",
                    path.display(),
                    err
                )
            }
            Problem::IO(err) => {
                write!(f, "IO error: {}", err)
            }
        }
    }
}

trait UpgradableConfig {
    type UpgradedConfig;

    fn upgrade(&self) -> Self::UpgradedConfig;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize() {
        use serde_json::json;
        let config = Config {
            version: LATEST_VERSION,
            log_level: log::Level::Warn,
            flux: FluxSettings {
                color_mode: ColorMode::Preset {
                    preset_name: flux::settings::ColorPreset::Original,
                },
            },
            platform: PlatformConfig::default(),
            location: None,
        };
        let expected = json!({
            "version": 2,
            "logLevel": "warn",
            "flux": {
                "colorMode": "preset",
                "presetName": "Original"
            },
            "platform": {}
        });
        assert_eq!(serde_json::to_value(config).unwrap(), expected);
    }

    #[test]
    fn deserialize_from_0_1_0() {
        use serde_json::json;

        let json_config = json!({
            "version": "0.1.0",
            "log_level": "WARN",
            "flux": {
                "color_mode": { "Preset": "Original" },
            }
        });

        assert_eq!(
            Config::from_string(&json_config.to_string(), None).unwrap(),
            Config {
                version: LATEST_VERSION,
                log_level: log::Level::Warn,
                flux: FluxSettings {
                    color_mode: ColorMode::Preset {
                        preset_name: flux::settings::ColorPreset::Original,
                    },
                },
                platform: PlatformConfig::default(),
                location: None,
            }
        );
    }
}
