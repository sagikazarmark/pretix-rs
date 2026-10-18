use std::time::Duration;

use http::{Method, StatusCode};

/// Controls automatic retries of failed requests.
///
/// Only requests that are safe to repeat are retried: `GET`, `HEAD` and `OPTIONS`
/// requests, and requests that carry an idempotency key (see
/// [`Request::idempotency_key`](crate::Request::idempotency_key)).
///
/// Retried are transport errors and transient responses (see
/// [`Error::is_retryable`](crate::Error::is_retryable)): `408`, `429` (for any method,
/// as the request was not executed), `500`, `502`, `503`, `504`, and `409 Conflict`
/// with `Retry-After` (pretix answers like this while a request with the same
/// idempotency key is still being processed). `Retry-After` is honored.
///
/// Durable execution engines that retry on their own should use
/// [`RetryPolicy::disabled`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
    max_retry_after: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            max_retry_after: Duration::from_secs(60),
        }
    }
}

impl RetryPolicy {
    /// Never retry.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            max_retries: 0,
            ..Self::default()
        }
    }

    /// Sets the maximum number of retries (not counting the initial attempt).
    #[must_use]
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Sets the delay before the first retry. It doubles with every further attempt.
    #[must_use]
    pub fn with_initial_backoff(mut self, backoff: Duration) -> Self {
        self.initial_backoff = backoff;
        self
    }

    /// Caps the exponential backoff delay.
    #[must_use]
    pub fn with_max_backoff(mut self, backoff: Duration) -> Self {
        self.max_backoff = backoff;
        self
    }

    /// The longest `Retry-After` the client is willing to wait. Longer requested delays
    /// are returned to the caller as an error instead.
    #[must_use]
    pub fn with_max_retry_after(mut self, max: Duration) -> Self {
        self.max_retry_after = max;
        self
    }

    /// The maximum number of retries.
    #[must_use]
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Exponential backoff, capped at `max_backoff`.
    pub(crate) fn backoff(&self, attempt: u32) -> Duration {
        let factor = 2u32.saturating_pow(attempt);
        self.initial_backoff
            .saturating_mul(factor)
            .min(self.max_backoff)
    }

    /// [`RetryPolicy::backoff`] with jitter (50–100 %), so that clients failing at the
    /// same time do not retry in lockstep.
    fn jittered_backoff(&self, attempt: u32) -> Duration {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.subsec_nanos());
        let fraction = 0.5 + f64::from(nanos % 1000) / 2000.0;
        self.backoff(attempt).mul_f64(fraction)
    }

    /// Returns the delay before retrying, or `None` if the outcome must not be retried.
    ///
    /// Uses the same rule as [`Error::is_retryable`](crate::Error::is_retryable).
    pub(crate) fn delay(&self, attempt: u32, outcome: Outcome<'_>) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }
        match outcome.result {
            OutcomeResult::Transport => outcome
                .request
                .is_repeatable()
                .then(|| self.jittered_backoff(attempt)),
            OutcomeResult::Status {
                status,
                retry_after,
            } => {
                let retry = match status {
                    // Only conflicts of requests with an idempotency key are in-flight
                    // duplicates; for other requests (e.g. job polling) `409` is a
                    // meaningful answer.
                    StatusCode::CONFLICT => {
                        outcome.request.has_idempotency_key && retry_after.is_some()
                    }
                    StatusCode::TOO_MANY_REQUESTS => true,
                    _ => {
                        crate::error::is_transient_status(status, retry_after)
                            && outcome.request.is_repeatable()
                    }
                };
                if !retry {
                    return None;
                }
                match retry_after {
                    Some(delay) if delay > self.max_retry_after => None,
                    Some(delay) => Some(delay),
                    None => Some(self.jittered_backoff(attempt)),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RequestKind<'a> {
    pub(crate) method: &'a Method,
    pub(crate) has_idempotency_key: bool,
}

impl RequestKind<'_> {
    /// Whether the request may be sent again without risking duplicate effects.
    pub(crate) fn is_repeatable(&self) -> bool {
        self.has_idempotency_key
            || matches!(*self.method, Method::GET | Method::HEAD | Method::OPTIONS)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Outcome<'a> {
    pub(crate) request: RequestKind<'a>,
    pub(crate) result: OutcomeResult,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum OutcomeResult {
    Transport,
    Status {
        status: StatusCode,
        retry_after: Option<Duration>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(method: &Method, key: bool, status: StatusCode) -> Outcome<'_> {
        Outcome {
            request: RequestKind {
                method,
                has_idempotency_key: key,
            },
            result: OutcomeResult::Status {
                status,
                retry_after: None,
            },
        }
    }

    #[test]
    fn retries_rate_limited_requests_of_any_method() {
        let policy = RetryPolicy::default();
        for method in [Method::GET, Method::POST] {
            let delay = policy
                .delay(0, outcome(&method, false, StatusCode::TOO_MANY_REQUESTS))
                .unwrap();
            assert!(delay >= Duration::from_millis(500) && delay <= Duration::from_secs(1));
        }
    }

    #[test]
    fn never_retries_unkeyed_post_on_server_errors() {
        let policy = RetryPolicy::default();
        for status in [
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::INTERNAL_SERVER_ERROR,
        ] {
            assert_eq!(policy.delay(0, outcome(&Method::POST, false, status)), None);
            assert!(
                policy
                    .delay(0, outcome(&Method::POST, true, status))
                    .is_some()
            );
        }
    }

    #[test]
    fn retries_conflict_only_with_key_and_retry_after() {
        let policy = RetryPolicy::default();
        let conflict = |method, key, retry_after| Outcome {
            request: RequestKind {
                method,
                has_idempotency_key: key,
            },
            result: OutcomeResult::Status {
                status: StatusCode::CONFLICT,
                retry_after,
            },
        };
        let five = Some(Duration::from_secs(5));
        assert_eq!(policy.delay(0, conflict(&Method::POST, true, five)), five);
        assert!(
            policy
                .delay(0, conflict(&Method::POST, true, None))
                .is_none()
        );
        assert!(
            policy
                .delay(0, conflict(&Method::GET, false, five))
                .is_none()
        );
    }

    #[test]
    fn respects_limits() {
        let policy = RetryPolicy::default().with_max_retries(2);
        assert!(
            policy
                .delay(
                    2,
                    outcome(&Method::GET, false, StatusCode::SERVICE_UNAVAILABLE)
                )
                .is_none()
        );
        assert!(
            RetryPolicy::disabled()
                .delay(
                    0,
                    outcome(&Method::GET, false, StatusCode::SERVICE_UNAVAILABLE)
                )
                .is_none()
        );

        let long = Outcome {
            request: RequestKind {
                method: &Method::GET,
                has_idempotency_key: false,
            },
            result: OutcomeResult::Status {
                status: StatusCode::TOO_MANY_REQUESTS,
                retry_after: Some(Duration::from_secs(3600)),
            },
        };
        assert!(policy.delay(0, long).is_none());
    }

    #[test]
    fn backoff_is_capped() {
        let policy = RetryPolicy::default().with_max_backoff(Duration::from_secs(5));
        assert_eq!(policy.backoff(0), Duration::from_secs(1));
        assert_eq!(policy.backoff(2), Duration::from_secs(4));
        assert_eq!(policy.backoff(10), Duration::from_secs(5));
    }
}
