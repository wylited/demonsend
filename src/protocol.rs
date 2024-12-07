use tokio::{sync::Mutex, net::UdpSocket};
use std::{collections::HashMap, sync::Arc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct LocalSend {
    pub device_info: DeviceInfo,
    pub udp_socket: Arc<UdpSocket>,
    pub peers: Arc<Mutex<HashMap<String, DeviceInfo>>>,
}

impl LocalSend {
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
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

        pub async fn handle_announcement(&self, buf: &[u8]) -> anyhow::Result<()> {
        if let Ok(device_info) = serde_json::from_slice::<DeviceInfo>(buf) {
            println!("Received announcement from {:?}", device_info);
            // Don't register ourselves
            if device_info.fingerprint != self.device_info.fingerprint {
                // If it's an announcement, respond with our info
                if device_info.announce {
                    let our_response = self.device_info.clone();
                    let mut response = our_response;
                    response.announce = false;

                    self.udp_socket
                        .send_to(
                            response.as_json().as_bytes(),
                            "224.0.0.167:53317",
                        )
                        .await?;
                }

                // Store the peer
                println!("New peer: {:?}", device_info.fingerprint);
                self.peers.lock().await.insert(device_info.fingerprint.clone(), device_info);
            }
        }
        Ok(())
    }
}
