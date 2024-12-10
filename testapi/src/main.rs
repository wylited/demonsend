use localsend_rs::{DeviceInfo, DeviceType};
use std::{path::PathBuf, sync::Arc};
use serde::{Serialize, Deserialize};
use directories::UserDirs;
use warp::Filter;
use localsend_rs::api::ApiError;
use std::collections::HashMap;
use localsend_rs::PrepareUploadRequest;

#[tokio::main]
async fn main() {
    // std::panic::set_hook(Box::new(|panic_info| {
    //     eprintln!("Thread panicked! Info: {:?}", panic_info);
    //     if let Some(location) = panic_info.location() {
    //         eprintln!("Panic occurred in file '{}' at line {}", location.file(), location.line());
    //     }
    // }));
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

    let discovery = client.discovery.clone();
    let device_info = client.device_info.clone();
    let _announcement_loop = tokio::spawn(async move {
        loop {
            println!("discoverying stuff");
            if let Err(e) = discovery.announce(&device_info).await {
                eprintln!("Failed to announce device info: {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
        }
    });

    let rx_inst = client.clone();
    let rx_discovery = client.discovery.clone();
    let _listen_loop = tokio::spawn(async move {
        loop {
            match rx_discovery.listen().await {
                Ok(peer) => {
                    rx_inst.add_peer(&peer.0).await;
                    if peer.0.announce { // Announce the device info to the peer if announce true
                        // Send a post request over http to the api /v2
                        rx_inst.client.post(&format!("http://{}/api/localsend/v2/register", peer.1))
                                      .json(&rx_inst.device_info)
                                      .send()
                                      .await
                                      .map_err(|e| {
                                          eprintln!("Failed to send device info to peer: {}", e);
                                      }).ok();

                        // fallback to udp
                        rx_inst.discovery.announce(&rx_inst.device_info).await.map_err(|e| {
                            eprintln!("Failed to announce device info: {}", e);
                        }).ok();
                    }
                }
                Err(e) => {
                    eprintln!("Failed to listen for peers: {e}");
                }
            }
        }
    });

    let client = Arc::new(client.clone());

    // Prepare register route
    let register_route = {
        let client = Arc::clone(&client);
        warp::path!("api" / "localsend" / "v2" / "register")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(move |new_device_info: DeviceInfo| {
                let client = Arc::clone(&client);
                async move {
                    client.handle_register(new_device_info)
                          .await
                          .map_err(|e| warp::reject::custom(ApiError::UnknownError(
                              "Registration failed".to_string()
                          )))
                }
            })
    };

    // Prepare upload route
    let prepare_upload = {
        let client = Arc::clone(&client);
        warp::path!("api" / "localsend" / "v2" / "prepare-upload")
            .and(warp::post())
            .and(warp::query::<HashMap<String, String>>())
            .and(warp::body::json())
            .and_then(move |query: HashMap<String, String>, request: PrepareUploadRequest| {
                let client = Arc::clone(&client);
                async move {
                    // Check PIN if required
                    if let Some(pin) = query.get("pin") {
                        // TODO Implement pins
                    }
                    client.handle_prepare_upload(request)
                          .await
                          .map_err(|e| warp::reject::custom(ApiError::UnknownError(
                              "idk good luck".to_string()
                          )))
                }
            })
    };

    // Upload route
    let upload = {
        let client = Arc::clone(&client);
        warp::path!("api" / "localsend" / "v2" / "upload")
            .and(warp::post())
            .and(warp::query::<HashMap<String, String>>())
            .and(warp::body::bytes())
            .and_then(move |query: HashMap<String, String>, bytes: bytes::Bytes| {
                let client = Arc::clone(&client);
                async move {
                    let session_id = query.get("sessionId")
                                          .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;
                    let file_id = query.get("fileId")
                                       .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;
                    let token = query.get("token")
                                     .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;

                    client.handle_upload(session_id, file_id, token, bytes)
                          .await
                          .map_err(|e| warp::reject::custom(ApiError::UnknownError(
                              "cannot upload good luck".to_string()
                          )))
                }
            })
    };

    // Cancel route
    let cancel = {
        let client = Arc::clone(&client);
        warp::path!("api" / "localsend" / "v2" / "cancel")
            .and(warp::post())
            .and(warp::query::<HashMap<String, String>>())
            .and_then(move |query: HashMap<String, String>| {
                let client = Arc::clone(&client);
                async move {
                    let session_id = query.get("sessionId")
                                          .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;

                    client.handle_cancel(session_id)
                          .await
                          .map_err(|e| warp::reject::custom(ApiError::UnknownError(
                              "cannot cancel good luck".to_string()
                          )))
                }
            })
    };

    // Combine all routes
    let routes = register_route
        .or(prepare_upload)
        .or(upload)
        .or(cancel);

    let port = client.device_info.port;
    tokio::spawn(async move {
        println!("start service");
        warp::serve(routes)
            .run(([0, 0, 0, 0], port))
            .await;
        println!("http died");
    });
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
    fn default() -> Config {
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
                port: 53318,
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
            port: 53318,
            protocol: "http".to_string(),
            download: true,
            announce: true,
        };
    }
}
