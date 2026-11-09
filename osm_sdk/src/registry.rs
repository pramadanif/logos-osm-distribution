//! Reading the on-chain registry (ADR 0001, ADR 0002).
//!
//! One account per region at a program-derived address. Account data is written through the
//! program, but the SDK still treats it as untrusted: sizes and counts are re-checked when read
//! (E72), and the decoder never trusts a length prefix beyond the bytes it has.

use std::io::Read;
use std::time::Duration;

use borsh::BorshDeserialize;
use osm_core::entry::{validate_cid, MAX_HISTORY, MAX_HOSTS};
use osm_core::{Level, PREDEFINED};
use sha2::{Digest, Sha256};

use crate::SdkError;

/// LEZ `AccountId::for_public_pda` prefix (LEZ v0.3.x `lee/state_machine/core/src/program/mod.rs`).
const PDA_PREFIX: &[u8; 32] = b"/LEE/v0.2/AccountId/PDA/\x00\x00\x00\x00\x00\x00\x00\x00";
const SEED_CONST: &str = "osm-region";
/// Largest region account accepted from the network. The program's own bound is far lower
/// (8 entries x 16 hosts is about 6 KB); 100 KiB is the LEZ v0.3 account limit.
pub const MAX_ACCOUNT_BYTES: usize = 100 * 1024;
const MAX_RPC_BODY: u64 = 4 * MAX_ACCOUNT_BYTES as u64;

fn seed_from_str(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..s.len()].copy_from_slice(s.as_bytes());
    out
}

pub fn decode_account_id(b58: &str) -> Result<[u8; 32], SdkError> {
    let bytes = bs58::decode(b58)
        .into_vec()
        .map_err(|e| SdkError::InvalidInput {
            field: "account id",
            why: format!("{b58:?}: {e}"),
        })?;
    bytes.try_into().map_err(|_| SdkError::InvalidInput {
        field: "account id",
        why: format!("{b58:?} is not 32 bytes"),
    })
}

/// Address of the region account for `region` under `program` (base58 in, base58 out).
/// Same derivation as the SPEL seeds `["osm-region", arg region]`.
pub fn region_account_id(program: &str, region: &str) -> Result<String, SdkError> {
    if osm_core::lookup(region).is_none() {
        return Err(SdkError::InvalidInput {
            field: "region",
            why: format!("{region:?} is not in the predefined set"),
        });
    }
    let program = decode_account_id(program)?;
    let mut h = Sha256::new();
    h.update(seed_from_str(SEED_CONST));
    h.update(seed_from_str(region));
    let seed: [u8; 32] = h.finalize().into();
    let mut h = Sha256::new();
    h.update(PDA_PREFIX);
    h.update(program);
    h.update(seed);
    let id: [u8; 32] = h.finalize().into();
    Ok(bs58::encode(id).into_string())
}

/// One registered copy, as stored on-chain (layout 1).
#[derive(BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub cid: String,
    pub source_url: String,
    pub checksum: [u8; 16],
    pub version: u32,
    pub size_bytes: u64,
    pub hosted: bool,
    /// Unix milliseconds.
    pub timestamp: u64,
    pub registered_by: [u8; 32],
    pub hosts: Vec<[u8; 32]>,
}

#[derive(BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct RegionAccount {
    pub layout: u8,
    pub region: String,
    pub parent: String,
    pub level: u8,
    pub history: Vec<Entry>,
}

impl Entry {
    pub fn registered_by_b58(&self) -> String {
        bs58::encode(self.registered_by).into_string()
    }
    pub fn hosts_b58(&self) -> Vec<String> {
        self.hosts
            .iter()
            .map(|h| bs58::encode(h).into_string())
            .collect()
    }
}

/// Decodes and re-checks one region account. `account` is only used in error text.
pub fn decode_region(
    bytes: &[u8],
    expected_region: &str,
    account: &str,
) -> Result<RegionAccount, SdkError> {
    let bad = |why: String| SdkError::RegistryDataInvalid {
        account: account.to_string(),
        why,
    };
    if bytes.len() > MAX_ACCOUNT_BYTES {
        return Err(bad(format!(
            "{} bytes exceeds {MAX_ACCOUNT_BYTES}",
            bytes.len()
        )));
    }
    let acc = RegionAccount::try_from_slice(bytes).map_err(|e| bad(format!("decode: {e}")))?;
    if acc.layout != 1 {
        return Err(bad(format!("unknown layout {}", acc.layout)));
    }
    if acc.region != expected_region {
        return Err(bad(format!(
            "holds {:?}, expected {expected_region:?}",
            acc.region
        )));
    }
    let def = osm_core::lookup(&acc.region).ok_or_else(|| bad("region not in set".into()))?;
    let level = match acc.level {
        0 => Level::Country,
        1 => Level::Subregion,
        l => return Err(bad(format!("level {l}"))),
    };
    if def.level != level || def.parent.unwrap_or("") != acc.parent {
        return Err(bad("parent/level differ from the predefined set".into()));
    }
    if acc.history.len() > MAX_HISTORY {
        return Err(bad(format!("{} history entries", acc.history.len())));
    }
    for e in &acc.history {
        if e.hosts.len() > MAX_HOSTS {
            return Err(bad(format!("{} hosts on {}", e.hosts.len(), e.cid)));
        }
        validate_cid(&e.cid).map_err(|_| bad(format!("invalid CID {:?}", e.cid)))?;
    }
    Ok(acc)
}

