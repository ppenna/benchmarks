// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

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
    future::Future,
    pin::Pin,
};

//==================================================================================================
// Structures
//==================================================================================================

#[derive(Deserialize)]
struct MessageJson {
    data: Vec<u8>,
}

pub struct HttpService {}

impl HttpService {
    pub fn new() -> Self {
        Self {}
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

    async fn serve(request: MessageJson) -> Result<Vec<u8>> {
        // Copy the request into a Vec<u8>
        Ok(request.data.clone())
    }
}

impl Service<Request<Incoming>> for HttpService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Incoming>) -> Self::Future {
        let future = async move {
            let body: Bytes = match request.collect().await {
                Ok(body) => body.to_bytes(),
                Err(_) => {
                    let reason: String = "failed to read body".to_string();
                    error!("{}", reason);
                    return Ok(Self::internal_server_error());
                },
            };

            // Deserialize the JSON directly into the struct
            let request: MessageJson = match serde_json::from_slice(body.as_ref()) {
                Ok(request) => request,
                Err(_) => {
                    let reason: String = "failed to deserialize JSON".to_string();
                    error!("{}", reason);
                    return Ok(Self::bad_request());
                },
            };

            // For now this is just copying the request data into the response.
            let bytes: Vec<u8> = match Self::serve(request).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    warn!("failed to serve request ({:?})", e);
                    return Ok(Self::internal_server_error());
                },
            };

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
