use daemonize::Daemonize;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::process;
use std::thread;
use std::time::Duration;

pub const PID_FILE: &str = "/tmp/example_daemon.pid";
pub const LOG_FILE: &str = "/tmp/example_daemon.log";

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
    loop {
        // Example daemon work
        let mut log = OpenOptions::new()
            .append(true)
            .create(true)
            .open(LOG_FILE)
            .unwrap();

        use std::io::Write;
        writeln!(log, "Daemon is running...").unwrap();
        thread::sleep(Duration::from_secs(60));
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
