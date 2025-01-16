// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use ::anyhow::Result;
use ::serde_json::{
    json,
    Value,
};
use ::tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt};
use log::debug;
use std::io::{BufRead, Read, Write};
use std::sync::Arc;
use std::time::Instant;

//==================================================================================================
// Constants
//==================================================================================================
const MAX_RESPONSE_SIZE: usize = 1024;
pub const MAX_REQUEST_SIZE: usize = 1024;

//==================================================================================================
// Functions
//================================================================================================== 

pub fn build_request(data: Vec<u8>) -> Vec<u8> {
    let json_obj: Value = json!({
        "data": data,
    });

    format!(
        "POST / HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        json_obj.to_string().len(),
        json_obj
    )
    .as_bytes()
    .to_vec()
}

pub fn build_empty_request() -> Vec<u8> {
    "POST / HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 0\r\n\r\n"
        .as_bytes()
        .to_vec()
}

pub async fn send_request(sockaddr: String, http_request: Arc<Vec<u8>>, total_invocations: u32) -> Result<Vec<u128>> {
    let mut latencies: Vec<u128> = Vec::with_capacity(total_invocations.try_into().unwrap());
    debug!("connected to server");

    // Parse response.
    for _ in 0..total_invocations {
        let mut stream: tokio::net::TcpStream = tokio::net::TcpStream::connect(&sockaddr).await?;
        let now: Instant = std::time::Instant::now();
        // Send request
        stream.write_all(&http_request).await?;
        let mut response: Vec<u8> = vec![0u8; MAX_RESPONSE_SIZE];

        // Receive response
        let n = stream.read(&mut response).await?;
        if n == 0 {
            return Err(anyhow::anyhow!("Failed to read response"));
        }
        let reader = tokio::io::BufReader::new(&response[..n] as &[u8]);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                let elapsed: u128 = now.elapsed().as_micros();
                latencies.push(elapsed);
                stream.shutdown().await?;
                break;
            }
        }
    }

    debug!("disconnected from server");
    Ok(latencies)
}

pub fn sync_send_request(sockaddr: String, http_request: Arc<Vec<u8>>, total_invocations: u32) -> Result<Vec<u128>> {
    // Use std::net::TcpStream to send the request
    let mut latencies: Vec<u128> = Vec::with_capacity(total_invocations.try_into().unwrap());
    let mut stream: std::net::TcpStream = std::net::TcpStream::connect(sockaddr)?;
    debug!("connected to server");

    for _ in 0..total_invocations {
        let now: Instant = std::time::Instant::now();
        // Send request
        stream.write_all(&http_request)?;
        let mut response: Vec<u8> = vec![0u8; MAX_RESPONSE_SIZE];

        // Receive response
        let n = stream.read(&mut response)?;
        let reader = std::io::BufReader::new(&response[..n] as &[u8]);
        for line in reader.lines() {
            if line?.is_empty() {
                let elapsed: u128 = now.elapsed().as_micros();
                latencies.push(elapsed);
                break;
            }
        }
    }

    stream.shutdown(std::net::Shutdown::Both)?;
    debug!("disconnected from server");
    Ok(latencies)
}
