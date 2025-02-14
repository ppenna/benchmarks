use crate::sandbox::Sandbox;
use anyhow::Result;
use log::debug;
use serde::Deserialize;
use std::{
    fs::File,
    process::{
        Child,
        Command,
    },
    str,
};
use uuid::Uuid;

#[derive(Deserialize)]
struct ProcessConfig {
    ip: String,
    port: u16,
    binary_path: String,
    output_dir: String,
}

pub struct Process {
    id: String,
    config: ProcessConfig,
    child_process: Option<Child>,
    iteration: usize,
}

impl Process {
    pub fn new(config_path: &str, iteration: usize) -> Self {
        // Open file
        // Append pwd to config_path
        let config_path = format!("{}/{}", std::env::current_dir().unwrap().display(), config_path);
        let file = std::fs::File::open(config_path).expect("Failed to open config file");
        let mut config: ProcessConfig =
            serde_json::from_reader(file).expect("Failed to load config file");

        // Update the port based on the iteration
        config.port += iteration as u16;

        let id = Uuid::new_v4().to_string();

        Process {
            id,
            config,
            child_process: None,
            iteration,
        }
    }

    fn create_log_file(
        process_output_dir: &str,
        id: &str,
        iteration: usize,
        suffix: &str,
    ) -> Result<File> {
        let log_file = format!("{}/process{}-{}{}", process_output_dir, id, iteration, suffix);
        let log = File::create(log_file).expect("failed to open log");
        Ok(log)
    }
}

impl Sandbox for Process {
    fn presetup(&mut self) -> Result<()> {
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let log_file_out =
            Self::create_log_file(&self.config.output_dir, &self.id, self.iteration, ".out")
                .unwrap();
        let log_file_err =
            Self::create_log_file(&self.config.output_dir, &self.id, self.iteration, ".err")
                .unwrap();

        let socket_addr = format!("{}:{}", &self.config.ip, self.config.port);

        debug!("Using socket address {}", socket_addr);

        let binary_path: String = self.config.binary_path.clone();

        let firecracker_args: Vec<String> = vec![binary_path, "-listen".to_string(), socket_addr];

        // Print the command we're going to run
        debug!("Starting Process sandbox with command: {:?}", firecracker_args);

        // Execute the program and send the output to /dev/null
        let process = Command::new(&firecracker_args[0])
            .args(&firecracker_args[1..])
            .stdout(log_file_out)
            .stderr(log_file_err)
            .spawn()?;

        // Waiting for Firecracker to finish setup (in a real application, you'd likely handle this better)
        debug!("Started Process sandbox with PID: {}", process.id());
        self.child_process = Some(process);
        Ok(())
    }

    fn kill(&mut self) -> Result<()> {
        self.child_process
            .as_mut()
            .unwrap()
            .kill()
            .expect("Failed to kill Process Sandbox");
        Ok(())
    }

    fn get_target_ip(&self) -> String {
        self.config.ip.clone()
    }

    fn get_target_port(&self) -> u16 {
        self.config.port
    }

    fn get_name(&self) -> String {
        "Process".to_string()
    }
}
