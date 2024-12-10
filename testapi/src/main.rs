use localsend_rs::{DeviceInfo, DeviceType};
use std::{path::PathBuf, sync::Arc};
use serde::{Serialize, Deserialize};
use directories::UserDirs;

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Thread panicked! Info: {:?}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!("Panic occurred in file '{}' at line {}", location.file(), location.line());
        }
    }));
    let config = Config::default();
    println!("{:?}", config);
    let info = DeviceInfo::new(
        config.alias.clone(),
        config.deviceModel.clone(),
        config.deviceType.clone(),
        config.port.clone(),
        config.protocol.clone(),
        config.download.clone(),
        config.announce.clone(),
    );
    println!("{:?}", info);
    let client = localsend_rs::client::Client::new(
        info,
        Arc::new(PathBuf::from(config.download_dir.clone())),
    )
    .await
    .unwrap();

    client.start_discovery();
    client.start_server();

    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down...");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub download_dir: String,
    pub alias: String,
    pub deviceModel: Option<String>,
    pub deviceType: Option<DeviceType>,
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
                deviceType: Some(DeviceType::Headless),
                port: 53317,
                protocol: "http".to_string(),
                download: true,
                announce: true,
            };
        }
        return Config {
            download_dir: "/home/wyli/Downloads".to_string(),
            alias: "demonsend".to_string(),
            deviceModel: None,
            deviceType: Some(DeviceType::Headless),
            port: 53317,
            protocol: "http".to_string(),
            download: true,
            announce: true,
        };
    }
}
