use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::helpers;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    icons: Icons,
}

#[derive(Debug, Deserialize, Serialize)]
struct Icons {
    players: HashMap<String, String>,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            players: HashMap::from([(String::from("sample-player"), String::from("🔊"))]),
        }
    }
}

static EMPTY_STRING: String = String::new();

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = helpers::dir::get_and_create_dir(dirs::config_dir)?;
        let config_path = config_dir.join("config.toml");

        if File::create_new(&config_path).is_ok() {
            Config::create_default_config_file(&config_path)?;
            log::info!("No config file found, new created");
        }

        let file_str = fs::read_to_string(config_path)?;
        Ok(toml::from_str(&file_str)?)
    }

    fn create_default_config_file(path: &PathBuf) -> Result<(), ConfigError> {
        let doc_string = r"# You can configure unique text to display for any given player
# It works by doing a partial match against a players name
# So for Firefox for example, which is advertised as 'Mozilla Firefox', you'd want something like this:
#
# [icons.players]
# mozilla = 'icon'
#
# Even 'moz' would work, but obviously more specific strings are better to ensure the correct player is matched";
        fs::write(
            path,
            format!("{}\n{}", doc_string, toml::to_string(&Config::default())?),
        )?;
        Ok(())
    }

    pub fn get_player_icon_by_partial_match(&self, player_name: &str) -> &String {
        for (k, v) in self.icons.players.iter() {
            if player_name.to_lowercase().contains(&k.to_lowercase()) {
                log::debug!("player icon ({k}) matched with player: {player_name}");
                return v;
            }
        }
        &EMPTY_STRING
    }
}

#[derive(Debug)]
pub(crate) enum ConfigError {
    InvalidType(toml::de::Error),
    DefaultConfigFail(toml::ser::Error),
    IOError(std::io::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::InvalidType(value)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(value: toml::ser::Error) -> Self {
        Self::DefaultConfigFail(value)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidType(msg) => {
                write!(f, "Got invalid type while parsing config: {msg}")
            }
            ConfigError::IOError(err) => write!(f, "IO Error: {err}"),
            ConfigError::DefaultConfigFail(err) => {
                write!(f, "Failed to generate default config: {err}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
