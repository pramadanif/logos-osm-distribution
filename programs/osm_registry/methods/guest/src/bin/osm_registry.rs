#![no_main]

//! LP-0018 OSM registry program (LEZ v0.3, SPEL `vpavlin/spel` @ e43b59be).
//!
//! One shard per predefined region at PDA `[ "osm-region", region ]` (ADR 0001). Entries are
//! append-only with a bounded ring; a CID already in the ring gains the signer as a host
//! (ADR 0002). There is no admin, fee or init step (ADR 0003). Every rule that does not need
//! account data runs in the plan half through `osm_core`, so the SDK and the program share it.

use spel_framework::prelude::*;

risc0_zkvm::guest::entry!(main);

#[lez_program]
mod osm_registry {
    #[allow(unused_imports)]
    use super::*;
    use osm_core::entry::{
        validate_entry, EntryInput, MAX_HISTORY, MAX_HOSTS, TIMESTAMP_WINDOW_MS,
    };
    use osm_core::Level;

    /// Layout version of [`RegionAccount`].
    pub const LAYOUT_VERSION: u8 = 1;

    /// One registered copy of a region snapshot.
    #[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Eq)]
    #[borsh(crate = "spel_framework::borsh")]
    pub struct Entry {
        pub cid: String,
        pub source_url: String,
        pub checksum: [u8; 16],
        pub version: u32,
        pub size_bytes: u64,
        pub hosted: bool,
        /// Unix milliseconds, bound to block time by the validity window.
        pub timestamp: u64,
        pub registered_by: [u8; 32],
        pub hosts: Vec<[u8; 32]>,
    }

    /// State of one region's shard.
    #[derive(BorshSerialize, BorshDeserialize)]
    #[borsh(crate = "spel_framework::borsh")]
    #[account_type]
    pub struct RegionAccount {
        pub layout: u8,
        pub region: String,
        pub parent: String,
        /// 0 = country, 1 = subregion.
        pub level: u8,
        /// Oldest first; at most `MAX_HISTORY`.
        pub history: Vec<Entry>,
    }

    /// Effect payload of [`register`]. Built only from validated plan input and the signer.
    #[derive(BorshSerialize, BorshDeserialize)]
    #[borsh(crate = "spel_framework::borsh")]
    pub struct Register {
        pub region: String,
        pub parent: String,
        pub level: u8,
        pub entry: Entry,
    }

    /// Effect payload of [`unhost`].
    #[derive(BorshSerialize, BorshDeserialize)]
    #[borsh(crate = "spel_framework::borsh")]
    pub struct Unhost {
        pub cid: String,
        pub host: [u8; 32],
    }

    fn level_of(level: u8) -> Level {
        match level {
            0 => Level::Country,
            1 => Level::Subregion,
            _ => panic!("P002 PositionMismatch: level must be 0 or 1"),
        }
    }

    /// Register a snapshot of one region, or add the signer as a host of a CID already listed.
    #[plan]
    pub fn register(
        ctx: ProgramContext,
        #[account(mut, pda = [literal("osm-region"), arg("region")])]
        region_account: AccountMeta,
        #[account(signer)]
        host: AccountMeta,
        region: String,
        parent: String,
        level: u8,
        cid: String,
        source_url: String,
        checksum: String,
        version: u32,
        size_bytes: u64,
        timestamp: u64,
    ) -> SpelResult {
        let parent_opt = (!parent.is_empty()).then_some(parent.as_str());
        let valid = validate_entry(&EntryInput {
            region: &region,
            parent: parent_opt,
            level: level_of(level),
            cid: &cid,
            source_url: &source_url,
            checksum_hex: &checksum,
            version,
            size_bytes,
            timestamp_ms: timestamp,
        })
        .unwrap_or_else(|e| panic!("{} {:?}", e.code(), e));
        let signer = *host.account_id.value();
        let window = timestamp.saturating_sub(TIMESTAMP_WINDOW_MS)
            ..timestamp.saturating_add(TIMESTAMP_WINDOW_MS);
        let effect = Effect::ApplyRegister(Register {
            region,
            parent,
            level,
            entry: Entry {
                cid,
                source_url,
                checksum: valid.checksum,
                version: valid.version,
                size_bytes,
                hosted: true,
                timestamp,
                registered_by: signer,
                hosts: vec![signer],
            },
        });
        // `timestamp` is Unix milliseconds, the unit of LEZ block timestamps.
        // start < end always: validate_entry rejected timestamps below MIN_TIMESTAMP_MS.
        Ok(SpelOutput::empty()
            .effect(&region_account, ctx.self_account_id, &effect)
            .try_with_timestamp_validity_window(window)
            .expect("P008 TimestampOutOfRange: empty validity window"))
    }

