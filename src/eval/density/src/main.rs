mod args;
mod logging;

//==================================================================================================
// Imports
//==================================================================================================

use args::Args;
use anyhow::Result;
use sandbox_lib::{
    sandbox::Sandbox,
    firecracker::Firecracker, 
    firecracker_snapshot::FirecrackerSnapshot,
    process::Process,
    unikraft::Unikraft,
    hyperlight::Hyperlight,
    net_lib::wait_for_port,
};
use client_lib::{build_request, send_request, MAX_REQUEST_SIZE};
use log::{error, debug};
use serde::Deserialize;
use std::collections::VecDeque;
use std::time::Duration;
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

async fn clean_sandbox(sandbox: &mut Box<dyn Sandbox>) -> Result<()> {
    sandbox.kill().expect("Failed to kill sandbox");
    sandbox.cleanup().expect("Failed to cleanup sandbox");
    Ok(())
}

fn get_free_avail_mem() -> Result<u64> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("cat /proc/meminfo | grep MemFree")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mem_free_kb: u64 = output_str.split_whitespace().nth(1).unwrap().parse().unwrap();
    let mem_free_mb = mem_free_kb / 1024;

    Ok(mem_free_mb)
}

async fn send_single_request(sandbox: &mut Box<dyn Sandbox>) -> Result<()> {
    let address = format!("{}:{}", sandbox.get_target_ip(), sandbox.get_target_port());
    let request_data: Vec<u8> = vec![0u8; MAX_REQUEST_SIZE];
    let http_request: Arc<Vec<u8>> = Arc::new(build_request(request_data));

    debug!("Sending request to {}", address);

    match send_request(address, http_request, 1).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to send request: {}", e);
            return Err(e);
        }
    };

    Ok(())
}


async fn start_sandbox_and_wait_for_server(sandbox: &mut Box<dyn Sandbox>) -> Result<()> {
    let system_name = sandbox.get_name();

    sandbox.presetup().expect("Failed to presetup VM");

    match sandbox.start() {
        Ok(_) => {
            if sandbox.get_name() != "Firecracker" {
                let found = wait_for_port(&sandbox.get_target_ip(), sandbox.get_target_port());
                if !found {
                    return Err(anyhow::anyhow!("Failed to start VM"));
                }
            }
            else {
                // Wait for 2 s 
                sleep(Duration::from_secs(2)).await;
            }
        }
        Err(e) => {
            error!("Failed to start {} VM: {}", &system_name, e);
            // exit the program with an error code
            std::process::exit(1);
        }
    }

    if sandbox.get_name() != "Firecracker" {
        // Send a single request to the server
        send_single_request(sandbox).await?;
    }

    Ok(())
}


async fn init_sandbox(sandbox: &mut Box<dyn Sandbox>, iteration: usize) -> Result<u64> {
    let system_name = sandbox.get_name();

    match start_sandbox_and_wait_for_server(sandbox).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to create sandbox: {}", e);
            clean_sandbox(sandbox).await?;
            return Err(anyhow::anyhow!("Failed to create sandbox"));    
        }
    };

    // Get free memory

    let free_mem_mb = get_free_avail_mem()?;

    println!("{},FREE_MEM_MB,{},{}", system_name, iteration, free_mem_mb);

    Ok(free_mem_mb)
}


fn clean_caches() {
    std::process::Command::new("sh")
        .arg("-c")
        .arg("echo 3 | sudo tee /proc/sys/vm/drop_caches")
        .output()
        .expect("Failed to execute command");
}

#[tokio::main]
async fn main() {
    logging::initialize(false);

    let args: Args = Args::parse(std::env::args().collect()).unwrap();
    let file = std::fs::File::open(args.config()).expect("Failed to open main config file");
    let config: EvalsConfig = serde_json::from_reader(file).expect("Failed to load main config file");

    let mut sandbox_queue: VecDeque<Box<dyn Sandbox>> = VecDeque::new();

    println!("SYSTEM,OP_TYPE,ITERATION, FREE_MEMORY");

    for eval in &config.evals {
        let eval_type = EvalType::from_string(&eval.type_of_eval);
        clean_caches();
        let mut iteration = 0;
        loop {
            debug!("{},ITERATION,{}", eval.type_of_eval, iteration);
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

            // Keep creating sandboxes until it breaks
            let mem = match init_sandbox(&mut sandbox, iteration).await {
                Ok(mem) => mem,
                Err(_) => {
                    break;
                }
            };
            sandbox_queue .push_back(sandbox);

            // Break if the free memory is less than the memory limit (512 MB being the default) 
            if mem < args.memory_limit() {
                break;
            }

            iteration += 1;
        }

        // Clean all the sandboxes
        for mut sandbox in &mut sandbox_queue {
            clean_sandbox(&mut sandbox).await.expect("Failed to clean sandbox");
        }

        sandbox_queue.clear();

        // Wait for a while
        sleep(Duration::from_secs(5)).await;
    }
}
