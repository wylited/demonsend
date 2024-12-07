use axum::{extract::{Query, State}, Json};
use serde::{Deserialize, Serialize};

use crate::protocol::{AppState, DeviceInfoV2, InfoQuery, LocalSend};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceInfoV1 {
    pub alias: String,
    pub deviceModel: Option<String>,
    pub deviceType: String,
    pub fingerprint: String,
    pub announcement: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceInfoV1Response {
    pub alias: String,
    pub deviceModel: Option<String>,
    pub deviceType: String,
}

pub async fn handle_v1_register(
    State(state): State<AppState>,
    Json(device_info): Json<DeviceInfoV1>,
) -> Json<DeviceInfoV1> {
    if device_info.fingerprint != state.device_info.fingerprint {
        let device_info_v2 = DeviceInfoV2 {
            alias: device_info.alias,
            version: "1.0".to_string(),
            deviceModel: device_info.deviceModel,
            deviceType: device_info.deviceType,
            fingerprint: device_info.fingerprint,
            port: 53317,
            protocol: "http".to_string(),
            download: true,
            announce: device_info.announcement,
        };
        state.peers.lock().await.insert(
            device_info_v2.fingerprint.clone(),
            device_info_v2
        );
    }
    Json(state.device_info.to_v1())
}

pub async fn handle_v1_info(
    State(state): State<AppState>,
    Query(query): Query<InfoQuery>,
) -> Json<DeviceInfoV1Response> {
    if query.fingerprint != state.device_info.fingerprint {
        Json(DeviceInfoV1Response {
            alias: state.device_info.alias,
            deviceModel: state.device_info.deviceModel,
            deviceType: state.device_info.deviceType,
        })
    } else {
        Json(DeviceInfoV1Response {
            alias: String::new(),
            deviceModel: None,
            deviceType: String::new(),
        })
    }
}

impl LocalSend {
    pub async fn handle_v1_announcement(&self, device_info: DeviceInfoV1) -> anyhow::Result<()> {
        if device_info.fingerprint != self.device_info.fingerprint {
            // If it's an announcement, respond with our info via UDP
            if device_info.announcement {
                let response = self.device_info.to_v1();
                self.udp_socket
                    .send_to(
                        serde_json::to_string(&response)?.as_bytes(),
                        "224.0.0.167:53317",
                    )
                    .await?;
            }

            // Convert V1 to V2 for storage
            let device_info_v2 = DeviceInfoV2 {
                alias: device_info.alias,
                version: "1.0".to_string(),
                deviceModel: device_info.deviceModel,
                deviceType: device_info.deviceType,
                fingerprint: device_info.fingerprint,
                port: 53317,
                protocol: "http".to_string(),
                download: true,
                announce: device_info.announcement,
            };

            self.peers.lock().await.insert(device_info_v2.fingerprint.clone(), device_info_v2);
        }
        Ok(())
    }
}
