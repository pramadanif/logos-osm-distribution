//! Step-1 client for the LEZ 0.4 actor registry (docs/status/lez-04-port.md).
//!
//!     LEE_WALLET_HOME_DIR=<wallet> osm-chain-v04 register <program> <host> <region> <parent> <level> \
//!         <cid> <source_url> <md5> <YYMMDD> <size>
//!     LEE_WALLET_HOME_DIR=<wallet> osm-chain-v04 region-account <program> <region>
//!
//! The transaction goes to the host's own actor under the program (the host signs); the program
//! forwards the entry to the region actor. Every actor the transaction touches is declared.
use std::collections::BTreeSet;

use anyhow::{anyhow, bail, Result};
use borsh::BorshSerialize;
use lee_core::account::AccountId;
use lee_core::program::PdaSeed;
use sha2::{Digest, Sha256};
use wallet::{AccountIdentity, AccountMention};

#[derive(BorshSerialize)]
enum Msg {
    Register { region: String, parent: String, level: u8, cid: String, source_url: String, checksum: String, version: u32, size_bytes: u64, timestamp: u64 },
    #[allow(dead_code)]
    Unhost { region: String, cid: String },
}

fn account(b58: &str) -> Result<AccountId> {
    let b: [u8; 32] = bs58::decode(b58.trim_start_matches("Public/")).into_vec()?.try_into().map_err(|_| anyhow!("{b58}: not 32 bytes"))?;
    Ok(AccountId::new(b))
}

fn region_seed(region: &str) -> PdaSeed {
    let mut both = [0u8; 64];
    both[..10].copy_from_slice(b"osm-region");
    both[32..32 + region.len()].copy_from_slice(region.as_bytes());
    PdaSeed::new(Sha256::digest(both).into())
}

fn mention(identity: AccountIdentity, program: AccountId, authorizes: bool) -> AccountMention {
    AccountMention { identity, program_account_id: program, authorizes, address_openings: BTreeSet::new() }
}

#[tokio::main]
async fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("region-account") => {
            let program = account(&a[2])?;
            println!("{}", bs58::encode(AccountId::for_public_pda(&program, &region_seed(&a[3])).value()).into_string());
            Ok(())
        }
        Some("register") if a.len() == 12 => {
            let program = account(&a[2])?;
            let host = account(&a[3])?;
            let region = a[4].clone();
            let region_account = AccountId::for_public_pda(&program, &region_seed(&region));
            let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64;
            let msg = Msg::Register {
                region,
                parent: a[5].clone(),
                level: a[6].parse()?,
                cid: a[7].clone(),
                source_url: a[8].clone(),
                checksum: a[9].clone(),
                version: a[10].parse()?,
                size_bytes: a[11].parse()?,
                timestamp: now_ms,
            };
            let accounts = vec![
                mention(AccountIdentity::Public(host), program, true),
                mention(AccountIdentity::PublicNoSign(region_account), program, false),
                mention(AccountIdentity::PublicNoSign(clock_core::CLOCK_01_PROGRAM_ACCOUNT_ID), clock_core::clock_account_id(), false),
            ];
            let w = wallet::WalletCore::from_env()?;
            let tx = w.send_pub_tx(accounts, 0, borsh::to_vec(&msg)?).await.map_err(|e| anyhow!("{e:?}"))?;
            println!("tx {}", hex_of(tx.as_ref()));
            Ok(())
        }
        _ => bail!("usage: osm-chain-v04 register <program> <host> <region> <parent> <level> <cid> <source_url> <md5> <YYMMDD> <size> | region-account <program> <region>"),
    }
}

fn hex_of(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
