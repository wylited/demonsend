use std::net::SocketAddrV4;

use crate::{models::device::DeviceInfo, Client};

pub mod http;
pub mod multicast;

impl Client {
    pub async fn announce(&self, socket: Option<SocketAddrV4>) -> crate::error::Result<()> {
        self.announce_http(socket).await?;
        self.announce_multicast().await?;
        Ok(())
    }

    async fn process_device(&self, message: &str) {
        if let Ok(device) = serde_json::from_str::<DeviceInfo>(message) {
            println!("Received device: {:?}", device);
            if device.fingerprint == self.device.fingerprint || device.announce != Some(true){
                return;
            }

            let mut peers = self.peers.lock().await;
            peers.insert(device.fingerprint.clone(), device.clone());

            // Announce in return upon receiving a valid device message and it wants announcements
            if let Err(e) = self.announce(None).await {
                eprintln!("Error during announcement: {}", e);
            }

        } else {
            eprintln!("Received invalid message: {}", message);
        }
    }
}
