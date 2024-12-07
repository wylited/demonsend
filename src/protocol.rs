use tokio::{sync::Mutex, net::UdpSocket};
use std::{collections::HashMap, sync::Arc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use axum::{
    extract::State,
    routing::{post, get, Router},
    Json
};
use std::net::SocketAddr;

use crate::protocol_v1::{handle_v1_info, handle_v1_register, DeviceInfoV1};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceInfoV2 {
    pub alias: String,
    pub version: String,
    pub deviceModel: Option<String>,
    pub deviceType: String,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: String,
    pub download: bool,
    pub announce: bool,
}

impl DeviceInfoV2 {
    pub fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn to_v1(&self) -> DeviceInfoV1 {
        DeviceInfoV1 {
            alias: self.alias.clone(),
            deviceModel: self.deviceModel.clone(),
            deviceType: self.deviceType.clone(),
            fingerprint: self.fingerprint.clone(),
            announcement: self.announce,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub device_info: DeviceInfoV2,
    pub udp_socket: Arc<UdpSocket>,
    pub peers: Arc<Mutex<HashMap<String, DeviceInfoV2>>>,
}

#[derive(Clone, Debug)]
pub struct LocalSend {
    pub device_info: DeviceInfoV2,
    pub udp_socket: Arc<UdpSocket>,
    pub peers: Arc<Mutex<HashMap<String, DeviceInfoV2>>>,
}

#[derive(Deserialize)]
pub struct InfoQuery {
    pub fingerprint: String,
}

async fn handle_v2_register(
    State(state): State<AppState>,
    Json(device_info): Json<DeviceInfoV2>,
) -> Json<DeviceInfoV2> {
    if device_info.fingerprint != state.device_info.fingerprint {
        state.peers.lock().await.insert(
            device_info.fingerprint.clone(),
            device_info
        );
    }
    Json(state.device_info)
}


impl LocalSend {
    pub async fn new() -> Self {
        let device_info = DeviceInfoV2 {
            alias: "demonsend".to_string(),
            version: "2.0".to_string(),
            deviceModel: None,
            deviceType: "headless".to_string(),
            fingerprint: Uuid::new_v4().to_string(),
            port: 53317,
            protocol: "http".to_string(),
            download: true,
            announce: true,
        };

        let socket = UdpSocket::bind("0.0.0.0:53317").await.unwrap();
        socket.join_multicast_v4("224.0.0.167".parse().unwrap(), "0.0.0.0".parse().unwrap()).unwrap();

        Self {
            device_info,
            udp_socket: Arc::new(socket),
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_http_server(&self) -> anyhow::Result<()> {
        let state = AppState {
            device_info: self.device_info.clone(),
            udp_socket: self.udp_socket.clone(),
            peers: self.peers.clone(),
        };

        let app = Router::new()
            .route("/api/localsend/v2/register", post(handle_v2_register))
            .route("/api/localsend/v1/register", post(handle_v1_register))
            .route("/api/localsend/v1/info", get(handle_v1_info))
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.device_info.port));
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        tokio::spawn(async {
            axum::serve(listener, app).await.unwrap();
        });

        println!("Started HTTP server");
        Ok(())
    }

    pub async fn handle_announcement(&self, buf: &[u8]) -> anyhow::Result<()> {
        // Try to parse as V2 first
        if let Ok(device_info) = serde_json::from_slice::<DeviceInfoV2>(buf) {
            self.handle_v2_announcement(device_info).await?;
        } else {
            // Try to parse as V1
            if let Ok(device_info) = serde_json::from_slice::<DeviceInfoV1>(buf) {
                self.handle_v1_announcement(device_info).await?;
            }
        }
        Ok(())
    }

    async fn handle_v2_announcement(&self, device_info: DeviceInfoV2) -> anyhow::Result<()> {
        if device_info.fingerprint != self.device_info.fingerprint {
            if device_info.announce {
                let mut response = self.device_info.clone();
                response.announce = false;

                self.udp_socket
                    .send_to(
                        response.as_json().as_bytes(),
                        "224.0.0.167:53317",
                    )
                    .await?;

                let client = reqwest::Client::new();
                let addr = format!("http://{}:{}/api/localsend/v2/register",
                    "127.0.0.1", // You'll need to get the actual IP from the UDP packet
                    device_info.port
                );

                match client.post(addr)
                    .json(&self.device_info)
                    .send()
                    .await {
                        Ok(_) => println!("HTTP registration successful"),
                        Err(e) => println!("HTTP registration failed: {}", e),
                }
            }

            self.peers.lock().await.insert(device_info.fingerprint.clone(), device_info);
        }
        Ok(())
    }
}
