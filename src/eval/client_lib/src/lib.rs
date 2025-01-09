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

pub async fn send_request(sockaddr: String, http_request: Arc<Vec<u8>>) -> Result<u128> {

    let now: Instant = std::time::Instant::now();
    let mut stream: TcpStream = TcpStream::connect(sockaddr).await?;
    debug!("connected to server");
    stream.write_all(&http_request).await?;
    let mut response: Vec<u8> = vec![0u8; MAX_RESPONSE_SIZE];

    // Parse response.
    loop {
        match stream.read(&mut response).await {
                // Succeeded to read from socket.
            Ok(n) => {
                // Check if connection was closed.
                if n == 0 {
                    anyhow::bail!("Connection closed by server");
                }
                // Try to read the response
                let reader = tokio::io::BufReader::new(&response[..n] as &[u8]);
                let mut lines = reader.lines();
                // Consume all lines.
                while let Some(line) = lines.next_line().await? {
                    // Check if are done parsing the response.
                    if line.is_empty() {
                        let elapsed: u128 = now.elapsed().as_nanos();
                        stream.shutdown().await?;
                        debug!("disconnected from server");
                        return Ok(elapsed);
                    }
                }
            },
            // Failed to read from socket.
            Err(e) => {
                anyhow::bail!("failed to read from socket: {}", e);
            },
        }
    }
}