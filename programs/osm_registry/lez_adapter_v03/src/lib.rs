//! LEZ v0.3 write path for the LP-0018 registry.
//!
//! Transactions are built by the client SPEL generated from the IDL
//! (`osm_registry_ffi/generated/osm_registry_client.rs`). That client returns as soon as the
//! sequencer accepts a transaction, and on LEZ v0.3 an accepted transaction can still change
//! nothing (docs/status/phase-B.md). So every write here is confirmed by reading the region
//! account back through `osm_sdk::registry`; only a visible change counts as success (H33).

use std::time::{Duration, Instant};

use osm_sdk::registry::{Registry, RpcRegistry};
use wallet::WalletCore;

#[allow(dead_code, unused_imports, unused_mut, clippy::all)]
mod client {
    include!(concat!(env!("OUT_DIR"), "/osm_registry_client.rs"));
}

pub use client::OsmRegistryClient;

/// One entry to register. `timestamp_ms` is filled in at submit time.
#[derive(Clone, Debug)]
pub struct NewEntry {
    pub region: String,
    pub cid: String,
    pub source_url: String,
    /// 32 lowercase hex characters.
    pub checksum: String,
    pub version: u32,
    pub size_bytes: u64,
}

/// Outcome of a confirmed write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The transaction was sent and the change is visible on-chain.
    Confirmed { tx_hash: String },
    /// The signer already hosts this CID for every region; nothing was sent, no fee spent.
    AlreadyRegistered,
}

/// Errors with the codes from docs/error-codes.md.
#[derive(Debug)]
pub enum AdapterError {
    /// S103: sent, but the change is not visible after the timeout (not included, or rejected
    /// during execution).
    TxNotIncluded { tx_hash: String, secs: u64 },
    /// S105: the paying account cannot cover the fee the sequencer reserves up front.
    InsufficientFunds { have: u128, need: u128 },
    /// S104: the sequencer refused the transaction.
    TxRejected(String),
    /// S106: wallet could not be opened or holds no key for the signer.
    Wallet(String),
    /// Any SDK error (validation S160, RPC S100, registry data S170, ...).
    Sdk(osm_sdk::SdkError),
}

