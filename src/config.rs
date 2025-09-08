use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub export: Export,
    pub ui: Ui,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub path: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ui {
    pub show_instructions: bool,
    pub auto_save: bool,
}

impl Default for Config {
    fn default() -> Self {
        let export = Export {
            path: "~/Documents/slothtime_exports".to_string(),
            format: "csv".to_string(),
        };
        let ui = Ui {
            show_instructions: true,
            auto_save: true,
        };
        Self { export, ui }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = "config.toml";
        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self)?;
        fs::write("config.toml", content)?;
        Ok(())
    }
}
