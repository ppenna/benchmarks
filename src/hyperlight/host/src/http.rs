// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use crate::sandbox::Sandbox;
use ::anyhow::Result;
use ::http_body_util::{
    BodyExt,
    Full,
};
use ::hyper::{
    body::{
        Bytes,
        Incoming,
    },
    service::Service,
    Request,
    Response,
    StatusCode,
};
use ::serde::Deserialize;
use ::serde_json::Value;
use ::std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::Arc,
};
use ::tokio::sync::Mutex;

//==================================================================================================
// Structures
//==================================================================================================

#[derive(Deserialize)]
struct MessageJson {
    data: Vec<u8>,
}

pub struct HttpServer {
    sandbox_file_path: String,
    ready_sandboxes: Arc<Mutex<VecDeque<Sandbox>>>,
}

impl HttpServer {
    pub fn new(sandbox_file_path: String, init_num_sandboxes: usize) -> Self {
        let mut ready_sanboxes = VecDeque::new();
        for _ in 0..init_num_sandboxes {
            let sandbox = Self::create_sandbox(&sandbox_file_path).unwrap();
            ready_sanboxes.push_back(sandbox);
        }
        let ready_sandboxes: Arc<Mutex<VecDeque<Sandbox>>> = Arc::new(Mutex::new(ready_sanboxes));

        Self {
            sandbox_file_path,
            ready_sandboxes,
        }
    }

    pub async fn add_sandbox(
        ready_sandboxes: Arc<Mutex<VecDeque<Sandbox>>>,
        sandbox_path: &str,
    ) -> Result<()> {
        let mut locked_sandboxes = ready_sandboxes.lock().await;
        match Self::create_sandbox(sandbox_path) {
            Ok(sandbox) => {
                locked_sandboxes.push_back(sandbox);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    fn create_sandbox(sandbox_path: &str) -> Result<Sandbox> {
        let mut sandbox: Sandbox = Sandbox::new(sandbox_path);
        match sandbox.init() {
            Ok(_) => Ok(sandbox),
            Err(e) => {
                let reason: String = format!("failed to initialize sandbox ({:?})", e);
                error!("{}", reason);
                Err(anyhow::anyhow!(reason))
            },
        }
    }

    ///
    /// # Description
    ///
    /// Helper function that creates a "bad request" response.
    ///
    /// # Returns
    ///
    /// A "bad request" response.
    ///
    fn bad_request() -> Response<Full<Bytes>> {
        let mut bad_request: Response<Full<Bytes>> = Response::new(Full::new(Bytes::new()));
        *bad_request.status_mut() = hyper::StatusCode::BAD_REQUEST;
        bad_request
    }

    ///
    /// # Description
    ///
    /// Helper function that creates an "internal server error" response.
    ///
    /// # Returns
    ///
    /// An "internal server error" response.
    ///
    fn internal_server_error() -> Response<Full<Bytes>> {
        let mut internal_server_error: Response<Full<Bytes>> =
            Response::new(Full::new(Bytes::new()));
        *internal_server_error.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
        internal_server_error
    }

    async fn serve(sandbox: &mut Sandbox, request: MessageJson) -> Result<Vec<u8>> {
        Ok(sandbox.run(request.data).unwrap())
    }
}

impl Service<Request<Incoming>> for HttpServer {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Incoming>) -> Self::Future {
        let sandbox_path_copy = self.sandbox_file_path.clone();
        let ready_sandboxes = self.ready_sandboxes.clone();

        let future = async move {
            let body: Bytes = match request.collect().await {
                Ok(body) => body.to_bytes(),
                Err(_) => {
                    let reason: String = "failed to read body".to_string();
                    error!("{}", reason);
                    return Ok(Self::internal_server_error());
                },
            };

            // If the body is empty it is meant to be a pre-creation of a sandbox
            if body.is_empty() {
                match Self::add_sandbox(ready_sandboxes, &sandbox_path_copy).await {
                    Ok(_) => {
                        return Ok(Response::builder()
                            .status(StatusCode::NO_CONTENT)
                            .body(Full::new(Bytes::new()))
                            .unwrap());
                    },
                    Err(_) => {
                        return Ok(Self::internal_server_error());
                    },
                }
            }

            // Check if a sandbox is ready to be used
            let mut sandbox: Sandbox;
            {
                // Lock scope
                let mut locked_sandboxes = ready_sandboxes.lock().await;
                sandbox = match locked_sandboxes.pop_front() {
                    Some(sandbox) => sandbox,
                    None => match Self::create_sandbox(&sandbox_path_copy) {
                        Ok(sandbox) => sandbox,
                        Err(_) => {
                            return Ok(Self::internal_server_error());
                        },
                    },
                };
            }

            // Deserialize the JSON directly into the struct
            let request: MessageJson = match serde_json::from_slice(body.as_ref()) {
                Ok(request) => request,
                Err(_) => {
                    let reason: String = "failed to deserialize JSON".to_string();
                    error!("{}", reason);
                    return Ok(Self::bad_request());
                },
            };

            let bytes: Vec<u8> = match Self::serve(&mut sandbox, request).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    warn!("failed to serve request ({:?})", e);
                    return Ok(Self::internal_server_error());
                },
            };

            // Return sandbox to the queue
            {
                // Lock scope
                let mut locked_sandboxes = ready_sandboxes.lock().await;
                locked_sandboxes.push_back(sandbox);
            }

            let json: Value = serde_json::json!({
                "response": String::from_utf8_lossy(&bytes).to_string(),
            });

            // Convert JSON to bytes.
            let bytes = match serde_json::to_vec(&json) {
                Ok(bytes) => Bytes::from(bytes),
                Err(_) => {
                    let reason: String = "failed to convert JSON to bytes".to_string();
                    error!("{}", reason);
                    return Ok(Self::internal_server_error());
                },
            };

            match Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("Content-Length", bytes.len())
                .body(Full::new(bytes))
            {
                Ok(response) => Ok(response),
                Err(_) => {
                    let reason: String = "failed to build response".to_string();
                    error!("{}", reason);
                    Ok(Self::internal_server_error())
                },
            }
        };
        Box::pin(future)
    }
}
