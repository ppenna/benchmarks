use client_lib::{build_empty_request, sync_send_request};
use crate::net_lib::wait_for_port;
use crate::sandbox::Sandbox;
use anyhow::Result; 
use log::{debug, error};
use serde::Deserialize;
use std::{process::{Child, Command}, str, sync::Arc};
use uuid::Uuid;
use std::fs::File;


#[derive(Deserialize)]
struct HyperlightConfig {
    guest_binary: String,
    host_binary: String,
    listen_ip: String,
    listen_port: u16,
    output_dir: String
}

pub struct Hyperlight {
    id: String,
    config: HyperlightConfig,
    child_process: Option<Child>,
    iteration: u16,
}

impl Hyperlight {
    pub fn new(config_path: &str, iteration: u16) -> Self {
        // Open file
        let file = std::fs::File::open(config_path).expect("Failed to open config file");
        let config: HyperlightConfig = serde_json::from_reader(file)
            .expect("Failed to load config file");

        let id = Uuid::new_v4().to_string();


        Hyperlight {
            id,
            config,
            child_process: None,
            iteration,
        }
    }

    fn create_log_file(output_dir: &str, id: &str, iteration: u16, suffix: &str) -> Result<File> {
        let log_file = format!("{}/hyperlight{}-{}{}", output_dir, id, iteration, suffix);
        let log = File::create(log_file).expect("failed to open log");
        Ok(log)
    }
}

impl Sandbox for Hyperlight {
    fn start(&mut self) -> Result<()> {
        // Send empty request
        let empty_request = Arc::new(build_empty_request());
        let address = format!("{}:{}", self.get_target_ip(), self.get_target_port());
        match sync_send_request(address, empty_request, 1) {
            Ok(_) => debug!("Successfully sent empty request to Hyperlight VM"),
            Err(e) => error!("Failed to send empty request to Hyperlight VM: {}", e),
        }

        Ok(())
    }

    fn kill(&mut self) -> Result<()> {
        self.child_process.as_mut().unwrap().kill().expect("Failed to kill Unikraft VM");
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    fn presetup(&mut self) -> Result<()> {
        let log_file_out = Self::create_log_file(&self.config.output_dir, &self.id, self.iteration,".out").unwrap();
        let log_file_err = Self::create_log_file(&self.config.output_dir, &self.id, self.iteration, ".err").unwrap();
        let mut start_cmd = Command::new(&self.config.host_binary)
            .arg("-listen")
            .arg(format!("{}:{}", &self.config.listen_ip, &self.config.listen_port))
            .arg("-guest")
            .arg(&self.config.guest_binary)
            .stdout(log_file_out)
            .stderr(log_file_err)
            .spawn()?;

        let ready = wait_for_port(&self.config.listen_ip, self.config.listen_port);

        if !ready {
            error!("Failed to start Hyperlight VM: Port {} is not open", self.config.listen_port);
            start_cmd.kill().expect("Failed to kill Hyperlight VM");
            std::process::exit(1);
        }
        
        debug!("Started Hyperlight VM with PID: {}", start_cmd.id());
        self.child_process = Some(start_cmd);
       Ok(()) 
    }

    fn get_name(&self) -> String {
        "Hyperlight".to_string()
    }

    fn get_target_port(&self) -> u16 {
        self.config.listen_port
    }

    fn get_target_ip(&self) -> String {
        self.config.listen_ip.clone()
    }
} 
