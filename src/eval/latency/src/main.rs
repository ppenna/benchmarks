mod args;
mod firecracker;
mod firecracker_snapshot;
mod hyperlight;
mod net_lib;
mod process;
mod sandbox;
mod unikraft;

use args::Args;
use firecracker::Firecracker;
use firecracker_snapshot::FirecrackerSnapshot;
use hyperlight::Hyperlight;
use net_lib::wait_for_port;
use process::Process;
use sandbox::Sandbox;
use unikraft::Unikraft;

use client_lib::{build_request, send_request, MAX_REQUEST_SIZE};
use log::{debug, error};
use serde::Deserialize;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::time::sleep;

enum EvalType {
    Firecracker,
    FirecrackerSnapshot,
    Process,
    Unikraft,
    Hyperlight,
}

impl EvalType {
    fn from_string(s: &str) -> Self {
        match s {
            "firecracker" => EvalType::Firecracker,
            "firecracker-snapshot" => EvalType::FirecrackerSnapshot,
            "unikraft" => EvalType::Unikraft,
            "process" => EvalType::Process,
            "hyperlight" => EvalType::Hyperlight,
            _ => panic!("Invalid eval type"),
        }
    }
}

#[derive(Deserialize)]
struct EvalConfig {
    type_of_eval: String,
    config_location: String,
}

#[derive(Deserialize)]
struct EvalsConfig {
    evals: Vec<EvalConfig>,
}


async fn process_sandbox(sandbox: &mut Box<dyn Sandbox>, data_size: usize, total_invocations: u32) {
    let system_name = sandbox.get_name();

    let presetup_time = Instant::now();
    sandbox.presetup().expect("Failed to presetup VM");
    let elapsed_in_micros = presetup_time.elapsed().as_micros();
    println!("{},PRESETUP,{}", &system_name, elapsed_in_micros);

    // Wait for 2 s 
    sleep(Duration::from_secs(2)).await;

    let current_time = Instant::now();

    // Start the VM
    match sandbox.start() {
        Ok(_) => {
            let found = wait_for_port(&sandbox.get_target_ip(), sandbox.get_target_port());
            if found {
                let elapsed_in_micros = current_time.elapsed().as_micros();
                println!("{},SETUP_SANDBOX,{}", &system_name, elapsed_in_micros);
            } else {
                error!("Failed to start {} VM: Port {} is not open", &system_name,  sandbox.get_target_port());
                sandbox.kill().expect("Failed to kill VM");
                sandbox.cleanup().expect("Failed to cleanup VM");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to start {} VM: {}", &system_name, e);
            // exit the program with an error code
            std::process::exit(1);
        }
    }

    // Build the request
    if data_size > MAX_REQUEST_SIZE {
        panic!("Request size is too large");
    }
    let request_data: Vec<u8> = vec![0u8; data_size];
    let http_request: Arc<Vec<u8>> = Arc::new(build_request(request_data));

    // Send the request
    let address = format!("{}:{}", sandbox.get_target_ip(), sandbox.get_target_port());
    let latencies = match send_request(address, http_request, total_invocations).await {
        Ok(latencies) => {
            debug!("Requests sents successfully");
            latencies
        }
        Err(e) => {
            eprintln!("Failed to send request: {}", e);
            sandbox.kill().expect("Failed to kill VM");
            sandbox.cleanup().expect("Failed to cleanup VM");
            std::process::exit(1);
        }
    };

    println!("{},FIRST_EXECUTION,{}", &system_name, latencies[0]);
    // Print the latencies
    for latency in &latencies[1..] {
        println!("{},EXECUTION,{}", &system_name, latency);
    }

    // Kill the VM
    sandbox.kill().expect("Failed to kill VM");

    // Cleanup the VM
    sandbox.cleanup().expect("Failed to cleanup VM");

}


#[tokio::main]
async fn main() {
    let args: Args = Args::parse(std::env::args().collect()).unwrap();
    let file = std::fs::File::open(args.config()).expect("Failed to open main config file");
    let config: EvalsConfig = serde_json::from_reader(file).expect("Failed to load main config file");

    println!("SYSTEM,OP_TYPE,LATENCY_MICROSECONDS");

    for eval in &config.evals {
        let eval_type = EvalType::from_string(&eval.type_of_eval);
        for iteration in 0..args.iterations() { 
            let mut sandbox: Box<dyn Sandbox> = match eval_type {
                EvalType::Firecracker => {
                    Box::new(Firecracker::new(&eval.config_location, iteration))
                }
                EvalType::Unikraft => {
                    Box::new(Unikraft::new(&eval.config_location, iteration))
                }
                EvalType::Process => {
                    Box::new(Process::new(&eval.config_location, iteration))
                }
                EvalType::FirecrackerSnapshot => {
                    Box::new(FirecrackerSnapshot::new(&eval.config_location))
                }
                EvalType:: Hyperlight => {
                    Box::new(Hyperlight::new(&eval.config_location, iteration))
                }
            };

            process_sandbox(&mut sandbox, args.data_size(), args.invocations()).await;

            // Sleep for a bit to allow the VM to cleanup
            sleep(Duration::from_secs(2)).await;
        }
    }
}
