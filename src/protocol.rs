use tokio::net::UdpSocket;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DeviceInfo {
    alias: String,
    version: String,
    device_model: Option<String>,
    device_type: String,
    fingerprint: String,
    port: u16,
    protocol: String,
    download: bool,
    announce: bool,
}

impl DeviceInfo {
    pub fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap()

    }
}

pub struct LocalSendInstance {
    pub device_info: DeviceInfo,
    pub udp_socket: Arc<UdpSocket>,
}

impl LocalSendInstance {
    pub async fn new() -> Self {
        let device_info = DeviceInfo {
            alias: "demonsend".to_string(),
            version: "2.0".to_string(),
            device_model: None,
            device_type: "headless".to_string(),
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
        }
    }

    pub async fn start_announcement_loop(&self) {
        let announcement = serde_json::to_string(&self.device_info).unwrap();
        let socket = self.udp_socket.clone();

        tokio::spawn(async move {
            loop {
                let _ = socket.send_to(
                    announcement.as_bytes(),
                    "224.0.0.167:53317"
                ).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    }
}
