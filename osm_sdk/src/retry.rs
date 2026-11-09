//! Exponential back-off with full jitter for Storage uploads (P-R1, ADR 0004).

use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base: Duration,
    pub cap: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_attempts: 6,
            base: Duration::from_millis(500),
            cap: Duration::from_secs(30),
        }
    }
}

impl RetryPolicy {
    /// Upper bound of the delay before retry number `retry` (1-based): min(cap, base * 2^(retry-1)).
    pub fn ceiling(&self, retry: u32) -> Duration {
        let factor = 1u32
            .checked_shl(retry.saturating_sub(1))
            .unwrap_or(u32::MAX);
        self.base.saturating_mul(factor).min(self.cap)
    }
}

/// Whether a failure is worth retrying.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attempt<E> {
    Transient(E),
    Permanent(E),
}

/// Source of jitter in `[0, 1)`. Injected so tests are deterministic.
pub trait Jitter {
    fn unit(&mut self) -> f64;
}

/// xorshift64*, seeded per process. Jitter only spreads retries; it needs no cryptographic quality.
pub struct XorShift(u64);

impl XorShift {
    pub fn seeded() -> Self {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        XorShift(t | 1)
    }
}

impl Jitter for XorShift {
    fn unit(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Outcome after retries: the value, or the last error and how many attempts were made.
#[derive(Debug, PartialEq, Eq)]
pub enum RetryResult<T, E> {
    Ok(T),
    Failed {
        attempts: u32,
        last: E,
        permanent: bool,
    },
}

pub fn retry<T, E>(
    policy: &RetryPolicy,
    jitter: &mut impl Jitter,
    mut sleep: impl FnMut(Duration),
    mut op: impl FnMut(u32) -> Result<T, Attempt<E>>,
) -> RetryResult<T, E> {
    let max = policy.max_attempts.max(1);
    let mut attempt = 1;
    loop {
        match op(attempt) {
            Ok(v) => return RetryResult::Ok(v),
            Err(Attempt::Permanent(e)) => {
                return RetryResult::Failed {
                    attempts: attempt,
                    last: e,
                    permanent: true,
                }
            }
            Err(Attempt::Transient(e)) if attempt >= max => {
                return RetryResult::Failed {
                    attempts: attempt,
                    last: e,
                    permanent: false,
                }
            }
            Err(Attempt::Transient(_)) => {
                sleep(
                    policy
                        .ceiling(attempt)
                        .mul_f64(jitter.unit().clamp(0.0, 1.0)),
                );
                attempt += 1;
            }
        }
    }
}
