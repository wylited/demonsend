use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use crate::{discovery::Discovery, DeviceInfo, LocalSendError, Result};

#[derive(Clone)]
pub struct LocalSend {
    pub device_info: DeviceInfo,
    pub discovery: Arc<Discovery>,
    pub peers: Arc<Mutex<HashMap<String, DeviceInfo>>>,
    pub download_dir: Arc<PathBuf>,
}

impl LocalSend {
    pub async fn new(
        device_info: DeviceInfo,
        peers: Arc<Mutex<HashMap<String, DeviceInfo>>>,
        download_dir: Arc<PathBuf>,
    ) -> Result<Self> {
        let port = device_info.port;

        let discovery = Discovery::new(port)
            .await
            .map_err(|e| {
                LocalSendError::Unknown(format!("Failed to create Discovery: {e}"))
            })?;

        Ok(Self {
            device_info,
            discovery: Arc::new(discovery),
            peers,
            download_dir,
        })
    }

    pub async fn add_peer(&self, peer: &DeviceInfo) {
        if peer.fingerprint == self.device_info.fingerprint {
            return;
        }
        let mut peers = self.peers.lock().await;
        peers.insert(peer.fingerprint.clone(), peer.clone());
    }

    pub fn start_discovery(&self) {
        let discovery = self.discovery.clone();
        let device_info = self.device_info.clone();
        let _announcement_loop = tokio::spawn(async move {
            loop {
                if let Err(e) = discovery.announce(&device_info).await {
                    eprintln!("Failed to announce device info: {e}");
                }
                tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
            }
        });

        let rx_inst = self.clone();
        let rx_discovery = self.discovery.clone();
        let client = reqwest::Client::new();
        let _listen_loop = tokio::spawn(async move {
            loop {
                match rx_discovery.listen().await {
                    Ok(peer) => {
                        rx_inst.add_peer(&peer.0).await;
                        if peer.0.announce { // Announce the device info to the peer if announce true
                            // Send a post request over http to the api /v2
                            client.post(&format!("http://{}/api/localsend/v2/register", peer.1))
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
    }
}
