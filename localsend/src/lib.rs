pub mod discovery;
pub mod error;
pub mod models;

use crate::models::device::DeviceInfo;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};

pub struct Client {
    pub device: DeviceInfo,
    pub socket: UdpSocket,
    pub multicast_addr: SocketAddrV4,
    pub peers: Arc<Mutex<HashMap<String, DeviceInfo>>>,
}

impl Client {
    pub fn default() -> crate::error::Result<Self> {
        let device = DeviceInfo::default();
        let socket = UdpSocket::bind("0.0.0.0:53317").unwrap();
        socket.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 167), &Ipv4Addr::new(0, 0, 0, 0))?;
        let multicast_addr = SocketAddrV4::new(Ipv4Addr::new(224, 0, 0, 167), 53317);
        let peers = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            device,
            socket,
            multicast_addr,
            peers,
        })
    }
}
