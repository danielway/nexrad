//! Shared HTTP client for AWS operations.
//!
//! This module provides a singleton HTTP client with connection pooling for
//! efficient reuse across multiple S3 operations.

use crate::result::{AwsError, Error, Result};
use reqwest::Client;
use std::sync::OnceLock;

/// Returns a reference to the shared HTTP client.
///
/// The client is built on first use and reused for all subsequent requests. On native targets, it
/// is configured with connection pooling for improved performance when making multiple requests to
/// the same host. If building fails, every call returns [`AwsError::HttpClient`].
pub(crate) fn client() -> Result<&'static Client> {
    static CLIENT: OnceLock<std::result::Result<Client, String>> = OnceLock::new();

    CLIENT
        .get_or_init(|| {
            #[allow(unused_mut)]
            let mut builder = Client::builder();

            // Connection pooling is only available on native targets
            #[cfg(not(target_arch = "wasm32"))]
            {
                builder = builder.pool_max_idle_per_host(4);
            }

            builder.build().map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| Error::Aws(AwsError::HttpClient(message.clone().into())))
}
