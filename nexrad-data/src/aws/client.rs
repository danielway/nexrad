//! Shared HTTP client for AWS operations.
//!
//! This module provides a singleton HTTP client with connection pooling for
//! efficient reuse across multiple S3 operations.

use once_cell::sync::Lazy;
use reqwest::Client;

/// Returns a reference to the shared HTTP client.
///
/// The client is lazily initialized on first use and reused for all subsequent
/// requests. On native targets, it is configured with connection pooling for
/// improved performance when making multiple requests to the same host.
pub fn client() -> &'static Client {
    static CLIENT: Lazy<Client> = Lazy::new(|| {
        #[allow(unused_mut)]
        let mut builder = Client::builder();

        // Connection pooling is only available on native targets
        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder.pool_max_idle_per_host(4);
        }

        builder
            .build()
            .unwrap_or_else(|e| panic!("Failed to create HTTP client: {e}"))
    });

    &CLIENT
}
