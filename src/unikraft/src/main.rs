use ::http_library::HttpService; 
use ::hyper::server::conn::http1;
use ::hyper_util::rt::TokioIo;
use ::tokio::net::{
    TcpListener,
    TcpStream,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listen_address = "0.0.0.0:8080";
    let http_listener: TcpListener = TcpListener::bind(listen_address).await?;
    loop {
        match http_listener.accept().await {
            Ok((stream, _sockaddr)) => {
                let client = HttpService::new();
                let io: TokioIo<TcpStream> = TokioIo::new(stream);
                if let Err(e) = http1::Builder::new().serve_connection(io, client).await  {
                    eprintln!("failed to serve connection ({:?})", e);
                }
            },
            Err(e) => {
                eprintln!("failed to accept connection ({:?})", e);
            },
        }
    }
}