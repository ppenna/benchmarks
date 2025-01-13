use crate::sandbox::Sandbox;
use anyhow::Result; 
use log::debug;
use serde::Deserialize;
use std::{process::{Child, Command, Stdio}, str};
use uuid::Uuid;
use std::fs::File;


#[derive(Deserialize)]
struct UnikraftConfig {
    guest_port: u16,
    host_port: u16,
    run_dir: String,
    memory: String,
    output_dir: String
}

pub struct Unikraft {
    id: String,
    config: UnikraftConfig,
    child_process: Option<Child>,
    iteration: u16,
}

impl Unikraft {
    pub fn new(config_path: &str, iteration: u16) -> Self {
        // Open file
        let file = std::fs::File::open(config_path).expect("Failed to open config file");
        let config: UnikraftConfig = serde_json::from_reader(file)
            .expect("Failed to load config file");

        let id = Uuid::new_v4().to_string();


        Unikraft {
            id,
            config,
            child_process: None,
            iteration,
        }
    }

    fn create_log_file(output_dir: &str, id: &str, iteration: u16, suffix: &str) -> Result<File> {
        let log_file = format!("{}/unikraft{}-{}{}", output_dir, id, iteration, suffix);
        let log = File::create(log_file).expect("failed to open log");
        Ok(log)
    }
}

impl Sandbox for Unikraft {
    fn start(&mut self) -> Result<()> {
        let log_file_out = Self::create_log_file(&self.config.output_dir, &self.id, self.iteration,".out").unwrap();
        let log_file_err = Self::create_log_file(&self.config.output_dir, &self.id, self.iteration, ".err").unwrap();
        let start_cmd = Command::new("kraft" )
        .arg("run")
        .arg("--rm")
        .arg("--plat")
        .arg("qemu")
        .arg("--arch")
        .arg("x86_64")
        .arg("-p")
        .arg(format!("{}:{}", self.config.host_port, self.config.guest_port))
        .arg("--memory")
        .arg(self.config.memory.clone())
        .arg(".")
        .current_dir(&self.config.run_dir)
        .stdout(log_file_out)
        .stderr(log_file_err)
        .spawn()?;

        debug!("Started Unikraft VM with PID: {}", start_cmd.id());
        self.child_process = Some(start_cmd);
        Ok(())
    }

    fn kill(&mut self) -> Result<()> {
        self.child_process.as_mut().unwrap().kill().expect("Failed to kill Unikraft VM");
        let _clean_cmd = Command::new("kraft" )
        .arg("rm")
        .arg("--all")
        .stdout(Stdio::null()) 
        .stderr(Stdio::null())
        .output()?;

        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    fn presetup(&mut self) -> Result<()> {
       Ok(()) 
    }

    fn get_name(&self) -> String {
        "Unikraft".to_string()
    }

    fn get_target_port(&self) -> u16 {
        self.config.host_port
    }

    fn get_target_ip(&self) -> String {
        "127.0.0.1".to_string()
    }


} 
