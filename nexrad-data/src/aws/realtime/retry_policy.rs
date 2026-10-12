//! Configurable retry policy for resilient operations.
//!
//! This module provides platform-independent retry logic by returning delay durations
//! rather than performing sleep operations internally. The caller controls timing.

use std::time::Duration;

/// Configuration for retry behavior with exponential backoff.
///
/// Start from [`default_download`](Self::default_download),
/// [`default_discovery`](Self::default_discovery), or [`Default`] and adjust with the chainable
/// `with_*` setters:
///
/// ```
/// use nexrad_data::aws::realtime::RetryPolicy;
/// use std::time::Duration;
///
/// let policy = RetryPolicy::default_download()
///     .with_max_attempts(3)
///     .with_initial_delay(Duration::from_millis(250));
/// assert_eq!(policy.max_attempts(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: usize,
    initial_delay: Duration,
    backoff_multiplier: f64,
    max_delay: Duration,
}

impl RetryPolicy {
    /// Creates a default retry policy for chunk downloads.
    ///
    /// - 5 attempts
    /// - 500ms initial delay
    /// - 2.0x backoff multiplier
    /// - 8 second max delay
    pub fn default_download() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(8),
        }
    }

    /// Creates a default retry policy for volume/chunk discovery.
    ///
    /// - 10 attempts
    /// - 500ms initial delay
    /// - 2.0x backoff multiplier
    /// - 16 second max delay
    pub fn default_discovery() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_millis(500),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(16),
        }
    }

    /// Maximum number of attempts before giving up.
    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    /// Delay before the first retry.
    pub fn initial_delay(&self) -> Duration {
        self.initial_delay
    }

    /// Multiplier applied to the delay after each attempt.
    pub fn backoff_multiplier(&self) -> f64 {
        self.backoff_multiplier
    }

    /// Maximum delay between attempts (caps exponential growth).
    pub fn max_delay(&self) -> Duration {
        self.max_delay
    }

    /// Sets the maximum number of retry attempts.
    pub fn with_max_attempts(mut self, n: usize) -> Self {
        self.max_attempts = n;
        self
    }

    /// Sets the initial delay before the first retry.
    pub fn with_initial_delay(mut self, d: Duration) -> Self {
        self.initial_delay = d;
        self
    }

    /// Sets the backoff multiplier applied after each attempt.
    pub fn with_backoff_multiplier(mut self, m: f64) -> Self {
        self.backoff_multiplier = m;
        self
    }

    /// Sets the maximum delay between attempts.
    pub fn with_max_delay(mut self, d: Duration) -> Self {
        self.max_delay = d;
        self
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_download()
    }
}

/// Tracks retry state for an operation.
///
/// This struct returns delay durations rather than performing sleep operations,
/// allowing the caller to control timing. This makes it suitable for use in any
/// async runtime or environment.
#[derive(Debug, Clone)]
pub(crate) struct RetryState {
    policy: RetryPolicy,
    attempt: usize,
}

impl RetryState {
    /// Creates a new retry state with the given policy.
    pub(crate) fn new(policy: RetryPolicy) -> Self {
        Self { policy, attempt: 0 }
    }

    /// Returns the current attempt number (1-indexed after first attempt).
    pub(crate) fn attempt(&self) -> usize {
        self.attempt
    }

    /// Returns true if more retry attempts are available.
    pub(crate) fn should_retry(&self) -> bool {
        self.attempt < self.policy.max_attempts
    }

    /// Records an attempt and returns the delay before the next retry.
    ///
    /// Returns `Some(duration)` if another attempt should be made after waiting,
    /// or `None` if max attempts have been exhausted.
    pub(crate) fn next_delay(&mut self) -> Option<Duration> {
        if self.attempt >= self.policy.max_attempts {
            return None;
        }

        let delay = if self.attempt == 0 {
            self.policy.initial_delay
        } else {
            let multiplier = self.policy.backoff_multiplier.powi(self.attempt as i32 - 1);
            let delay_nanos = self.policy.initial_delay.as_nanos() as f64 * multiplier;
            if delay_nanos.is_finite() && delay_nanos < self.policy.max_delay.as_nanos() as f64 {
                // Negative products (from a negative multiplier) saturate to zero.
                Duration::from_nanos(delay_nanos as u64)
            } else {
                self.policy.max_delay
            }
        };

        self.attempt += 1;
        Some(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_defaults() {
        let download = RetryPolicy::default_download();
        assert_eq!(download.max_attempts(), 5);
        assert_eq!(download.initial_delay(), Duration::from_millis(500));
        assert_eq!(download.backoff_multiplier(), 2.0);
        assert_eq!(download.max_delay(), Duration::from_secs(8));

        let discovery = RetryPolicy::default_discovery();
        assert_eq!(discovery.max_attempts(), 10);
        assert_eq!(discovery.max_delay(), Duration::from_secs(16));
    }

    #[test]
    fn test_retry_policy_builder() {
        let policy = RetryPolicy::default_download()
            .with_max_attempts(3)
            .with_initial_delay(Duration::from_millis(100));

        assert_eq!(policy.max_attempts(), 3);
        assert_eq!(policy.initial_delay(), Duration::from_millis(100));
    }

    #[test]
    fn test_retry_state_basic() {
        let policy = RetryPolicy::default_download().with_max_attempts(3);
        let mut state = RetryState::new(policy);

        assert_eq!(state.attempt(), 0);
        assert!(state.should_retry());

        // First attempt
        let delay = state.next_delay();
        assert!(delay.is_some());
        assert_eq!(state.attempt(), 1);

        // Second attempt
        let delay = state.next_delay();
        assert!(delay.is_some());
        assert_eq!(state.attempt(), 2);

        // Third attempt
        let delay = state.next_delay();
        assert!(delay.is_some());
        assert_eq!(state.attempt(), 3);

        // No more attempts
        assert!(!state.should_retry());
        let delay = state.next_delay();
        assert!(delay.is_none());
    }

    #[test]
    fn test_retry_state_backoff() {
        let policy = RetryPolicy::default_download()
            .with_max_attempts(4)
            .with_initial_delay(Duration::from_millis(100))
            .with_backoff_multiplier(2.0)
            .with_max_delay(Duration::from_secs(10));

        let mut state = RetryState::new(policy);

        // First delay is initial
        let delay1 = state.next_delay().unwrap();
        assert_eq!(delay1, Duration::from_millis(100));

        // Second delay is initial (first retry)
        let delay2 = state.next_delay().unwrap();
        assert_eq!(delay2, Duration::from_millis(100));

        // Third delay is 2x
        let delay3 = state.next_delay().unwrap();
        assert_eq!(delay3, Duration::from_millis(200));

        // Fourth delay is 4x
        let delay4 = state.next_delay().unwrap();
        assert_eq!(delay4, Duration::from_millis(400));
    }

    #[test]
    fn test_retry_state_max_delay_cap() {
        let policy = RetryPolicy::default_download()
            .with_max_attempts(10)
            .with_initial_delay(Duration::from_secs(1))
            .with_backoff_multiplier(10.0)
            .with_max_delay(Duration::from_secs(5));

        let mut state = RetryState::new(policy);

        // First delay
        let _ = state.next_delay();
        // Second delay would be 1s
        let _ = state.next_delay();
        // Third delay would be 10s but capped at 5s
        let delay = state.next_delay().unwrap();
        assert_eq!(delay, Duration::from_secs(5));
    }
}
