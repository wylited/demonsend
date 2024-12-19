use crate::config::Config;
use anyhow::Result;
use daemonize::Daemonize;
use localsend::Client;
use localsend::models::device::DeviceInfo;
use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

pub const PID_FILE: &str = "/tmp/demonsend.pid";
pub const LOG_FILE: &str = "/tmp/demonsend.log";
pub const SOCKET_PATH: &str = "/tmp/demonsend.sock";

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

pub fn daemon_logic(config: Config) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        if std::path::Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH).unwrap();
        }
        let listener = UnixListener::bind(SOCKET_PATH).unwrap();

        let info = DeviceInfo {
            alias: config.alias.clone(),
            version: "2.1".to_string(),
            device_model: config.deviceModel.clone(),
            device_type: config.deviceType.clone(),
            fingerprint: "demonsend only!".to_string(),
            port: config.port.clone(),
            protocol: "http".to_string(),
            download: true,
            announce: Some(true),
        };

        let client = Client::with_config(info.clone(), config.port.clone(), config.download_dir.clone()).await.unwrap();

        let (server_handle, udp_handle, announcement_handle) = client.start().await.unwrap();
        server_handle.await.unwrap();
        udp_handle.await.unwrap();
        announcement_handle.await.unwrap();

    });

    Ok(())
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

    println!("Received: {}", response);
    Ok(())
}
