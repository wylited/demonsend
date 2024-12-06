use daemonize::Daemonize;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process;

pub const PID_FILE: &str = "/tmp/demonsend.pid";
pub const LOG_FILE: &str = "/tmp/demonsend.log";
pub const SOCKET_PATH: &str = "/tmp/demonsend.sock";

pub fn start_daemon() {
    if is_running() {
        println!("Daemon is already running!");
        process::exit(1);
    }

    let stdout = File::create(LOG_FILE).unwrap();
    let stderr = File::create(LOG_FILE).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(PID_FILE)
        .chown_pid_file(true)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            // Daemon process starts here
            daemon_logic();
        }
        Err(e) => {
            eprintln!("Error starting daemon: {}", e);
            process::exit(1);
        }
    }
}

fn daemon_logic() {
    // Set up the Unix domain socket
    if std::path::Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH).unwrap();
    }

    let listener = UnixListener::bind(SOCKET_PATH).unwrap();

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
}

pub fn check_status() {
    if is_running() {
        println!("Daemon is running");
    } else {
        println!("Daemon is not running");
    }
}

pub fn stop_daemon() {
    if !is_running() {
        println!("Daemon is not running");
        return;
    }

    let pid = std::fs::read_to_string(PID_FILE).unwrap();
    let pid: i32 = pid.trim().parse().unwrap();

    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    println!("Daemon stopped");
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

pub fn send_ping() -> Result<(), Box<dyn std::error::Error>> {
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
