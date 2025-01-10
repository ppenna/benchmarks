use crate::sandbox::Sandbox;
use anyhow::Result; 
use log::debug;
use serde::Deserialize;
use std::{process::{Child, Command}, str};
use uuid::Uuid;
use std::fs::File;

#[derive(Deserialize)]
struct FirecrackerSnapshotConfig {
    firecracker_binary_dir: String,
    start_snapshot_script: String,
    firecracker_socket_prefix: String,
    snapshot_file: String,
    mem_file: String,
    network_setup_file: String,
    network_cleanup_file: String,
    output_dir: String,
}

pub struct FirecrackerSnapshot {
    id: String,
    config: FirecrackerSnapshotConfig,
    child_process: Option<Child>,
}

impl FirecrackerSnapshot {
    pub fn new(config_path: &str) -> Self {
        // Open file
        let file = std::fs::File::open(config_path).expect("Failed to open config file");
        let config: FirecrackerSnapshotConfig = serde_json::from_reader(file)
            .expect("Failed to load config file");

        let id = Uuid::new_v4().to_string();

        FirecrackerSnapshot {
            id,
            config,
            child_process: None,
        }
    }

    fn get_tap_ip(&self) -> String {
        "172.16.0.1".to_string()
    }

    fn create_log_file(output_dir: &str, id: &str, iteration: u16, suffix: &str) -> Result<File> {
        let log_file = format!("{}/firecracker_snapshot{}-{}{}", output_dir, id, iteration, suffix);
        let log = File::create(log_file).expect("failed to open log");
        Ok(log)
    }
}

impl Sandbox for FirecrackerSnapshot {
    fn presetup(&mut self) -> Result<()> {
        // Run the command in self.config.network_setup_file
        let tap_device = "tap0";
        let execution_setup= Command::new("sh")
            .arg("-c")
            .arg(format!("{} {} {}", &self.config.network_setup_file, tap_device, &self.get_tap_ip()))
            .output()
            .expect("Failed to execute network setup script");
        debug!("Network setup script executed with output: {} and error: {}", str::from_utf8(&execution_setup.stdout)?, str::from_utf8(&execution_setup.stderr)?);
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        // Run the command in self.config.network_setup_file
        let execution_cleanup= Command::new(&self.config.network_cleanup_file)
            .arg("tap0")
            .output().expect("Failed to execute network cleanup script");
        debug!("Network cleanup script executed with output: {} and error {}", str::from_utf8(&execution_cleanup.stdout)?, str::from_utf8(&execution_cleanup.stderr)?); 
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let socket_addr = format!("{}{}.socket",&self.config.firecracker_socket_prefix, self.id);

        debug!("Using socket address {}", socket_addr);

        let firecracker_args: Vec<String> = vec![
            format!("{}/firecracker", self.config.firecracker_binary_dir.clone()),
            "--api-sock".to_string(),
            socket_addr.clone(),
        ];

        // Print the command we're going to run
        debug!("Starting Firecracker VM with command: {:?}", firecracker_args);

        // Execute the program and send the output to /dev/null
        let stdout_file = Self::create_log_file(&self.config.output_dir, &self.id, 0, ".out")?;
        let stderr_file = Self::create_log_file(&self.config.output_dir, &self.id, 0, ".err")?;

        let firecracker_process = Command::new(&firecracker_args[0])
            .args(&firecracker_args[1..])
            .current_dir(&self.config.firecracker_binary_dir)
            .stdout(stdout_file)
            .stderr(stderr_file)
            .spawn()?;

        // Waiting for Firecracker to finish setup (in a real application, you'd likely handle this better)
        debug!("Started Firecracker VM with PID: {}", firecracker_process.id());

        // Send http request to unix socket to load snapshot
        let load_snapshot= Command::new("sh")
            .arg("-c")
            .arg(format!("{} {} {} {}", &self.config.start_snapshot_script, &socket_addr, &self.config.snapshot_file, &self.config.mem_file))
            .output()
            .expect("Failed to execute load snapshot");

        debug!("Snapshot loaded with output: {} and error: {}", str::from_utf8(&load_snapshot.stdout)?, str::from_utf8(&load_snapshot.stderr)?);

        self.child_process = Some(firecracker_process);
        Ok(())
    }

    fn get_target_ip(&self) -> String {
        "172.16.0.2".to_string()
    }

    fn get_target_port(&self) -> u16 {
        8080
    }

    fn get_name(&self) -> String {
        "Firecracker-Snapshot".to_string()
    }

    fn kill(&mut self) -> Result<()> {
        self.child_process.as_mut().unwrap().kill().expect("Failed to kill Firecracker VM");
        Ok(())
    }
}