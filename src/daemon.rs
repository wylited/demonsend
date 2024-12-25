use crate::config::Config;
use anyhow::Result;
use daemonize::Daemonize;
use localsend::Client;
use localsend::models::device::DeviceInfo;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::process;

pub const PID_FILE: &str = "/tmp/demonsend.pid";
pub const LOG_FILE: &str = "/tmp/demonsend.log";
pub const SOCKET_PATH: &str = "/tmp/demonsend.sock";
pub const VERSION: &str = "2.1";

pub fn start_daemon(config: Config) -> Result<()> {
    let _ = config;
    if is_running() {
        println!("Daemon is already running!");
        process::exit(1);
    }

    let stdout = File::create(LOG_FILE)?;
    let stderr = File::create(LOG_FILE)?;

    let daemonize = Daemonize::new()
        .pid_file(PID_FILE)
        .chown_pid_file(true)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            println!("Daemon started");
            daemon_logic(config)?;
        }
        Err(e) => {
            eprintln!("Error starting daemon: {}", e);
            process::exit(1);
        }
    }
    Ok(())
}


#[derive(Debug)]
enum Command {
    Version,
    Peers,
    Sessions,
    Info,
    Refresh,
    Send(String, PathBuf),
    Unknown(String),
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "version" => Command::Version,
            "peers" => Command::Peers,
            "sessions" => Command::Sessions,
            "info" => Command::Info,
            "refresh" => Command::Refresh,
            _ if s.starts_with("send") => {
                let parts: Vec<_> = s.split_whitespace().collect();
                if parts.len() < 3 {
                    Command::Unknown(s.to_string())
                } else {
                    Command::Send(parts[1].to_string(), PathBuf::from(parts[2]))
                }
            }
            _ => Command::Unknown(s.to_string()),
        }
    }
}

pub fn daemon_logic(config: Config) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        if std::path::Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH).unwrap();
        }

        let listener = UnixListener::bind(SOCKET_PATH).unwrap();

        let info = DeviceInfo {
            alias: config.alias.clone(),
            version: VERSION.to_string(),
            device_model: config.device_model.clone(),
            device_type: config.device_type.clone(),
            fingerprint: "demonsend only!".to_string(),
            port: config.port.clone(),
            protocol: "http".to_string(),
            download: true,
            announce: Some(true),
        };

        let client = Client::with_config(info.clone(), config.port.clone(), config.download_dir.clone()).await.unwrap();
        let client = Arc::new(client);
        let client_clone = client.clone();

        let (server_handle, udp_handle, announcement_handle) = client.start().await.unwrap();

        // Spawn a task to handle IPC
        let ipc_handle = tokio::spawn(async move {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buffer = [0; 1024];
                        match stream.read(&mut buffer) {
                            Ok(n) => {
                                let command = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                                let response = handle_command(&command, &client_clone).await;
                                let _ = stream.write_all(response.as_bytes());
                            }
                            Err(e) => eprintln!("Error reading from socket: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Error accepting connection: {}", e),
                }
            }
        });

        server_handle.await.unwrap();
        udp_handle.await.unwrap();
        announcement_handle.await.unwrap();
        ipc_handle.await.unwrap();
    });

    Ok(())
}

async fn handle_command(command: &str, client: &Arc<Client>) -> String {
    let cmd = Command::from(command);
    match cmd {
        Command::Version => {
            json!({
                "status": "success",
                "version": VERSION
            }).to_string()
        }
        Command::Peers => {
            let peers = client.peers.lock().await;
            let peer_list: Vec<_> = peers
                .iter()
                .map(|(fingerprint, (addr, info))| {
                    json!({
                        "fingerprint": fingerprint,
                        "address": addr.to_string(),
                        "alias": info.alias,
                        "device_model": info.device_model,
                        "device_type": info.device_type
                    })
                })
                .collect();

            json!({
                "status": "success",
                "peers": peer_list
            }).to_string()
        }
        Command::Sessions => {
            let sessions = client.sessions.lock().await;
            let session_list: Vec<_> = sessions
                .iter()
                .map(|(id, session)| {
                    json!({
                        "id": id,
                        "session": session
                    })
                })
                .collect();

            json!({
                "status": "success",
                "sessions": session_list
            }).to_string()
        }
        Command::Info => {
            json!({
                "status": "success",
                "device": {
                    "alias": client.device.alias,
                    "version": client.device.version,
                    "device_model": client.device.device_model,
                    "device_type": client.device.device_type,
                    "port": client.port,
                    "download_dir": client.download_dir
                }
            }).to_string()
        }
        Command::Refresh => {
            client.refresh_peers().await;
            json!({
                "status": "success",
                "message": "Refreshed peers"
            }).to_string()
        }
        Command::Send(peer, path) => {
            match client.send_file(peer.clone(), path).await {
                Ok(_) => {
                    json!({
                        "status": "success",
                        "message": format!("File sending initiated to peer: {}", peer)
                    }).to_string()
                }
                Err(e) => {
                    json!({
                        "status": "error",
                        "message": format!("Failed to send file: {}", e)
                    }).to_string()
                }
            }
        }
        Command::Unknown(cmd) => {
            json!({
                "status": "error",
                "message": format!("Unknown command: {}", cmd)
            }).to_string()
        }
    }
}

pub fn check_status() -> Result<()> {
    if is_running() {
        println!("Daemon is running");
    } else {
        println!("Daemon is not running");
    }
    Ok(())
}

pub fn stop_daemon() -> Result<()> {
    if !is_running() {
        println!("Daemon is not running");
        return Ok(());
    }

    let pid = std::fs::read_to_string(PID_FILE).unwrap();
    let pid: i32 = pid.trim().parse().unwrap();

    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    println!("Daemon stopped");
    Ok(())
}

pub fn is_running() -> bool {
    if !PathBuf::from(PID_FILE).exists() {
        return false;
    }

    let pid = match std::fs::read_to_string(PID_FILE) {
        Ok(pid) => pid.trim().parse::<i32>().unwrap_or(0),
        Err(_) => return false,
    };

    unsafe { libc::kill(pid, 0) == 0 }
}

pub fn send_command(command: &String) -> Result<()> {
    if !is_running() {
        println!("Daemon is not running");
        return Ok(());
    }

    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(command.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    // Parse and pretty print the JSON response
    match serde_json::from_str::<Value>(&response) {
        Ok(json) => {
            if json["status"] == "success" {
                if let Some(version) = json.get("version") {
                    println!("Version: {}", version);
                }
                if let Some(peers) = json.get("peers") {
                    println!("Peers:");
                    println!("{}", serde_json::to_string_pretty(peers).unwrap());
                }
                if let Some(sessions) = json.get("sessions") {
                    println!("Sessions:");
                    println!("{}", serde_json::to_string_pretty(sessions).unwrap());
                }
                if let Some(device) = json.get("device") {
                    println!("Device Info:");
                    println!("{}", serde_json::to_string_pretty(device).unwrap());
                }
                if let Some(message) = json.get("message") {
                    println!("{}", message);
                }
            } else if json["status"] == "error" {
                if let Some(message) = json.get("message") {
                    println!("Error: {}", message);
                }
            }
        }
        Err(e) => println!("Error parsing response: {}", e),
    }

    Ok(())
}
