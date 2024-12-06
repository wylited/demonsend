use anyhow::{Context, Result};
use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub download_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        if let Some(user_dirs) = UserDirs::new() {
            return Config {
                download_dir: user_dirs
                    .download_dir()
                    .expect("there was no download directory")
                    .to_str()
                    .unwrap()
                    .to_string(),
            };
        }
        return Config {
            download_dir: "".to_string(),
        };
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let contents = fs::read_to_string(&config_path).context("Failed to read config file")?;

        toml::from_str(&contents).context("Failed to parse config file")
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // Ensure the config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string(self)?;
        fs::write(&config_path, contents).context("Failed to write config file")
    }
}

fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("tech", "wyli", "demonsend")
        .context("Failed to determine config directory")?;

    Ok(proj_dirs.config_dir().join("config.toml"))
}

impl Config {
    pub fn initialize_interactive() -> Result<Self> {
        println!("Welcome to demonsend configuration!");
        println!("Please enter your preferred downloads directory:");

        let mut download_dir = String::new();
        std::io::stdin().read_line(&mut download_dir)?;
        let download_dir = download_dir.trim().to_string();

        let config = Config { download_dir };

        config.save()?;
        println!("Configuration saved successfully!");

        Ok(config)
    }
}
