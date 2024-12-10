use crate::config::Config;
use anyhow::Result;
use daemonize::Daemonize;
use localsend_rs::DeviceInfo;
use std::fs::File;
use std::io::{Read, Write};
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

        let info = DeviceInfo::new(config.alias.clone(), config.deviceModel.clone(), config.deviceType.clone(), config.port.clone(), config.protocol.clone(), config.download.clone(), config.announce.clone());

        let client = localsend_rs::client::Client::new(info, Arc::new(PathBuf::from(config.download_dir.clone()))).await.unwrap();

        client.start_discovery();
        client.start_server();
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