/// A registry entry with its region, for listings and queries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredEntry {
    pub region: &'static str,
    pub entry: Entry,
}

/// Sorts by timestamp, then region, then CID (P-R3).
pub fn sort_entries(v: &mut [RegisteredEntry]) {
    v.sort_by(|a, b| {
        (a.entry.timestamp, a.region, &a.entry.cid).cmp(&(
            b.entry.timestamp,
            b.region,
            &b.entry.cid,
        ))
    });
}

/// Read access to the registry. `fetch` returns the program's shard bytes of an account, or
/// `None` when the region has never been registered.
pub trait RegistrySource {
    fn fetch(&self, account: &str) -> Result<Option<Vec<u8>>, SdkError>;
}

/// JSON-RPC reader against a LEZ sequencer (`getAccount`).
pub struct RpcRegistry {
    rpc: String,
    program: String,
    agent: ureq::Agent,
}

impl RpcRegistry {
    pub fn new(rpc: &str, program: &str) -> Result<Self, SdkError> {
        decode_account_id(program)?;
        Ok(RpcRegistry {
            rpc: rpc.to_string(),
            program: program.to_string(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(10))
                .timeout_read(Duration::from_secs(30))
                .build(),
        })
    }

    pub fn program(&self) -> &str {
        &self.program
    }
}

impl RegistrySource for RpcRegistry {
    fn fetch(&self, account: &str) -> Result<Option<Vec<u8>>, SdkError> {
        let body = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "getAccount", "params": [account]});
        let resp = self
            .agent
            .post(&self.rpc)
            .set("content-type", "application/json")
            .send_string(&body.to_string())
            .map_err(|e| SdkError::RpcUnreachable {
                url: self.rpc.clone(),
                cause: e.to_string(),
            })?;
        let mut raw = Vec::new();
        resp.into_reader()
            .take(MAX_RPC_BODY)
            .read_to_end(&mut raw)
            .map_err(|e| SdkError::RpcUnreachable {
                url: self.rpc.clone(),
                cause: e.to_string(),
            })?;
        let v: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|e| SdkError::RegistryDataInvalid {
                account: account.to_string(),
                why: format!("RPC answer is not JSON: {e}"),
            })?;
        if let Some(err) = v.get("error") {
            return Err(SdkError::RegistryDataInvalid {
                account: account.to_string(),
                why: format!("RPC error: {err}"),
            });
        }
        let shard = &v["result"]["data"]["shards"][&self.program];
        if shard.is_null() {
            return Ok(None);
        }
        let arr = shard
            .as_array()
            .ok_or_else(|| SdkError::RegistryDataInvalid {
                account: account.to_string(),
                why: "shard is not a byte array".into(),
            })?;
        if arr.len() > MAX_ACCOUNT_BYTES {
            return Err(SdkError::RegistryDataInvalid {
                account: account.to_string(),
                why: format!("{} bytes exceeds {MAX_ACCOUNT_BYTES}", arr.len()),
            });
        }
        arr.iter()
            .map(|b| b.as_u64().and_then(|b| u8::try_from(b).ok()))
            .collect::<Option<Vec<u8>>>()
            .map(Some)
            .ok_or_else(|| SdkError::RegistryDataInvalid {
                account: account.to_string(),
                why: "shard holds a value that is not a byte".into(),
            })
    }
}

/// Registry queries over any source (P-F7: by region, by parent, by CID).
pub struct Registry<S> {
    pub source: S,
    pub program: String,
}

impl<S: RegistrySource> Registry<S> {
    /// One region's account, or `None` when it was never registered.
    pub fn region(&self, region: &str) -> Result<Option<RegionAccount>, SdkError> {
        let account = region_account_id(&self.program, region)?;
        match self.source.fetch(&account)? {
            None => Ok(None),
            Some(bytes) if bytes.is_empty() => Ok(None),
            Some(bytes) => decode_region(&bytes, region, &account).map(Some),
        }
    }

    /// Every entry of every region whose predefined position matches `filter`, sorted by
    /// timestamp. Malformed accounts are reported, not silently dropped.
    fn collect(
        &self,
        filter: impl Fn(&osm_core::RegionDef) -> bool,
    ) -> (Vec<RegisteredEntry>, Vec<SdkError>) {
        let mut out = Vec::new();
        let mut errors = Vec::new();
        for def in PREDEFINED.iter().filter(|d| filter(d)) {
            match self.region(def.region) {
                Ok(Some(acc)) => out.extend(acc.history.into_iter().map(|entry| RegisteredEntry {
                    region: def.region,
                    entry,
                })),
                Ok(None) => {}
                Err(e) => errors.push(e),
            }
        }
        sort_entries(&mut out);
        (out, errors)
    }

    pub fn all(&self) -> (Vec<RegisteredEntry>, Vec<SdkError>) {
        self.collect(|_| true)
    }

    pub fn by_parent(&self, parent: &str) -> (Vec<RegisteredEntry>, Vec<SdkError>) {
        self.collect(|d| d.parent == Some(parent))
    }

    pub fn by_cid(&self, cid: &str) -> (Vec<RegisteredEntry>, Vec<SdkError>) {
        let (mut v, e) = self.all();
        v.retain(|r| r.entry.cid == cid);
        (v, e)
    }
}
