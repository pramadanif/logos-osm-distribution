//! Storage backend trait (ADR 0004) and the retrying upload wrapper.

use std::path::Path;
use std::time::Duration;

use crate::retry::{retry, Attempt, Jitter, RetryPolicy, RetryResult};
use crate::SdkError;

/// Failure from one backend call, classified for retry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageFault {
    /// Connect error, timeout, HTTP 5xx: retried.
    Transient(String),
    /// HTTP 4xx, bad file: not retried.
    Permanent(String),
}

/// Streams by path, never by byte buffer (H27). Implemented by the REST node client and by the
/// Basecamp `storage_module` bridge.
pub trait StorageBackend {
    fn upload_file(&mut self, path: &Path) -> Result<String, StorageFault>;
    fn download_to_file(&mut self, cid: &str, dest: &Path) -> Result<(), StorageFault>;
    fn exists(&mut self, cid: &str) -> Result<bool, StorageFault>;
}

/// Uploads with back-off and jitter; after the last attempt returns `StorageUploadExhausted`.
pub fn upload_with_retry(
    backend: &mut impl StorageBackend,
    path: &Path,
    policy: &RetryPolicy,
    jitter: &mut impl Jitter,
    sleep: impl FnMut(Duration),
) -> Result<String, SdkError> {
    let res = retry(policy, jitter, sleep, |_| {
        backend.upload_file(path).map_err(|f| match f {
            StorageFault::Transient(m) => Attempt::Transient(m),
            StorageFault::Permanent(m) => Attempt::Permanent(m),
        })
    });
    match res {
        RetryResult::Ok(cid) => {
            osm_core::entry::validate_cid(&cid).map_err(|_| SdkError::InvalidInput {
                field: "cid from storage node",
                why: format!("{cid:?} is not a valid CID"),
            })?;
            Ok(cid)
        }
        RetryResult::Failed { attempts, last, .. } => Err(SdkError::StorageUploadExhausted {
            attempts,
            last_cause: last,
        }),
    }
}
