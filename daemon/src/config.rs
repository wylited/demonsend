use anyhow::{Context, Result};
use directories::{ProjectDirs, UserDirs};
use inquire::{Confirm, Select, Text};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub download_dir: String,
    pub alias: String,
    pub deviceModel: Option<String>,
    pub deviceType: String,
    pub port: u16,
    pub protocol: String,
    pub download: bool,
    pub announce: bool,
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
                alias: "demonsend".to_string(),
                deviceModel: None,
                deviceType: "headless".to_string(),
                port: 53317,
                protocol: "http".to_string(),
                download: true,
                announce: true,
            };
        }
        return Config {
            download_dir: "".to_string(),
            alias: "demonsend".to_string(),
            deviceModel: None,
            deviceType: "headless".to_string(),
            port: 53317,
            protocol: "http".to_string(),
            download: true,
            announce: true,
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

        let default_dirs = UserDirs::new()
            .and_then(|dirs| dirs.download_dir().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_default();

        let download_dir = loop {
            if let Ok(input) = Text::new("Enter your preferred downloads directory:").prompt() {
                if let Ok(path) = fs::canonicalize(&input) {
                    break path.to_string_lossy().to_string();
                }
                println!("Please enter a valid path");
            }
        };

        let alias = Text::new("Enter your alias:")
            .with_default("demonsend")
            .prompt()?;

        let deviceModel = Text::new("Enter your device model:")
            .with_default("")
            .prompt()
            .ok();

        let device_types = vec!["mobile", "desktop", "web", "headless", "server"];

        let deviceType = Select::new("Select your device type:", device_types)
            .with_starting_cursor(3) // headless as default
            .prompt()?;

        let protocols = vec!["http", "https"];
        let protocol = Select::new("Select protocol:", protocols)
            .with_starting_cursor(0) // https as default
            .prompt()?;

        let port = loop {
            if let Ok(input) = Text::new("Enter port number:")
                .with_default("53317")
                .prompt()
            {
                if let Ok(port) = input.parse::<u16>() {
                    break port;
                }
                println!("Please enter a valid port number");
            }
        };

        let download = Confirm::new("Enable downloads?")
            .with_default(true)
            .prompt()?;

        let announce = Confirm::new("Enable announcements?")
            .with_default(true)
            .prompt()?;

        let config = Config {
            download_dir,
            alias,
            deviceModel,
            deviceType: deviceType.to_string(),
            port,
            protocol: protocol.to_string(),
            download,
            announce,
        };

        config.save()?;
        println!("Configuration saved successfully!");

        Ok(config)
    }
}
