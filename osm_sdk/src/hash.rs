//! Streaming MD5 of large files (H27): fixed buffer, never the whole file in memory.

use md5::{Digest, Md5};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::SdkError;

const BUF: usize = 1 << 20;
/// Every OSM PBF starts with a 4-byte big-endian header length, then a BlobHeader whose first
/// field is the string "OSMHeader".
const PBF_MAGIC: &[u8] = b"OSMHeader";

pub fn md5_file(path: &Path, mut progress: impl FnMut(u64)) -> Result<[u8; 16], SdkError> {
    let mut f = File::open(path).map_err(|e| SdkError::io(path, e))?;
    let mut h = Md5::new();
    let mut buf = vec![0u8; BUF];
    let mut done = 0u64;
    loop {
        let n = f.read(&mut buf).map_err(|e| SdkError::io(path, e))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        done = done.saturating_add(n as u64);
        progress(done);
    }
    Ok(h.finalize().into())
}

pub fn to_hex(d: &[u8; 16]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Cheap check that a file looks like an OSM PBF before hashing gigabytes.
pub fn looks_like_pbf(path: &Path) -> Result<bool, SdkError> {
    let mut head = [0u8; 32];
    let mut f = File::open(path).map_err(|e| SdkError::io(path, e))?;
    let n = f.read(&mut head).map_err(|e| SdkError::io(path, e))?;
    Ok(head[..n].windows(PBF_MAGIC.len()).any(|w| w == PBF_MAGIC))
}

/// Import-time check (P-F5, P-R2): the local file must hash to Geofabrik's MD5 for `snapshot`.
pub fn verify_import(path: &Path, want: &[u8; 16], snapshot: &str) -> Result<(), SdkError> {
    if !looks_like_pbf(path)? {
        return Err(SdkError::NotAPbf {
            path: path.to_path_buf(),
        });
    }
    let got = md5_file(path, |_| {})?;
    if &got == want {
        Ok(())
    } else {
        Err(SdkError::ImportChecksumMismatch {
            got: to_hex(&got),
            want: to_hex(want),
            snapshot: snapshot.to_string(),
        })
    }
}
