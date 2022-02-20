use std::{ops::Deref, path::PathBuf};

use regex::Regex;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to parse configuration CSV")]
    InvalidCsv(#[from] csv::Error),
    #[error("Invalid regex in configuration")]
    InvalidRegex(#[from] regex::Error),
    #[error("Unable to open the config path")]
    InvalidPath,
    #[error("Unable to save configuration")]
    Save(std::io::Error),
}

pub struct Config(Vec<(Regex, Regex)>);

impl Config {
    pub fn csv_path() -> Result<PathBuf, ConfigError> {
        if let Some(dirs) = directories::ProjectDirs::from("dev.berwyn", "", "CleanWeb") {
            let path = dirs.config_dir();
            let mut path = path.to_owned();

            path.push("config.csv");

            Ok(path)
        } else {
            Err(ConfigError::InvalidPath)
        }
    }

    fn default_rules() -> Vec<(Regex, Regex)> {
        vec![
            (
                Regex::new("twitter.com").unwrap(),
                Regex::new("t|s").unwrap(),
            ),
            (Regex::new(".*").unwrap(), Regex::new("utm_.*").unwrap()),
        ]
    }

    pub fn ensure_exists(&self) -> Result<(), ConfigError> {
        let path = Self::csv_path()?;

        std::fs::create_dir_all(path.parent().unwrap()).map_err(ConfigError::Save)?;

        let contents = if self.0.is_empty() {
            Self::default_rules()
        } else {
            self.0.clone()
        };

        let mut writer = csv::Writer::from_path(path)?;
        for (host, param) in contents {
            writer.write_record(vec![host.as_str(), param.as_str()])?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self(Self::default_rules())
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = ConfigError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut entries = Vec::new();
        let mut reader = csv::Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: ConfigEntry = result?;

            let host_regex = Regex::new(&record.host_regex)?;
            let param_regex = Regex::new(&record.param_regex)?;

            entries.push((host_regex, param_regex));
        }

        Ok(Self(entries))
    }
}

impl Deref for Config {
    type Target = Vec<(Regex, Regex)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize)]
struct ConfigEntry {
    host_regex: String,
    param_regex: String,
}
