mod args;
mod firecracker;
mod sandbox;

use sandbox::Sandbox;

use args::Args;
use client_lib::{build_request, send_request, MAX_REQUEST_SIZE};
use firecracker::Firecracker;
use std::net::TcpStream;
use std::time::Duration;
use std::time::Instant;
use std::sync::Arc;
use log::{debug, error};
use tokio::time::sleep;

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
    debug!("Port {} is open after {} retries", port, retries);

    true
}


#[tokio::main]
async fn main() {
    let args: Args = Args::parse(std::env::args().collect()).unwrap();

    println!("SYSTEM,OP_TYPE,LATENCY_MICROSECONDS");

    for iteration in 0..args.iterations() { 
        let mut sandbox: Box<dyn Sandbox> = Box::new(Firecracker::new(args.config(), iteration));


        let system_name = sandbox.get_name();

        let presetup_time = Instant::now();
        sandbox.presetup().expect("Failed to presetup Firecracker VM");
        let elapsed_in_micros = presetup_time.elapsed().as_micros();
        println!("{},PRESETUP,{}", &system_name, elapsed_in_micros);

        let current_time = Instant::now();

        // Start the Firecracker VM
        match sandbox.start() {
            Ok(_) => {
                let found = wait_for_port(&sandbox.get_target_ip(), sandbox.get_target_port()).await;
                if found {
                    let elapsed_in_micros = current_time.elapsed().as_micros();
                    println!("{},START,{}", &system_name, elapsed_in_micros);
                } else {
                    error!("Failed to start Firecracker VM: Port {} is not open", sandbox.get_target_port());
                    sandbox.kill().expect("Failed to kill Firecracker VM");
                    sandbox.cleanup().expect("Failed to cleanup Firecracker VM");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to start Firecracker VM: {}", e);
                // exit the program with an error code
                std::process::exit(1);
            }
        }

        // Build the request
        if args.data_size() > MAX_REQUEST_SIZE {
            panic!("Request size is too large");
        }
        let request_data: Vec<u8> = vec![0u8; args.data_size()];
        let http_request: Arc<Vec<u8>> = Arc::new(build_request(request_data));

        // Send the request
        let address = format!("{}:{}", sandbox.get_target_ip(), sandbox.get_target_port());
        let latencies = match send_request(address, http_request, args.invocations()).await {
            Ok(latencies) => {
                debug!("Requests sents successfully");
                latencies
            }
            Err(e) => {
                eprintln!("Failed to send request: {}", e);
                sandbox.kill().expect("Failed to kill Firecracker VM");
                sandbox.cleanup().expect("Failed to cleanup Firecracker VM");
                std::process::exit(1);
            }
        };

        println!("{},FIRST_EXECUTION,{}", &system_name, latencies[0]);
        // Print the latencies
        for latency in &latencies[1..] {
            println!("{},EXECUTION,{}", &system_name, latency);
        }

        // Kill the Firecracker VM
        sandbox.kill().expect("Failed to kill Firecracker VM");

        // Cleanup the Firecracker VM
        sandbox.cleanup().expect("Failed to cleanup Firecracker VM");
    }
}
