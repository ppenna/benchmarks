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
use ::tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt},
    net::TcpStream,
};
use log::debug;
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

pub async fn send_request(sockaddr: String, http_request: Arc<Vec<u8>>, total_invocations: u32) -> Result<Vec<u128>> {
    let mut latencies: Vec<u128> = Vec::with_capacity(total_invocations.try_into().unwrap());
    let mut stream: TcpStream = TcpStream::connect(sockaddr).await?;
    debug!("connected to server");

    // Parse response.
    for _ in 0..total_invocations {
        let now: Instant = std::time::Instant::now();
        // Send request
        stream.write_all(&http_request).await?;
        let mut response: Vec<u8> = vec![0u8; MAX_RESPONSE_SIZE];

        // Receive response
        let n = stream.read(&mut response).await?;
        let reader = tokio::io::BufReader::new(&response[..n] as &[u8]);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                let elapsed: u128 = now.elapsed().as_micros();
                latencies.push(elapsed);
                break;
            }
        }
    }

    stream.shutdown().await?;
    debug!("disconnected from server");
    Ok(latencies)
}