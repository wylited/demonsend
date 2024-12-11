use crate::{Client, DeviceInfo};
use std::time::Duration;

impl Client {
    pub async fn announce(&self) -> crate::error::Result<()> {
        let msg = self.device.to_json()?;
        let addr = self.multicast_addr.clone();
        self.socket.send_to(msg.as_bytes(), addr)?;
        Ok(())
    }

    pub async fn listen(&self) -> crate::error::Result<()> {
        let mut buf = [0; 1024];

        loop {
            self.socket.set_read_timeout(Some(Duration::from_secs(5)))?;
            if let Err(e) = self.receive_message(&mut buf).await {
                eprintln!("Error receiving message: {}", e);
            }
        }
    }

    async fn receive_message(&self, buf: &mut [u8]) -> crate::error::Result<()> {
        match self.socket.recv_from(buf) {
            Ok((size, src)) => {
                let received_msg = String::from_utf8_lossy(&buf[..size]);
                println!("Received message from {}: {}", src, received_msg);
                self.process_message(&received_msg).await;
            }
            Err(e) => {
                return Err(e.into()); // Convert error to your crate's error type
            }
        }
        Ok(())
    }

    async fn process_message(&self, message: &str) {
        if let Ok(device) = serde_json::from_str::<DeviceInfo>(message) {
            println!("Received device: {:?}", device);
            if device.fingerprint == self.device.fingerprint || device.announce != Some(true){
                return;
            }

            let mut peers = self.peers.lock().unwrap();
            peers.insert(device.fingerprint.clone(), device.clone());

            // Announce in return upon receiving a valid device message and it wants announcements
            if let Err(e) = self.announce().await {
                eprintln!("Error during announcement: {}", e);
            }

        } else {
            eprintln!("Received invalid message: {}", message);
        }
    }
}
