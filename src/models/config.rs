use std::{
    collections::HashMap,
    fs::{self, File},
};

use toml::{Table, Value};

use crate::helpers;

pub struct Config {
    player_icons: HashMap<String, String>,
}

static EMPTY_STRING: String = String::new();

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = helpers::dir::get_and_create_dir(dirs::config_dir)?;
        let config_path = config_dir.join("config.toml");

        if File::create_new(&config_path).is_ok() {
            log::info!("No config file found, new created");
        }

        let file_str = fs::read_to_string(config_path)?;
        let table: Table = file_str.parse()?;

        let mut config = Config {
            player_icons: HashMap::new(),
        };
        table.iter().for_each(|(k, v)| match k.as_str() {
            "player_icons" => config.parse_player_icons(v),
            _ => log::warn!("Got unknown key while parsing config: '{k}'"),
        });

        Ok(config)
    }

    fn parse_player_icons(&mut self, v: &Value) {
        if !v.is_table() {
            log::warn!("Found player_icons key, but the value is not a table like expected");
            return;
        }

        let mut has_errors = false;
        self.player_icons = v
            .as_table()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.as_str()
                        .unwrap_or_else(|| {
                            has_errors = true;
                            log::warn!(
                                "'{k}' got unexpected value. Only strings of text are supported"
                            );
                            ""
                        })
                        .to_string(),
                )
            })
            .collect();

        if has_errors {
            log::warn!("Invalid values found while parsing 'player_icons'");
        }
    }

    pub fn get_player_icon_by_partial_match(&self, player_name: &str) -> &String {
        for (k, v) in self.player_icons.iter() {
            if player_name.to_lowercase().contains(&k.to_lowercase()) {
                log::debug!("player icon ({k}) matched with player: {player_name}");
                return v;
            }
        }
        &EMPTY_STRING
    }
}
