use std::{collections::HashMap, path::PathBuf, sync::Arc, time::SystemTime};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{filters::body::bytes, Filter};
use crate::{api::{handle_rejection, ApiError}, discovery::Discovery, file::FileMetadata, DeviceInfo, LocalSendError, PrepareUploadRequest, PrepareUploadResponse, Result};

#[derive(Clone)]
pub struct Client {
    pub device_info: DeviceInfo,
    pub discovery: Arc<Discovery>,
    pub peers: Arc<Mutex<HashMap<String, DeviceInfo>>>,
    pub download_dir: Arc<PathBuf>,
    pub sessions: Arc<Mutex<HashMap<String, TransferSession>>>,
    pub client: reqwest::Client,
}

pub struct TransferSession {
    pub session_id: String,
    pub device_info: DeviceInfo,
    pub files: HashMap<String, FileMetadata>,
    pub file_tokens: HashMap<String, String>,
    pub created_at: SystemTime,
    pub status: SessionStatus,
}

#[derive(PartialEq)]
pub enum SessionStatus {
    Preparing,
    Transferring,
    Completed,
    Cancelled,
}

impl Client {
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
            sessions: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
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
           rx_inst.client.post(&format!("http://{}/api/localsend/v2/register", peer.1))
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

    pub fn handle_register(&self, new_device_info: DeviceInfo) -> impl warp::Reply {
        // if the finger print is the same, do nothing
        if new_device_info.fingerprint == self.device_info.fingerprint {
           return warp::reply::reply(); // early return
        }

        self.add_peer(&new_device_info);

        // TODO implement response (http and udp, udp easy, http need to find socket addr)

        warp::reply::reply()
    }

    pub fn start_server(&self) {
        let client = Arc::new(self.clone());

        // Register route
        let register_route = {
            let client = Arc::clone(&client);
            warp::path!("api" / "localsend" / "v2" / "register")
                .and(warp::post())
                .and(warp::body::json())
                .map(move |new_device_info: DeviceInfo| {
                    client.handle_register(new_device_info)
                })
        };

        // Prepare upload route
        let prepare_upload = {
            let client = Arc::clone(&client);
            warp::path!("api" / "localsend" / "v2" / "prepare-upload")
                .and(warp::post())
                .and(warp::query::<HashMap<String, String>>())
                .and(warp::body::json())
                .and_then(move |query: HashMap<String, String>, request: PrepareUploadRequest| {
                    let client = Arc::clone(&client);
                    async move {
                        // Check PIN if required
                        if let Some(pin) = query.get("pin") {
                            // TODO Implement pins
                        }
                        client.handle_prepare_upload(request)
                              .await
                              .map_err(|e| warp::reject::custom(ApiError::UnknownError(
                                  "idk good luck".to_string()
                              )))
                    }
                })
        };

        // Upload route
        let upload = {
            let client = Arc::clone(&client);
            warp::path!("api" / "localsend" / "v2" / "upload")
                .and(warp::post())
                .and(warp::query::<HashMap<String, String>>())
                .and(warp::body::bytes())
                .and_then(move |query: HashMap<String, String>, bytes: bytes::Bytes| {
                    let client = Arc::clone(&client);
                    async move {
                        let session_id = query.get("sessionId")
                                              .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;
                        let file_id = query.get("fileId")
                                           .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;
                        let token = query.get("token")
                                         .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;

                        client.handle_upload(session_id, file_id, token, bytes)
                              .await
                              .map_err(|e| warp::reject::custom(ApiError::UnknownError(
                                  "cannot upload good luck".to_string()
                              )))
                    }
                })
        };

        // Cancel route
        let cancel = {
            let client = Arc::clone(&client);
            warp::path!("api" / "localsend" / "v2" / "cancel")
                .and(warp::post())
                .and(warp::query::<HashMap<String, String>>())
                .and_then(move |query: HashMap<String, String>| {
                    let client = Arc::clone(&client);
                    async move {
                        let session_id = query.get("sessionId")
                                              .ok_or_else(|| warp::reject::custom(ApiError::InvalidParameters))?;

                        client.handle_cancel(session_id)
                              .await
                              .map_err(|e| warp::reject::custom(ApiError::UnknownError(
                                  "cannot cancel good luck".to_string()
                              )))
                    }
                })
        };

        // Combine all routes
        let routes = register_route
            .or(prepare_upload)
            .or(upload)
            .or(cancel);

        let port = self.device_info.port;
        tokio::spawn(async move {
            warp::serve(routes)
                .run(([0, 0, 0, 0], port))
                .await;
        });
    }
}
