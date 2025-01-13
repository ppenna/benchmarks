// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

//==================================================================================================
// Modules
//==================================================================================================

mod args;
mod http;
mod logging;
mod sandbox;

//==================================================================================================
// Imports
//==================================================================================================

// Must come first.
#[macro_use]
extern crate log;

use crate::{
    args::Args,
    http::HttpServer,
};
use ::anyhow::Result;
use ::hyper::server::conn::http1;
use ::hyper_util::rt::TokioIo;
use ::tokio::{
    net::{
        TcpListener,
        TcpStream,
    },
    signal::unix::{
        signal,
        Signal,
        SignalKind,
    },
};

//==================================================================================================
// Standalone Functions
//==================================================================================================

#[tokio::main]
pub async fn main() -> Result<()> {
    logging::initialize(false);

    let args: Args = Args::parse(std::env::args().collect())?;

    let mut signals: Signal = signal(SignalKind::interrupt())?;
    let http_listener: TcpListener = TcpListener::bind(args.listen_sockaddr()).await?;

    loop {
        tokio::select! {
           result = http_listener.accept() => {
                match result {
                    Ok((stream, sockaddr)) => {
                        debug!("accepted connection from {:?}", sockaddr);
                        let filepath = args.guest().to_string();
                        let client =
                            HttpServer::new(filepath);
                        let io: TokioIo<TcpStream> = TokioIo::new(stream);
                        if let Err(e) = http1::Builder::new().serve_connection(io, client).await  {
                            error!("failed to serve connection ({:?})", e);
                        }
                    },
                    Err(e) => {
                        error!("failed to accept connection ({:?})", e);
                    },
                }
            },
            _ = signals.recv() => {
                info!("received exit signal, stopping...");
                break;
            },
        }
    }

    Ok(())
}
