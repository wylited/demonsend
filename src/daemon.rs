use crate::protocol::LocalSend;
use anyhow::Result;
use daemonize::Daemonize;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process;

pub const PID_FILE: &str = "/tmp/demonsend.pid";
pub const LOG_FILE: &str = "/tmp/demonsend.log";
pub const SOCKET_PATH: &str = "/tmp/demonsend.sock";

pub fn start_daemon() -> Result<()> {
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
            daemon_logic()?;
        }
        Err(e) => {
            eprintln!("Error starting daemon: {}", e);
            process::exit(1);
        }
    }
    Ok(())
}

pub fn daemon_logic() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        if std::path::Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH).unwrap();
        }

        let listener = UnixListener::bind(SOCKET_PATH).unwrap();
        let demonsend = LocalSend::new().await;

        if let Err(e) = demonsend.start_http_server().await {
            eprintln!("Failed to start HTTP server: {}", e);
            return;
        }

        // Start the announcement loop
        let _announcement_loop = tokio::spawn({
            let announcement = demonsend.device_info.as_json();
            let socket = demonsend.udp_socket.clone();

            async move {
                loop {
                    let _ = socket
                        .send_to(announcement.as_bytes(), "224.0.0.167:53317")
                        .await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });

        // Start the UDP listener loop
        let _udp_listener = tokio::spawn({
            let demonsend = demonsend.clone();

            async move {
                let mut buf = [0; 1024];
                loop {
                    if let Ok((size, _)) = demonsend.udp_socket.recv_from(&mut buf).await {
                        if let Err(e) = demonsend.handle_announcement(&buf[..size]).await {
                            eprintln!("Error handling announcement: {}", e);
                        }
                    }
                }
            }
        });

        // Handle Unix socket communications
        loop {
            match listener.accept() {
                Ok((mut socket, _)) => {
                    let mut buffer = [0; 1024];
                    let n = socket.read(&mut buffer).unwrap();
                    let message = String::from_utf8_lossy(&buffer[..n]);

                    match message.as_ref() {
                        "ping" => {
                            socket.write_all(b"pong").unwrap();
                        }
                        "list" => {
                            let peers = demonsend.peers.lock().await;
                            let peers_json = serde_json::to_string(&*peers).unwrap();
                            socket.write_all(peers_json.as_bytes()).unwrap();
                        }
                        "refresh" => {
                            let mut peers = demonsend.peers.lock().await;
                            peers.clear();
                        }
                        _ => {
                            socket.write_all(b"unknown command").unwrap();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
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
