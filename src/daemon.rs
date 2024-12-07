use crate::protocol::LocalSendInstance;
use daemonize::Daemonize;
use log::{error, info};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process;
use anyhow::Result;

pub const PID_FILE: &str = "/tmp/demonsend.pid";
pub const PRINTLN_FILE: &str = "/tmp/demonsend.println";
pub const SOCKET_PATH: &str = "/tmp/demonsend.sock";

pub fn start_daemon() -> Result<()>{
    if is_running() {
        println!("Daemon is already running!");
        process::exit(1);
    }

    let stdout = File::create(PRINTLN_FILE)?;
    let stderr = File::create(PRINTLN_FILE)?;

    let daemonize = Daemonize::new()
        .pid_file(PID_FILE)
        .chown_pid_file(true)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            println!("Daemon started");
            daemon_logic();
        }
        Err(e) => {
            eprintln!("Error starting daemon: {}", e);
            process::exit(1);
        }
    }
    Ok(())
}

pub fn daemon_logic() -> Result<()>{
    // Create a new tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        // Set up the Unix domain socket
        if std::path::Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH).unwrap();
        }

        let listener = UnixListener::bind(SOCKET_PATH).unwrap();
        let demonsend = LocalSendInstance::new().await;

        // Start the announcement loop in a separate task
        let _announcement_loop = tokio::spawn({
            let announcement = demonsend.device_info.as_json();
            let socket = demonsend.udp_socket.clone();

            async move {
                loop {
                    let _ = socket
                        .send_to(announcement.as_bytes(), "224.0.0.167:53317")
                        .await;
                    println!("Announcement sent");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
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

                    if message == "ping" {
                        socket.write_all(b"pong").unwrap();
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
        return Ok(())
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

pub fn send_ping() -> Result<(), anyhow::Error> {
    if !is_running() {
        println!("Daemon is not running");
        return Ok(());
    }

    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(b"ping")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    println!("Received: {}", response);
    Ok(())
}
