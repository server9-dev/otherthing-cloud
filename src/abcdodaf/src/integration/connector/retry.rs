//! Retry policies for failed requests

use std::time::Duration;

/// Retry strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryStrategy {
    /// No retry
    NoRetry,
    /// Fixed delay between retries
    FixedDelay,
    /// Exponential backoff (delay increases exponentially)
    ExponentialBackoff,
    /// Linear backoff (delay increases linearly)
    LinearBackoff,
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Strategy to use
    pub strategy: RetryStrategy,
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial delay for first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier (for exponential backoff)
    pub backoff_multiplier: f64,
}

impl RetryPolicy {
    /// Create a new retry policy with no retry
    pub fn no_retry() -> Self {
        Self {
            strategy: RetryStrategy::NoRetry,
            max_attempts: 1,
            initial_delay: Duration::from_secs(0),
            max_delay: Duration::from_secs(0),
            backoff_multiplier: 1.0,
        }
    }

    /// Create a new retry policy with fixed delay
    pub fn fixed_delay(max_attempts: usize, delay: Duration) -> Self {
        Self {
            strategy: RetryStrategy::FixedDelay,
            max_attempts,
            initial_delay: delay,
            max_delay: delay,
            backoff_multiplier: 1.0,
        }
    }

    /// Create a new retry policy with exponential backoff
    pub fn exponential_backoff(
        max_attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            strategy: RetryStrategy::ExponentialBackoff,
            max_attempts,
            initial_delay,
            max_delay,
            backoff_multiplier: 2.0,
        }
    }

    /// Create a new retry policy with linear backoff
    pub fn linear_backoff(
        max_attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            strategy: RetryStrategy::LinearBackoff,
            max_attempts,
            initial_delay,
            max_delay,
            backoff_multiplier: 1.0,
        }
    }

    /// Calculate delay for a retry attempt
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 || self.strategy == RetryStrategy::NoRetry {
            return Duration::from_secs(0);
        }

        let delay_ms = match self.strategy {
            RetryStrategy::NoRetry => 0,
            RetryStrategy::FixedDelay => self.initial_delay.as_millis() as u64,
            RetryStrategy::ExponentialBackoff => {
                let base = self.initial_delay.as_millis() as f64;
                let exp = (self.backoff_multiplier).powi((attempt - 1) as i32);
                (base * exp) as u64
            }
            RetryStrategy::LinearBackoff => {
                let base = self.initial_delay.as_millis() as u64;
                base * (attempt as u64)
            }
        };

        let capped_delay = std::cmp::min(delay_ms, self.max_delay.as_millis() as u64);
        Duration::from_millis(capped_delay)
    }

    /// Check if should retry
    pub fn should_retry(&self, attempt: usize) -> bool {
        attempt < self.max_attempts
    }

    /// Get the next retry delay
    pub fn next_retry_delay(&self, attempt: usize) -> Option<Duration> {
        if self.should_retry(attempt) {
            Some(self.calculate_delay(attempt + 1))
        } else {
            None
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential_backoff(
            3,
            Duration::from_millis(100),
            Duration::from_secs(30),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_retry() {
        let policy = RetryPolicy::no_retry();
        assert!(!policy.should_retry(0));
        assert_eq!(policy.next_retry_delay(0), None);
    }

    #[test]
    fn test_fixed_delay() {
        let delay = Duration::from_secs(5);
        let policy = RetryPolicy::fixed_delay(3, delay);

        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));

        assert_eq!(policy.calculate_delay(1), delay);
        assert_eq!(policy.calculate_delay(2), delay);
    }

    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy::exponential_backoff(
            4,
            Duration::from_millis(100),
            Duration::from_secs(10),
        );

        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);
        let delay3 = policy.calculate_delay(3);

        // Each should be approximately double the previous
        assert!(delay2.as_millis() >= delay1.as_millis() * 2 - 10);
        assert!(delay3.as_millis() >= delay2.as_millis() * 2 - 10);
    }

    #[test]
    fn test_linear_backoff() {
        let policy = RetryPolicy::linear_backoff(
            4,
            Duration::from_millis(100),
            Duration::from_secs(10),
        );

        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);

        assert_eq!(delay1.as_millis() as u64, 100);
        assert_eq!(delay2.as_millis() as u64, 200);
    }

    #[test]
    fn test_max_delay_capping() {
        let policy = RetryPolicy::exponential_backoff(
            10,
            Duration::from_secs(1),
            Duration::from_secs(5),
        );

        let delay = policy.calculate_delay(10);
        assert!(delay <= Duration::from_secs(5));
    }
}
