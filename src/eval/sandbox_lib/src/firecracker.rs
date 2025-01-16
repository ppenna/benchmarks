use crate::sandbox::Sandbox;
use anyhow::Result; 
use log::debug;
use serde::Deserialize;
use std::{fs::File, io::{Read, Write}, process::{Child, Command}, str};
use uuid::Uuid;


#[derive(Deserialize)]
struct FirecrackerConfig {
    firecracker_binary_dir: String,
    firecracker_socket_prefix: String,
    config_file_template: String,
    network_setup_file: String,
    network_cleanup_file: String,
}

pub struct Firecracker {
    id: String,
    config: FirecrackerConfig,
    child_process: Option<Child>,
    iteration: usize,
    log_location: String,
    vm_config_location: String,
}

impl Firecracker {
    pub fn new(config_path: &str, iteration: usize) -> Self {
        // Open file
        let file = std::fs::File::open(config_path).expect("Failed to open config file");
        let config: FirecrackerConfig = serde_json::from_reader(file)
            .expect("Failed to load config file");

        let id = Uuid::new_v4().to_string();
        let log_location = Self::create_log_file(&config.firecracker_binary_dir, &id).unwrap();

        Firecracker {
            config,
            child_process: None,
            id,
            iteration,
            log_location,
            vm_config_location: "".to_string(),
        }
    }

    pub fn get_gateway_ip(&self) -> String {
        "172.16.0.1".to_string()
    }

    fn get_prefixes(&self, offset: usize) -> (u8, u8) {
        if offset >= 256 * 256 {
            panic!("Too many iterations");
        }
        let last_prefix = offset % 256;
        let second_prefix = last_prefix / 256;
        (second_prefix as u8, last_prefix as u8)
    }

    pub fn get_tap_ip(&self) -> String {
        let ip_offset = 1 + 2*self.iteration;
        let (second_to_last_prefix, last_prefix) = self.get_prefixes(ip_offset);
        format!("172.16.{}.{}", second_to_last_prefix, last_prefix)
    }

    pub fn get_mac_address(&self) -> String {
        let offset= 2 + 2 * self.iteration;
       let (second_to_last_prefix, last_prefix) = self.get_prefixes(offset);
        format!("06:00:AC:10:{:02x}:{:02x}",second_to_last_prefix, last_prefix)
    }

    fn create_log_file(firecracker_binary_dir: &str, id: &str) -> Result<String> {
        let log_file = format!("{}/firecracker_{}.log", firecracker_binary_dir, id);
        let execution_log = Command::new("touch")
            .arg(&log_file)
            .output()
            .expect("Failed to create log file");
        debug!("Log file created with output: {:?}", execution_log);
        Ok(log_file)
    }

    fn create_firecracker_process_log(&self, suffix: &str) -> Result<File> {
        let log_file = format!("/tmp/firecracker_{}_{}.{}", self.id, self.iteration, suffix);
        let log = File::create(log_file).expect("failed to open log");
        Ok(log)
    }

    fn create_vm_config(&mut self) -> Result<()> {
        let vm_location = format!("{}/vm_config_{}.json", self.config.firecracker_binary_dir, self.id);

        // Open the template file
        let mut template_file = std::fs::File::open(&self.config.config_file_template).expect("Failed to open template file");
        // Read file to template
        let mut template = String::new();
        template_file.read_to_string(&mut template).expect("Failed to read template file");
        // Rewrite the template file with the correct values for {{guest_ip}}, {{tap_ip}}, {{tap_id}}, {{mac_address}}, and {{firecracker_log_location}}
        let result = template
            .replace("{{guest_ip}}", &self.get_target_ip())
            .replace("{{tap_ip}}", &self.get_tap_ip())
            .replace("{{tap_id}}", &format!("tap{}", self.iteration))
            .replace("{{mac_address}}", &self.get_mac_address())
            .replace("{{firecracker_log_location}}", &self.log_location);

        // Write the result to the vm_location
        let mut vm_file = std::fs::File::create(&vm_location).expect("Failed to create VM config file");
        vm_file.write_all(result.as_bytes()).expect("Failed to write to VM config file");
        vm_file.flush().expect("Failed to flush VM config file");
        self.vm_config_location = vm_location.clone(); 

        Ok(())
    }
}

impl Sandbox for Firecracker {
    fn presetup(&mut self) -> Result<()> {
        self.create_vm_config()?;
        // Run the command in self.config.network_setup_file
        let tap_device = format!("tap{}", self.iteration);
        let execution_setup= Command::new("sh")
            .arg("-c")
            .arg(format!("{} {} {} {}", &self.config.network_setup_file, &tap_device, &self.get_tap_ip(), &self.get_target_ip()))
            .output()
            .expect("Failed to execute network setup script");
        debug!("Network setup script executed with output: {} and error: {}", str::from_utf8(&execution_setup.stdout)?, str::from_utf8(&execution_setup.stderr)?);
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        // Run the command in self.config.network_setup_file
        let execution_cleanup= Command::new(&self.config.network_cleanup_file)
            .arg(format!("tap{}", self.iteration))
            .arg(&self.get_target_ip())
            .output().expect("Failed to execute network cleanup script");
        debug!("Network cleanup script executed with output: {} and error {}", str::from_utf8(&execution_cleanup.stdout)?, str::from_utf8(&execution_cleanup.stderr)?); 
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let socket_addr = format!("{}{}.socket",&self.config.firecracker_socket_prefix, self.id);

        debug!("Using socket address {}", socket_addr);

        let output_file = self.create_firecracker_process_log("out")?;
        let error_file = self.create_firecracker_process_log("err")?;

        let firecracker_args: Vec<String> = vec![
            format!("{}/firecracker", self.config.firecracker_binary_dir.clone()),
            "--config-file".to_string(),
            self.vm_config_location.clone(),
            "--api-sock".to_string(),
            socket_addr
        ];

        // Print the command we're going to run
        debug!("Starting Firecracker VM with command: {:?}", firecracker_args);

        // Execute the program and send the output to /dev/null
        let firecracker_process = Command::new(&firecracker_args[0])
            .args(&firecracker_args[1..])
            .current_dir(&self.config.firecracker_binary_dir)
            .stdout(output_file)
            .stderr(error_file)
            .spawn()?;

        // Waiting for Firecracker to finish setup (in a real application, you'd likely handle this better)
        debug!("Started Firecracker VM with PID: {}", firecracker_process.id());
        self.child_process = Some(firecracker_process);
        Ok(())
    }

    fn kill(&mut self) -> Result<()> {
        self.child_process.as_mut().unwrap().kill().expect("Failed to kill Firecracker VM");
        Ok(())
    }

    fn get_target_ip(&self) -> String {
        let ip_offset= 2 + 2*self.iteration;
        let (second_to_last_prefix, last_prefix) = self.get_prefixes(ip_offset);
        format!("172.16.{}.{}", second_to_last_prefix, last_prefix)
    }

    fn get_target_port(&self) -> u16 {
        8080
    }

    fn get_name(&self) -> String {
        "Firecracker".to_string()
    }
}