//! P-R1 / H13: retries with back-off against a fault-injecting backend.

use osm_sdk::retry::{Jitter, RetryPolicy};
use osm_sdk::storage::{upload_with_retry, StorageBackend, StorageFault};
use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;

struct Scripted(VecDeque<Result<String, StorageFault>>, u32);

impl StorageBackend for Scripted {
    fn upload_file(&mut self, _: &Path) -> Result<String, StorageFault> {
        self.1 += 1;
        self.0
            .pop_front()
            .unwrap_or(Err(StorageFault::Transient("connection refused".into())))
    }
    fn download_to_file(&mut self, _: &str, _: &Path) -> Result<(), StorageFault> {
        unreachable!()
    }
    fn exists(&mut self, _: &str) -> Result<bool, StorageFault> {
        unreachable!()
    }
}

struct Max;
impl Jitter for Max {
    fn unit(&mut self) -> f64 {
        0.999_999
    }
}

const CID: &str = "zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb"; // real v0.5.3 upload, 2026-11-09

fn run(
    script: Vec<Result<String, StorageFault>>,
    policy: RetryPolicy,
) -> (Result<String, osm_sdk::SdkError>, u32, Vec<Duration>) {
    let mut b = Scripted(script.into(), 0);
    let mut sleeps = Vec::new();
    let r = upload_with_retry(&mut b, Path::new("x"), &policy, &mut Max, |d| {
        sleeps.push(d)
    });
    (r, b.1, sleeps)
}

#[test]
fn recovers_after_transient_failures() {
    let t = || Err(StorageFault::Transient("503".into()));
    let (r, calls, sleeps) = run(vec![t(), t(), Ok(CID.into())], RetryPolicy::default());
    assert_eq!(r.unwrap(), CID);
    assert_eq!(calls, 3);
    assert_eq!(sleeps.len(), 2);
    assert!(sleeps[0] < Duration::from_millis(500) && sleeps[1] < Duration::from_millis(1000));
    assert!(sleeps[1] > sleeps[0], "back-off grows");
}

#[test]
fn exhausts_with_clear_error() {
    let p = RetryPolicy {
        max_attempts: 4,
        ..RetryPolicy::default()
    };
    let (r, calls, sleeps) = run(vec![], p);
    let e = r.unwrap_err();
    assert_eq!(e.code(), "S111");
    assert_eq!(calls, 4);
    assert_eq!(sleeps.len(), 3);
    assert!(
        e.to_string().contains("4 attempts") && e.to_string().contains("connection refused"),
        "{e}"
    );
}

#[test]
fn permanent_failure_is_not_retried() {
    let (r, calls, sleeps) = run(
        vec![Err(StorageFault::Permanent("422 bad filename".into()))],
        RetryPolicy::default(),
    );
    assert_eq!(r.unwrap_err().code(), "S111");
    assert_eq!((calls, sleeps.len()), (1, 0));
}

#[test]
fn node_returning_garbage_cid_is_rejected() {
    let (r, _, _) = run(vec![Ok("<html>".into())], RetryPolicy::default());
    assert_eq!(r.unwrap_err().code(), "S160");
}

#[test]
fn ceiling_is_capped_and_overflow_safe() {
    let p = RetryPolicy::default();
    assert_eq!(p.ceiling(1), Duration::from_millis(500));
    assert_eq!(p.ceiling(2), Duration::from_secs(1));
    assert_eq!(p.ceiling(7), Duration::from_secs(30));
    assert_eq!(p.ceiling(u32::MAX), Duration::from_secs(30));
    let zero = RetryPolicy {
        max_attempts: 0,
        ..p
    };
    let (_, calls, _) = run(vec![], zero);
    assert_eq!(calls, 1, "max_attempts 0 still tries once");
}