impl AdapterError {
    pub fn code(&self) -> &'static str {
        match self {
            AdapterError::TxNotIncluded { .. } => "S103",
            AdapterError::TxRejected(_) => "S104",
            AdapterError::InsufficientFunds { .. } => "S105",
            AdapterError::Wallet(_) => "S106",
            AdapterError::Sdk(e) => e.code(),
        }
    }
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::TxNotIncluded { tx_hash, secs } => write!(
                f,
                "S103 TxNotIncluded: transaction {tx_hash} was sent but the registry shows no change after {secs} s. Nothing was registered."
            ),
            AdapterError::TxRejected(m) => write!(f, "S104 TxRejected: the network rejected the transaction: {m}"),
            AdapterError::InsufficientFunds { have, need } => write!(
                f,
                "S105 InsufficientFunds: the paying account has {have} lepta; about {need} lepta must be available for this transaction's fee reserve. Claim from the faucet or fund it first."
            ),
            AdapterError::Wallet(m) => write!(f, "S106 Wallet: {m}"),
            AdapterError::Sdk(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<osm_sdk::SdkError> for AdapterError {
    fn from(e: osm_sdk::SdkError) -> Self {
        AdapterError::Sdk(e)
    }
}

pub struct Adapter {
    wallet: WalletCore,
    program_b58: String,
    rpc: String,
    pub confirm_timeout: Duration,
}

/// Fee the sequencer reserves before running a transaction, with the gas limit the SPEL client
/// uses (`SPEL_GAS_LIMIT`, default 500,000): `(gas_limit + 100,000) * 64` lepta. Measured actual
/// fees on LEZ v0.3.1 were far lower (register 414,960 lepta), but the reserve must be available.
pub fn fee_reserve() -> u128 {
    let gas_limit: u64 = std::env::var("SPEL_GAS_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500_000);
    (u128::from(gas_limit) + 100_000) * 64
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl Adapter {
    /// Opens the wallet from `LEE_WALLET_HOME_DIR`.
    pub async fn from_env(rpc: &str, program_b58: &str) -> Result<Self, AdapterError> {
        osm_sdk::registry::decode_account_id(program_b58)?;
        let wallet = WalletCore::from_env()
            .await
            .map_err(|e| AdapterError::Wallet(e.to_string()))?;
        Ok(Adapter {
            wallet,
            program_b58: program_b58.to_string(),
            rpc: rpc.to_string(),
            confirm_timeout: Duration::from_secs(120),
        })
    }

    /// Native balance of an account, in lepta.
    pub fn balance(&self, account_b58: &str) -> Result<u128, AdapterError> {
        Ok(osm_sdk::registry::account_balance(&self.rpc, account_b58)?)
    }

    fn ensure_funds(&self, payer_b58: &str) -> Result<(), AdapterError> {
        let have = self.balance(payer_b58)?;
        let need = fee_reserve();
        if have < need {
            return Err(AdapterError::InsufficientFunds { have, need });
        }
        Ok(())
    }

    fn registry(&self) -> Result<Registry<RpcRegistry>, AdapterError> {
        Ok(Registry {
            source: RpcRegistry::new(&self.rpc, &self.program_b58)?,
            program: self.program_b58.clone(),
        })
    }

    fn hosts(&self, region: &str, cid: &str, signer: &[u8; 32]) -> Result<bool, AdapterError> {
        Ok(self
            .registry()?
            .region(region)?
            .map(|a| {
                a.history
                    .iter()
                    .any(|e| e.cid == cid && e.hosts.contains(signer))
            })
            .unwrap_or(false))
    }

    async fn wait_until(
        &self,
        tx_hash: String,
        done: impl Fn() -> Result<bool, AdapterError>,
    ) -> Result<Outcome, AdapterError> {
        let start = Instant::now();
        loop {
            if done()? {
                return Ok(Outcome::Confirmed { tx_hash });
            }
            if start.elapsed() >= self.confirm_timeout {
                return Err(AdapterError::TxNotIncluded {
                    tx_hash,
                    secs: self.confirm_timeout.as_secs(),
                });
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    /// Registers (or mirrors) up to `MAX_BATCH` entries as `host` and waits until every one is
    /// visible with `host` among its hosts. One entry uses `register`, more use `register_batch`.
    pub async fn register(
        &self,
        host_b58: &str,
        entries: &[NewEntry],
    ) -> Result<Outcome, AdapterError> {
        let host = osm_sdk::registry::decode_account_id(host_b58)?;
        let ts = now_ms();
        // Validate everything locally first, with the program's own rules (osm_core).
        let names: Vec<&str> = entries.iter().map(|e| e.region.as_str()).collect();
        osm_core::entry::validate_batch_shape(&names, &[]).map_err(|e| {
            osm_sdk::SdkError::InvalidInput {
                field: "batch",
                why: format!("{} {e:?}", e.code()),
            }
        })?;
        let mut pending = Vec::new();
        for e in entries {
            let def =
                osm_core::lookup(&e.region).ok_or_else(|| osm_sdk::SdkError::InvalidInput {
                    field: "region",
                    why: format!("{:?} is not in the predefined set", e.region),
                })?;
            osm_core::entry::validate_entry(&osm_core::entry::EntryInput {
                region: &e.region,
                parent: def.parent,
                level: def.level,
                cid: &e.cid,
                source_url: &e.source_url,
                checksum_hex: &e.checksum,
                version: e.version,
                size_bytes: e.size_bytes,
                timestamp_ms: ts,
            })
            .map_err(|err| osm_sdk::SdkError::InvalidInput {
                field: "entry",
                why: format!("{}: {} {err:?}", e.region, err.code()),
            })?;
            if !self.hosts(&e.region, &e.cid, &host)? {
                pending.push((e.clone(), def));
            }
        }
        if pending.is_empty() {
            return Ok(Outcome::AlreadyRegistered);
        }
        self.ensure_funds(host_b58)?;
        let program =
            client::parse_program_id_hex(&self.program_b58).map_err(AdapterError::TxRejected)?;
        let signer = nssa::AccountId::new(host);
        let c = OsmRegistryClient::new(&self.wallet, program);
        let level = |d: &osm_core::RegionDef| u8::from(d.level == osm_core::Level::Subregion);
        let tx_hash = if pending.len() == 1 {
            let (e, d) = &pending[0];
            c.register(
                client::RegisterAccounts {
                    region_account: client::compute_region_account_pda(&program, &e.region),
                    host: signer,
                },
                e.region.clone(),
                d.parent.unwrap_or("").to_string(),
                level(d),
                e.cid.clone(),
                e.source_url.clone(),
                e.checksum.clone(),
                e.version,
                e.size_bytes,
                ts,
            )
            .await
        } else {
            c.register_batch(
                client::RegisterBatchAccounts {
                    host: signer,
                    region_accounts: pending
                        .iter()
                        .map(|(e, _)| client::compute_region_account_pda(&program, &e.region))
                        .collect(),
                },
                pending.iter().map(|(e, _)| e.region.clone()).collect(),
                pending
                    .iter()
                    .map(|(_, d)| d.parent.unwrap_or("").to_string())
                    .collect(),
                pending.iter().map(|(_, d)| level(d)).collect(),
                pending.iter().map(|(e, _)| e.cid.clone()).collect(),
                pending.iter().map(|(e, _)| e.source_url.clone()).collect(),
                pending.iter().map(|(e, _)| e.checksum.clone()).collect(),
                pending.iter().map(|(e, _)| u64::from(e.version)).collect(),
                pending.iter().map(|(e, _)| e.size_bytes).collect(),
                ts,
            )
            .await
        }
        .map_err(AdapterError::TxRejected)?;
        self.wait_until(tx_hash, || {
            for (e, _) in &pending {
                if !self.hosts(&e.region, &e.cid, &host)? {
                    return Ok(false);
                }
            }
            Ok(true)
        })
        .await
    }

    /// Removes `host` from the hosts of `cid` in `region` and waits until that is visible.
    pub async fn unhost(
        &self,
        host_b58: &str,
        region: &str,
        cid: &str,
    ) -> Result<Outcome, AdapterError> {
        let host = osm_sdk::registry::decode_account_id(host_b58)?;
        if osm_core::lookup(region).is_none() {
            return Err(osm_sdk::SdkError::InvalidInput {
                field: "region",
                why: format!("{region:?} is not in the predefined set"),
            }
            .into());
        }
        if !self.hosts(region, cid, &host)? {
            return Ok(Outcome::AlreadyRegistered);
        }
        self.ensure_funds(host_b58)?;
        let program =
            client::parse_program_id_hex(&self.program_b58).map_err(AdapterError::TxRejected)?;
        let c = OsmRegistryClient::new(&self.wallet, program);
        let tx_hash = c
            .unhost(
                client::UnhostAccounts {
                    region_account: client::compute_region_account_pda(&program, region),
                    host: nssa::AccountId::new(host),
                },
                region.to_string(),
                cid.to_string(),
            )
            .await
            .map_err(AdapterError::TxRejected)?;
        self.wait_until(tx_hash, || Ok(!self.hosts(region, cid, &host)?))
            .await
    }
}
