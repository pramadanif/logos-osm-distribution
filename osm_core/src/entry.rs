//! Entry validation shared by the registry program and the SDK (ADR 0002).
//!
//! Every check works on borrowed input and allocates nothing, so the guest can run it as is.
//! Input is rejected, never trimmed or normalised.

use crate::{is_valid_position, lookup, Level};

/// Base of every accepted `source_url`.
pub const GEOFABRIK_BASE: &str = "https://download.geofabrik.de/";
/// Largest accepted PBF (64 GiB). The biggest file in the set is a few GB.
pub const MAX_SIZE_BYTES: u64 = 64 * 1024 * 1024 * 1024;
/// Earliest accepted registration time, 2026-11-01T00:00:00Z.
pub const MIN_TIMESTAMP: u64 = 1_761_955_200;
/// Half width of the validity window around a client timestamp, in seconds.
pub const TIMESTAMP_WINDOW_SECS: u64 = 600;
/// Bounds per region account (ADR 0001).
pub const MAX_HISTORY: usize = 8;
pub const MAX_HOSTS: usize = 16;
/// CID length bounds in characters.
pub const MIN_CID_LEN: usize = 46;
pub const MAX_CID_LEN: usize = 96;

/// Program error codes, numbered as in docs/error-codes.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum EntryError {
    RegionNotInSet = 1,
    PositionMismatch = 2,
    BadCid = 3,
    BadSourceUrl = 4,
    VersionMismatch = 5,
    BadChecksum = 6,
    SizeOutOfRange = 7,
    TimestampOutOfRange = 8,
}

impl EntryError {
    pub const fn code(self) -> &'static str {
        match self {
            EntryError::RegionNotInSet => "P001",
            EntryError::PositionMismatch => "P002",
            EntryError::BadCid => "P003",
            EntryError::BadSourceUrl => "P004",
            EntryError::VersionMismatch => "P005",
            EntryError::BadChecksum => "P006",
            EntryError::SizeOutOfRange => "P007",
            EntryError::TimestampOutOfRange => "P008",
        }
    }
}

/// The client-supplied part of a registration, before the program adds signer fields.
#[derive(Clone, Copy, Debug)]
pub struct EntryInput<'a> {
    pub region: &'a str,
    pub parent: Option<&'a str>,
    pub level: Level,
    pub cid: &'a str,
    pub source_url: &'a str,
    pub checksum_hex: &'a str,
    pub version: u32,
    pub size_bytes: u64,
    pub timestamp: u64,
}

/// Validated values the program stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidEntry {
    pub checksum: [u8; 16],
    pub version: u32,
}

pub fn validate_entry(e: &EntryInput<'_>) -> Result<ValidEntry, EntryError> {
    if lookup(e.region).is_none() {
        return Err(EntryError::RegionNotInSet);
    }
    if !is_valid_position(e.region, e.parent, e.level) {
        return Err(EntryError::PositionMismatch);
    }
    validate_cid(e.cid)?;
    let snapshot = snapshot_from_source_url(e.region, e.source_url)?;
    if snapshot != e.version {
        return Err(EntryError::VersionMismatch);
    }
    let checksum = parse_md5_hex(e.checksum_hex)?;
    if e.size_bytes == 0 || e.size_bytes > MAX_SIZE_BYTES {
        return Err(EntryError::SizeOutOfRange);
    }
    if e.timestamp < MIN_TIMESTAMP {
        return Err(EntryError::TimestampOutOfRange);
    }
    Ok(ValidEntry {
        checksum,
        version: snapshot,
    })
}

/// True when `timestamp` is within the validity window around `block_time`.
pub fn timestamp_in_window(timestamp: u64, block_time: u64) -> bool {
    timestamp >= MIN_TIMESTAMP && timestamp.abs_diff(block_time) <= TIMESTAMP_WINDOW_SECS
}

const BASE58: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Accepts a CIDv0 (`Qm` + 44 base58 characters) or a base58btc CIDv1 (`z` + base58).
///
/// Structural check only; the exact codec prefix Logos Storage emits is pinned once a real
/// upload has been captured (docs/status/phase--1.md, SC-−1.3).
pub fn validate_cid(cid: &str) -> Result<(), EntryError> {
    let b = cid.as_bytes();
    if !(MIN_CID_LEN..=MAX_CID_LEN).contains(&b.len()) || !b.iter().all(|c| BASE58.contains(c)) {
        return Err(EntryError::BadCid);
    }
    let v0 = b.len() == 46 && b.starts_with(b"Qm");
    let v1 = b[0] == b'z';
    if v0 || v1 {
        Ok(())
    } else {
        Err(EntryError::BadCid)
    }
}

/// Parses 32 lowercase hex characters, the form Geofabrik publishes in `.md5` files.
pub fn parse_md5_hex(s: &str) -> Result<[u8; 16], EntryError> {
    let b = s.as_bytes();
    if b.len() != 32 {
        return Err(EntryError::BadChecksum);
    }
    let mut out = [0u8; 16];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = (hex_nibble(b[2 * i])? << 4) | hex_nibble(b[2 * i + 1])?;
    }
    Ok(out)
}

fn hex_nibble(c: u8) -> Result<u8, EntryError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err(EntryError::BadChecksum),
    }
}

/// Returns the snapshot date as `YYMMDD` when `url` is
/// `https://download.geofabrik.de/<geofabrik_path>-YYMMDD.osm.pbf` for `region`.
pub fn snapshot_from_source_url(region: &str, url: &str) -> Result<u32, EntryError> {
    let def = lookup(region).ok_or(EntryError::RegionNotInSet)?;
    let rest = url
        .strip_prefix(GEOFABRIK_BASE)
        .and_then(|r| r.strip_prefix(def.geofabrik_path))
        .and_then(|r| r.strip_prefix('-'))
        .and_then(|r| r.strip_suffix(".osm.pbf"))
        .ok_or(EntryError::BadSourceUrl)?;
    let d = rest.as_bytes();
    if d.len() != 6 || !d.iter().all(u8::is_ascii_digit) {
        return Err(EntryError::BadSourceUrl);
    }
    let num = |i: usize| u32::from(d[i] - b'0') * 10 + u32::from(d[i + 1] - b'0');
    let (yy, mm, dd) = (num(0), num(2), num(4));
    if !(1..=12).contains(&mm) || !(1..=31).contains(&dd) {
        return Err(EntryError::BadSourceUrl);
    }
    Ok(yy * 10_000 + mm * 100 + dd)
}
