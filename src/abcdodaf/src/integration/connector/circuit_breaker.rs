//! Circuit breaker pattern for fault tolerance

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit breaker is closed (normal operation)
    Closed,
    /// Circuit breaker is open (failing fast)
    Open,
    /// Circuit breaker is half-open (testing recovery)
    HalfOpen,
}

impl std::fmt::Display for CircuitBreakerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "Closed"),
            Self::Open => write!(f, "Open"),
            Self::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    failure_threshold: usize,
    /// Duration to wait before trying to recover
    timeout: Duration,
    /// Number of successful calls needed to close the circuit from half-open
    success_threshold: usize,
}

/// Circuit breaker state tracker
#[derive(Debug)]
struct CircuitBreakerStateTracker {
    /// Current state
    state: CircuitBreakerState,
    /// Failure count
    failure_count: usize,
    /// Success count (for half-open state)
    success_count: usize,
    /// Time when circuit was opened
    opened_at: Option<SystemTime>,
    /// Configuration
    config: CircuitBreakerConfig,
}

impl CircuitBreakerStateTracker {
    /// Create a new circuit breaker tracker
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            opened_at: None,
            config,
        }
    }

    /// Check if circuit should transition from open to half-open
    fn check_timeout(&mut self) {
        if self.state == CircuitBreakerState::Open {
            if let Some(opened_at) = self.opened_at {
                if opened_at.elapsed().unwrap_or(Duration::MAX) >= self.config.timeout {
                    self.state = CircuitBreakerState::HalfOpen;
                    self.success_count = 0;
                }
            }
        }
    }

    /// Record a successful call
    fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitBreakerState::Open => {
                // Do nothing in open state
            }
        }
    }

    /// Record a failed call
    fn record_failure(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                    self.opened_at = Some(SystemTime::now());
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                self.opened_at = Some(SystemTime::now());
                self.failure_count = 0;
                self.success_count = 0;
            }
            CircuitBreakerState::Open => {
                // Stay open
            }
        }
    }
}

/// Thread-safe circuit breaker
#[derive(Clone)]
pub struct CircuitBreaker {
    inner: Arc<Mutex<CircuitBreakerStateTracker>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        let config = CircuitBreakerConfig {
            failure_threshold,
            timeout,
            success_threshold: 1,
        };

        Self {
            inner: Arc::new(Mutex::new(CircuitBreakerStateTracker::new(config))),
        }
    }

    /// Create a circuit breaker with custom success threshold
    pub fn with_success_threshold(
        failure_threshold: usize,
        timeout: Duration,
        success_threshold: usize,
    ) -> Self {
        let config = CircuitBreakerConfig {
            failure_threshold,
            timeout,
            success_threshold,
        };

        Self {
            inner: Arc::new(Mutex::new(CircuitBreakerStateTracker::new(config))),
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitBreakerState {
        let mut tracker = self.inner.lock().unwrap();
        tracker.check_timeout();
        tracker.state
    }

    /// Check if request can proceed
    pub fn can_proceed(&self) -> bool {
        self.state() != CircuitBreakerState::Open
    }

    /// Record a successful call
    pub fn record_success(&self) {
        let mut tracker = self.inner.lock().unwrap();
        tracker.record_success();
    }

    /// Record a failed call
    pub fn record_failure(&self) {
        let mut tracker = self.inner.lock().unwrap();
        tracker.record_failure();
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        let mut tracker = self.inner.lock().unwrap();
        tracker.state = CircuitBreakerState::Closed;
        tracker.failure_count = 0;
        tracker.success_count = 0;
        tracker.opened_at = None;
    }

    /// Get failure count
    pub fn failure_count(&self) -> usize {
        self.inner.lock().unwrap().failure_count
    }

    /// Get time until circuit can retry (for open state)
    pub fn time_until_retry(&self) -> Option<Duration> {
        let tracker = self.inner.lock().unwrap();
        if tracker.state == CircuitBreakerState::Open {
            if let Some(opened_at) = tracker.opened_at {
                let elapsed = opened_at.elapsed().unwrap_or(Duration::ZERO);
                if elapsed < tracker.config.timeout {
                    return Some(tracker.config.timeout - elapsed);
                }
            }
        }
        None
    }
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("state", &self.state())
            .field("failure_count", &self.failure_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_circuit_breaker_open() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(1));

        assert_eq!(cb.state(), CircuitBreakerState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);

        assert!(!cb.can_proceed());
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(100));

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);

        thread::sleep(Duration::from_millis(150));
        assert_eq!(cb.state(), CircuitBreakerState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(100));

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);

        thread::sleep(Duration::from_millis(150));
        assert_eq!(cb.state(), CircuitBreakerState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_circuit_breaker_reset_from_half_open() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(100));

        cb.record_failure();
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cb.state(), CircuitBreakerState::HalfOpen);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(10));

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_circuit_breaker_time_until_retry() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(200));

        cb.record_failure();
        let time_until_retry = cb.time_until_retry();
        assert!(time_until_retry.is_some());
        assert!(time_until_retry.unwrap().as_millis() > 0);
    }

    #[test]
    fn test_circuit_breaker_clone() {
        let cb1 = CircuitBreaker::new(1, Duration::from_secs(1));
        let cb2 = cb1.clone();

        cb1.record_failure();
        assert_eq!(cb2.state(), CircuitBreakerState::Open);
    }
}