    /// Register up to `MAX_BATCH` regions in one transaction (P-F7). Vectors are parallel: element
    /// `i` of every vector describes `region_accounts[i]`. Each account must be the PDA of its
    /// region, checked here because a `Vec<AccountMeta>` carries no seed constraint.
    #[plan]
    pub fn register_batch(
        ctx: ProgramContext,
        #[account(signer)]
        host: AccountMeta,
        // A `Vec<AccountMeta>` takes the remaining accounts, so it must be the last account param.
        #[account(mut)]
        region_accounts: Vec<AccountMeta>,
        regions: Vec<String>,
        parents: Vec<String>,
        levels: Vec<u8>,
        cids: Vec<String>,
        source_urls: Vec<String>,
        checksums: Vec<String>,
        versions: Vec<u64>,
        sizes: Vec<u64>,
        timestamp: u64,
    ) -> SpelResult {
        let names: Vec<&str> = regions.iter().map(String::as_str).collect();
        osm_core::entry::validate_batch_shape(
            &names,
            &[
                region_accounts.len(),
                parents.len(),
                levels.len(),
                cids.len(),
                source_urls.len(),
                checksums.len(),
                versions.len(),
                sizes.len(),
            ],
        )
        .unwrap_or_else(|e| panic!("{} {:?}", e.code(), e));
        let signer = *host.account_id.value();
        let mut out = SpelOutput::empty();
        for i in 0..regions.len() {
            let expected = spel_framework::pda::compute_pda_multi(
                &ctx.self_account_id,
                &[&"osm-region", &regions[i].as_str()],
            );
            assert!(
                region_accounts[i].account_id == expected,
                "P012 AccountMismatch: account {i} is not the PDA of {}",
                regions[i]
            );
            let version = u32::try_from(versions[i]).expect("P005 VersionMismatch: version overflows u32");
            let parent_opt = (!parents[i].is_empty()).then_some(parents[i].as_str());
            let valid = validate_entry(&EntryInput {
                region: &regions[i],
                parent: parent_opt,
                level: level_of(levels[i]),
                cid: &cids[i],
                source_url: &source_urls[i],
                checksum_hex: &checksums[i],
                version,
                size_bytes: sizes[i],
                timestamp_ms: timestamp,
            })
            .unwrap_or_else(|e| panic!("{} {:?} at {}", e.code(), e, regions[i]));
            let effect = Effect::ApplyRegister(Register {
                region: regions[i].clone(),
                parent: parents[i].clone(),
                level: levels[i],
                entry: Entry {
                    cid: cids[i].clone(),
                    source_url: source_urls[i].clone(),
                    checksum: valid.checksum,
                    version: valid.version,
                    size_bytes: sizes[i],
                    hosted: true,
                    timestamp,
                    registered_by: signer,
                    hosts: vec![signer],
                },
            });
            out = out.effect(&region_accounts[i], ctx.self_account_id, &effect);
        }
        let window = timestamp.saturating_sub(TIMESTAMP_WINDOW_MS)
            ..timestamp.saturating_add(TIMESTAMP_WINDOW_MS);
        Ok(out
            .try_with_timestamp_validity_window(window)
            .expect("P008 TimestampOutOfRange: empty validity window"))
    }

    /// Remove the signer from the hosts of `cid` in `region`.
    #[plan]
    pub fn unhost(
        ctx: ProgramContext,
        #[account(mut, pda = [literal("osm-region"), arg("region")])]
        region_account: AccountMeta,
        #[account(signer)]
        host: AccountMeta,
        region: String,
        cid: String,
    ) -> SpelResult {
        if osm_core::lookup(&region).is_none() {
            panic!("P001 RegionNotInSet");
        }
        let effect = Effect::ApplyUnhost(Unhost {
            cid,
            host: *host.account_id.value(),
        });
        Ok(SpelOutput::empty().effect(&region_account, ctx.self_account_id, &effect))
    }

    fn decode(pre: &ShardData) -> Option<RegionAccount> {
        let bytes: &[u8] = pre.as_ref();
        if bytes.is_empty() {
            return None;
        }
        Some(RegionAccount::try_from_slice(bytes).expect("region shard decodes"))
    }

    fn encode(acc: &RegionAccount) -> ShardData {
        let bytes = borsh::to_vec(acc).expect("RegionAccount serializes");
        ShardData::try_from(bytes).expect("RegionAccount fits under DATA_MAX_LENGTH")
    }

    #[apply]
    pub fn apply_register(effect: Register, pre_data: &ShardData) -> Option<ShardData> {
        let mut acc = decode(pre_data).unwrap_or(RegionAccount {
            layout: LAYOUT_VERSION,
            region: effect.region.clone(),
            parent: effect.parent.clone(),
            level: effect.level,
            history: Vec::new(),
        });
        assert!(acc.region == effect.region, "P012 AccountMismatch");
        let signer = effect.entry.registered_by;
        if let Some(e) = acc.history.iter_mut().find(|e| e.cid == effect.entry.cid) {
            if e.hosts.contains(&signer) {
                return None; // idempotent mirror add
            }
            assert!(e.hosts.len() < MAX_HOSTS, "P009 HostsFull");
            e.hosts.push(signer);
            e.hosted = true;
        } else {
            acc.history.push(effect.entry);
            if acc.history.len() > MAX_HISTORY {
                acc.history.remove(0);
            }
        }
        Some(encode(&acc))
    }

    #[apply]
    pub fn apply_unhost(effect: Unhost, pre_data: &ShardData) -> Option<ShardData> {
        let mut acc = decode(pre_data)?;
        let e = acc.history.iter_mut().find(|e| e.cid == effect.cid)?;
        let before = e.hosts.len();
        e.hosts.retain(|h| *h != effect.host);
        if e.hosts.len() == before {
            return None;
        }
        e.hosted = !e.hosts.is_empty();
        Some(encode(&acc))
    }
}
