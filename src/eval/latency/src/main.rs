mod args;

use args::Args;
use serde::Deserialize;
use std::process::{Child, Command, Stdio};
use std::net::TcpStream;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;

#[derive(Deserialize)]
struct Config {
    firecracker_binary_dir: String,
    firecracker_socket: String,
    config_file: String,
    target_ip: String,
    port_to_check: u16,
}

async fn check_port(ip: &str, port: u16) -> bool {
    let address = format!("{}:{}", ip, port);
    TcpStream::connect_timeout(&address.parse().unwrap(), Duration::from_millis(1)).is_ok() 
}

async fn wait_for_port(ip: &str, port: u16) -> bool {
    let max_retries = 10000;
    let mut retries = 0;
    while !check_port(ip, port).await {
        sleep(Duration::from_millis(1)).await;
        retries += 1;
        if retries > max_retries {
            return false;
        }
    }
    println!("Port {} is open after {} retries", port, retries);

    true
}

fn start_firecracker_vm(config: &Config) -> Result<Child, Box<dyn std::error::Error>> {
    let firecracker_args: Vec<String> = vec![
        format!("{}/firecracker", &config.firecracker_binary_dir.clone()),
        "--config-file".to_string(),
        config.config_file.clone(),
        "--api-sock".to_string(),
        config.firecracker_socket.clone()
    ];

    // Print the command we're going to run
    println!("Starting Firecracker VM with command: {:?}", firecracker_args);

    // Execute the program and send the output to /dev/null
    let firecracker_process = Command::new(&firecracker_args[0])
        .args(&firecracker_args[1..])
        .current_dir(&config.firecracker_binary_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Waiting for Firecracker to finish setup (in a real application, you'd likely handle this better)
    println!("Started Firecracker VM with PID: {}", firecracker_process.id());
    Ok(firecracker_process)
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse(std::env::args().collect()).unwrap();
    // Open file
    let file = std::fs::File::open(args.config()).expect("Failed to open config file");

    // Load configuration from file
    let config: Config = serde_json::from_reader(file)
        .expect("Failed to load config file");

    let current_time = Instant::now();

    let mut process_info: Child;
    // Start the Firecracker VM
    match start_firecracker_vm(&config) {
        Ok(process) => {
            let found = wait_for_port(&config.target_ip, config.port_to_check).await;
            process_info = process;
            if found {
                let elapsed_in_micros = current_time.elapsed().as_micros();
                println!("Firecracker VM is up and running! Took {} microseconds to start", elapsed_in_micros);
            } else {
                eprintln!("Failed to start Firecracker VM: Port {} is not open", config.port_to_check);
                process_info.kill().expect("Failed to kill Firecracker VM");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to start Firecracker VM: {}", e);
            // exit the program with an error code
            std::process::exit(1);
        }
    }

    // Kill the Firecracker VM
    process_info.kill().expect("Failed to kill Firecracker VM");

}
