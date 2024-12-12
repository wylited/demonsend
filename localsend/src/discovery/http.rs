use std::{collections::HashMap, net::SocketAddrV4, sync::Arc};
use tokio::sync::Mutex;

use axum::{extract::State, Extension, Json};

use crate::{models::device::DeviceInfo, Client};

impl Client {
    pub async fn announce_http(&self, ip: Option<SocketAddrV4>) -> crate::error::Result<()> {
        if let Some(ip) = ip {
            let url = format!("http://{}/api/localsend/v2/register", ip);
            let client = reqwest::Client::new();
            let res = client.post(&url).json(&self.device).send().await?;
            println!("{:?}", res);
        }
        Ok(())
    }

    pub async fn announce_http_legacy(&self, address_list: Vec<SocketAddrV4>) -> crate::error::Result<()> {
        let client = reqwest::Client::new();
        for ip in address_list {
            let url = format!("http://{}/api/localsend/v2/register", ip);
            let res = client.post(&url).json(&self.device).send().await?;
            println!("{:?}", res);
        }
        Ok(())
    }
}

pub async fn register_device(
    State(peers): State<Arc<Mutex<HashMap<String, DeviceInfo>>>>,
    Extension(client): Extension<DeviceInfo>,
    Json(device): Json<DeviceInfo>,
) -> Json<DeviceInfo> {
    peers.lock().await.insert(device.fingerprint.clone(), device.clone());
    Json(client)
}
