use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub file: PathBuf,
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
        // Get home dir/ location for config
        let home_dir = dirs::home_dir().unwrap();
        let config_dir = home_dir.join(".slothtime");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).unwrap();
        }
        let file = config_dir.join("slothtime.toml");
        let export = Export {
            path: "~/Documents/slothtime_exports".to_string(),
            format: "csv".to_string(),
        };
        let ui = Ui {
            show_instructions: true,
            auto_save: true,
        };
        Self { file, export, ui }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config = Self::default();
        if config.file.exists() {
            let content = fs::read_to_string(config.file)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self)?;
        fs::write(self.file.clone(), content)?;
        Ok(())
    }
}
