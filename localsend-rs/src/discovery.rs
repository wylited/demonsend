use crate::{DeviceInfo, LocalSendError, Result};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct Discovery {
    socket: UdpSocket,
    multicast_addr: SocketAddr,
}

impl Discovery {
    pub async fn new(port: u16) -> Result<Self> {
        let socket = UdpSocket::bind(("224.0.0.167", port)).await.map_err(|e| {
            LocalSendError::Unknown(format!("Failed to bind socket: {e}"))
        })?;

        let multicast_addr = "224.0.0.167:53317".parse().unwrap();

        Ok(Self {
            socket,
            multicast_addr,
        })
    }

    pub async fn announce(&self, device_info: &DeviceInfo) -> Result<()> {
        let message = serde_json::to_string(&device_info).map_err(|e| {
            LocalSendError::Json(e)
        })?;

        self.socket.send_to(message.as_bytes(), self.multicast_addr).await.map_err(|e| {
            LocalSendError::Unknown(format!("Failed to send announcement: {e}"))
        })?;

        Ok(())
    }

    pub async fn listen(&self) -> Result<(DeviceInfo, SocketAddr)> {
        let mut buf = vec![0u8; 65535];
        let (len, addr) = self.socket.recv_from(&mut buf).await.map_err(|e| {
            LocalSendError::Unknown(format!("Failed to receive data: {e}"))
        })?;

        let message = String::from_utf8_lossy(&buf[..len]);
        let device_info: DeviceInfo = serde_json::from_str(&message).map_err(|e| {
            LocalSendError::Json(e)
        })?;

        Ok((device_info, addr))
    }
}
