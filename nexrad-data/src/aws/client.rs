//! Shared HTTP client for AWS operations.
//!
//! This module provides a singleton HTTP client with connection pooling for
//! efficient reuse across multiple S3 operations.

use crate::result::{AwsError, Error, Result};
use reqwest::Client;
use std::sync::OnceLock;

/// How long the native client waits to establish a connection.
#[cfg(not(target_arch = "wasm32"))]
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// How long the native client waits for the next bytes of a response before failing the request.
#[cfg(not(target_arch = "wasm32"))]
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// The longest a single native request may take, from sending it to reading the last byte of the
/// response body.
#[cfg(not(target_arch = "wasm32"))]
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// Returns a reference to the shared HTTP client.
///
/// The client is built on first use and reused for all subsequent requests. If building fails,
/// every call returns [`AwsError::HttpClient`].
///
/// On native targets, the client pools connections and applies [`CONNECT_TIMEOUT`],
/// [`READ_TIMEOUT`] and [`REQUEST_TIMEOUT`]. These options do not exist on wasm32, where requests
/// go through the browser's `fetch` and its own limits apply. Keep the `crate::aws` module docs in
/// step with these values.
pub(crate) fn client() -> Result<&'static Client> {
    static CLIENT: OnceLock<std::result::Result<Client, String>> = OnceLock::new();

    CLIENT
        .get_or_init(|| {
            #[allow(unused_mut)]
            let mut builder = Client::builder();

            // Connection pooling and timeouts are only available on native targets
            #[cfg(not(target_arch = "wasm32"))]
            {
                builder = builder
                    .pool_max_idle_per_host(4)
                    .connect_timeout(CONNECT_TIMEOUT)
                    .read_timeout(READ_TIMEOUT)
                    .timeout(REQUEST_TIMEOUT);
            }

            builder.build().map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| Error::Aws(AwsError::HttpClient(message.clone().into())))
}
