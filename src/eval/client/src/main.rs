// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Configuration
//==================================================================================================

#![deny(clippy::all)]

//==================================================================================================
// Modules
//==================================================================================================

mod args;

//==================================================================================================
// Imports
//==================================================================================================

// Must come first.
#[macro_use]
extern crate log;

use self::args::Args;
use client_lib::{build_request, send_request, MAX_REQUEST_SIZE};
use ::anyhow::Result; 
use ::flexi_logger::Logger;
use ::std::{
    env,
    sync::{
        atomic::AtomicUsize,
        Arc,
        Once,
    },
    thread,
    time::{
        Duration,
        Instant,
    },
};
use ::tokio::sync::{
    mpsc,
    Mutex,
};
use tokio::task::JoinHandle;

//==================================================================================================
// Constants
//==================================================================================================


//==================================================================================================
// Standalone Functions
//==================================================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging system.
    initialize();

    // Parse and retrieve command-line arguments.
    let args: Args = Args::parse(env::args().collect())?;
    let frequency: u128 = args.frequency();
    let duration: u64 = args.duration();
    let sockaddr: String = args.connect_sockaddr();
    let size: usize = args.size();

    // Check if request size is valid.
    if size > MAX_REQUEST_SIZE {
        anyhow::bail!("request size is too large (MAX_REQUEST_SIZE={:?})", size);
    }

    let (stop_tx, stop_rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel(1);

    let data: Vec<u8> = vec![0u8; size];

    let sockaddr: String = sockaddr.clone();
    let http_request: Arc<Vec<u8>> = Arc::new(build_request(data));
    let thread =
        tokio::spawn(async move { client(sockaddr, http_request, frequency, stop_rx).await });

    thread::sleep(Duration::from_secs(duration));

    // Stop all threads.
    if let Err(e) = stop_tx.send(true).await {
        anyhow::bail!("failed to send stop signal: {}", e);
    }
    let mut latencies: Vec<u64> = thread.await??;

    // Compute statistics from latencies.
    latencies.sort();
    let p50_index: usize = ((latencies.len() * 50) / 100).max(1) - 1;
    let p99_index: usize = ((latencies.len() * 99) / 100).max(1) - 1;

    let p50: u64 = latencies[p50_index];
    let p99: u64 = latencies[p99_index];

    println!("{:?},{:?},{:?},{:?},{:?}", frequency, duration, latencies.len(), p50, p99);

    Ok(())
}

///
/// # Description
///
/// Initializes the logger.
///
/// # Note
///
/// If the logger cannot be initialized, the function will panic.
///
pub fn initialize() {
    static INIT_LOG: Once = Once::new();
    INIT_LOG.call_once(|| {
        Logger::try_with_env()
            .expect("malformed RUST_LOG environment variable")
            .start()
            .expect("failed to initialize logger");
    });
}

///
/// # Description
///
/// This asynchronous function sends HTTP requests to a specified remote server at a regular
/// interval defined by `frequency` nanoseconds.  For each request, a new asynchronous task is
/// spawned. This task performs the following steps:
///
///   1. Connects to the server using the provided `sockaddr`.
///   2. Sends the specified `http_request`.
///   3. Waits for the server's response.
///
/// The latency of each request is measured and stored in the `latencies` vector.
///
/// # Parameters
///
/// - `sockaddr`: The address of the server to which the requests will be sent.
/// - `http_request`: The HTTP request to be sent to the server.
/// - `frequency`: The interval, in nanoseconds, between consecutive requests.
/// - `stop_rx`: A receiver used to signal the client to stop sending requests.
///
/// # Returns
///
/// A vector containing the latency of each request in nanoseconds.
///
async fn client(
    sockaddr: String,
    http_request: Arc<Vec<u8>>,
    frequency: u128,
    mut stop_rx: mpsc::Receiver<bool>,
) -> Result<Vec<u64>, anyhow::Error> {
    let latencies: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::with_capacity(2 ^ 16)));

    // Send first request.
    let mut stop_sending: bool = false;
    let mut last_sent: Instant = std::time::Instant::now();
    let nrequests: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    loop {
        if stop_sending {
            debug!("stopping client...");
            debug!("waiting tasks to finish...");
            // TODO: cancel tasks.
            for handle in handles {
                if let Err(e) = handle.await? {
                    error!("failed to join handle: {}", e);
                }
            }
            debug!("stopped!");
            return Ok(latencies.lock().await.clone());
        } else if last_sent.elapsed().as_nanos() >= frequency {
            let http_request_clone: Arc<Vec<u8>> = http_request.clone();
            let sockaddr_clone: String = sockaddr.clone();
            let requests_clone: Arc<AtomicUsize> = nrequests.clone();
            let latencies_clone: Arc<Mutex<Vec<u64>>> = latencies.clone();

            // Spawn a new asynchronous task.
            let handle: JoinHandle<std::result::Result<(), anyhow::Error>> =
                tokio::spawn(async move {
                    match send_request(sockaddr_clone, http_request_clone, 1).await {
                        Ok(elapsed) => {
                            latencies_clone.lock().await.push(elapsed[0] as u64);
                            requests_clone
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            debug!("elapsed: {} ns", elapsed[0]);
                            Ok(())
                        },
                        Err(_) => {
                            anyhow::bail!("Connection closed by server");
                        },
                    }

                });

            handles.push(handle);

            last_sent = std::time::Instant::now();
        }

        // Check if we should stop sending requests.
        if stop_rx.try_recv().is_ok() {
            stop_sending = true;
        }
    }
}