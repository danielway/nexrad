//! Configuration shared by the real-time chunk iterator and stream.

use crate::aws::realtime::RetryPolicy;

/// Configuration for consuming real-time NEXRAD chunks, shared by
/// [`ChunkIterator`](super::ChunkIterator) and [`chunk_stream`](super::chunk_stream).
///
/// The retry policies apply to [`chunk_stream`](super::chunk_stream) only: the
/// [`ChunkIterator`](super::ChunkIterator) never sleeps or retries, and leaves polling cadence to
/// its caller.
///
/// Create one with [`PollConfig::new`] and adjust it with the chainable `with_*` setters. New
/// options are added as further setters, so existing code that builds a `PollConfig` keeps
/// compiling.
#[derive(Debug, Clone)]
pub struct PollConfig {
    site: String,
    download_policy: RetryPolicy,
    discovery_policy: RetryPolicy,
}

impl PollConfig {
    /// Creates a new poll configuration for the specified site with default policies.
    pub fn new(site: impl Into<String>) -> Self {
        Self {
            site: site.into(),
            download_policy: RetryPolicy::default_download(),
            discovery_policy: RetryPolicy::default_discovery(),
        }
    }

    /// The radar site identifier (e.g., "KDMX").
    pub fn site(&self) -> &str {
        &self.site
    }

    /// Retry policy for downloading individual chunks in [`chunk_stream`](super::chunk_stream).
    pub fn download_policy(&self) -> &RetryPolicy {
        &self.download_policy
    }

    /// Retry policy for discovering new volumes in [`chunk_stream`](super::chunk_stream).
    pub fn discovery_policy(&self) -> &RetryPolicy {
        &self.discovery_policy
    }

    /// Sets the retry policy for downloading individual chunks in
    /// [`chunk_stream`](super::chunk_stream).
    #[must_use]
    pub fn with_download_policy(mut self, policy: RetryPolicy) -> Self {
        self.download_policy = policy;
        self
    }

    /// Sets the retry policy for discovering new volumes in [`chunk_stream`](super::chunk_stream).
    #[must_use]
    pub fn with_discovery_policy(mut self, policy: RetryPolicy) -> Self {
        self.discovery_policy = policy;
        self
    }
}
