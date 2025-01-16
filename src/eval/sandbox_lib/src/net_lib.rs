use std::{net::TcpStream, thread::sleep, time::Duration};
use log::debug;

pub fn check_port(ip: &str, port: u16) -> bool {
    let address = format!("{}:{}", ip, port);
    TcpStream::connect_timeout(&address.parse().unwrap(), Duration::from_millis(1)).is_ok() 
}

pub fn wait_for_port(ip: &str, port: u16) -> bool {
    let max_retries = 10000;
    let mut retries = 0;
    debug!("Waiting for port {} in ip {} to open", port, ip);
    while !check_port(ip, port) {
        sleep(Duration::from_millis(1));
        retries += 1;
        if retries > max_retries {
            debug!("Port {} is not open after {} retries", port, retries);
            return false;
        }
    }
    debug!("Port {} is open after {} retries", port, retries);

    true
}

