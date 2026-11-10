//! LP-0018 OSM registry for LEZ 0.4 (actor model), step 1 of the port: no SPEL yet.
//!
//! Same rules and the same `RegionAccount` byte layout as the v0.3 program, so the SDK decoder is
//! shared. Flow (docs/status/lez-04-port.md):
//! 1. A transaction sends `Register` to the host's own actor under this program. The host signed,
//!    so that actor is authorized and its account id is the signer.
//! 2. The host actor validates the entry (osm_core), asks the clock program to assert the
//!    timestamp window, and `send_sync`s `Apply` to the region actor with the region's PDA seed.
//! 3. The region actor accepts `Apply` only when authorized through this program and sent by one
//!    of its own actors, and records the sender as `registered_by`.

use borsh::{BorshDeserialize, BorshSerialize};
use lee_core::account::{AccountId, Actor};
use lee_core::program::{PdaSeed, ReceiveInput, Response, SyncMessage};
use osm_core::entry::{validate_batch_shape, validate_entry, EntryInput, MAX_HISTORY, MAX_HOSTS, TIMESTAMP_WINDOW_MS};
use osm_core::Level;

lee_core::actor_entrypoint!(handle);

/// Messages of this program. Variant order is the wire format.
#[derive(BorshSerialize, BorshDeserialize)]
pub enum Msg {
    /// From a transaction, to the host's actor.
    Register { region: String, parent: String, level: u8, cid: String, source_url: String, checksum: String, version: u32, size_bytes: u64, timestamp: u64 },
    /// From a transaction, to the host's actor.
    Unhost { region: String, cid: String },
    /// From the host's actor, to the region actor.
    Apply { region: String, parent: String, level: u8, entry: Entry },
    /// From the host's actor, to the region actor.
    Remove { cid: String },
    /// From a transaction, to the host's actor: up to `MAX_BATCH` registrations, one clock check.
    RegisterBatch { entries: Vec<NewEntry>, timestamp: u64 },
}

/// One registration as a transaction carries it (the fields of `Msg::Register`).
#[derive(BorshSerialize, BorshDeserialize)]
pub struct NewEntry {
    pub region: String,
    pub parent: String,
    pub level: u8,
    pub cid: String,
    pub source_url: String,
    pub checksum: String,
    pub version: u32,
    pub size_bytes: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Eq)]
pub struct Entry {
    pub cid: String,
    pub source_url: String,
    pub checksum: [u8; 16],
    pub version: u32,
    pub size_bytes: u64,
    pub hosted: bool,
    pub timestamp: u64,
    pub registered_by: [u8; 32],
    pub hosts: Vec<[u8; 32]>,
}

/// Identical layout to the v0.3 program's account type.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct RegionAccount {
    pub layout: u8,
    pub region: String,
    pub parent: String,
    pub level: u8,
    pub history: Vec<Entry>,
}

fn region_seed(region: &str) -> PdaSeed {
    use risc0_zkvm::sha::{Impl, Sha256 as _};
    let mut both = [0u8; 64];
    both[..10].copy_from_slice(b"osm-region");
    both[32..32 + region.len()].copy_from_slice(region.as_bytes());
    PdaSeed::new(Impl::hash_bytes(&both).as_bytes().try_into().expect("32 bytes"))
}

fn region_actor(program: AccountId, region: &str) -> (Actor, PdaSeed) {
    let seed = region_seed(region);
    (Actor::new(AccountId::for_public_pda(&program, &seed), program), seed)
}

fn level_of(l: u8) -> Level {
    match l {
        0 => Level::Country,
        1 => Level::Subregion,
        _ => panic!("P002 PositionMismatch: level must be 0 or 1"),
    }
}

fn handle(input: &ReceiveInput, msg: Msg) -> Response {
    let program = input.receiver.program_account_id;
    match msg {
        Msg::Register { region, parent, level, cid, source_url, checksum, version, size_bytes, timestamp } => {
            // Only a transaction signed by the host may start a registration for that host.
            assert!(input.from.is_none() && input.is_authorized, "S104 host actor needs the host's signature");
            let e = NewEntry { region, parent, level, cid, source_url, checksum, version, size_bytes };
            let apply = apply_message(input, program, e, timestamp);
            assert_clock(Response::keep_state(), timestamp).send(apply)
        }
        Msg::RegisterBatch { entries, timestamp } => {
            assert!(input.from.is_none() && input.is_authorized, "S104 host actor needs the host's signature");
            let regions: Vec<&str> = entries.iter().map(|e| e.region.as_str()).collect();
            validate_batch_shape(&regions, &[]).unwrap_or_else(|e| panic!("{} {:?}", e.code(), e));
            // The batch shares one timestamp: the clock is asserted once for all entries.
            let mut r = assert_clock(Response::keep_state(), timestamp);
            for e in entries {
                let apply = apply_message(input, program, e, timestamp);
                r = r.send(apply);
            }
            r
        }
        Msg::Unhost { region, cid } => {
            assert!(input.from.is_none() && input.is_authorized, "S104 host actor needs the host's signature");
            assert!(osm_core::lookup(&region).is_some(), "P001 RegionNotInSet");
            let (to, seed) = region_actor(program, &region);
            Response::keep_state().send(SyncMessage::new(to, &Msg::Remove { cid }).with_pda_seeds([seed]))
        }
        Msg::Apply { region, parent, level, entry } => {
            let host = own_sender(input);
            assert!(entry.registered_by == host && entry.hosts == [host], "P012 entry not from its sender");
            assert!(input.receiver.account_id == region_actor(program, &region).0.account_id, "P012 AccountMismatch");
            let mut acc = if input.pre_state.is_empty() {
                RegionAccount { layout: 1, region: region.clone(), parent, level, history: Vec::new() }
            } else {
                borsh::from_slice(&input.pre_state).expect("region state decodes")
            };
            assert!(acc.region == region, "P012 AccountMismatch");
            if let Some(e) = acc.history.iter_mut().find(|e| e.cid == entry.cid) {
                if e.hosts.contains(&host) {
                    return Response::keep_state();
                }
                assert!(e.hosts.len() < MAX_HOSTS, "P009 HostsFull");
                e.hosts.push(host);
                e.hosted = true;
            } else {
                acc.history.push(entry);
                if acc.history.len() > MAX_HISTORY {
                    acc.history.remove(0);
                }
            }
            Response::set_state(borsh::to_vec(&acc).expect("serializes"))
        }
        Msg::Remove { cid } => {
            let host = own_sender(input);
            if input.pre_state.is_empty() {
                return Response::keep_state();
            }
            let mut acc: RegionAccount = borsh::from_slice(&input.pre_state).expect("region state decodes");
            let Some(e) = acc.history.iter_mut().find(|e| e.cid == cid) else { return Response::keep_state() };
            let before = e.hosts.len();
            e.hosts.retain(|h| *h != host);
            if e.hosts.len() == before {
                return Response::keep_state();
            }
            e.hosted = !e.hosts.is_empty();
            Response::set_state(borsh::to_vec(&acc).expect("serializes"))
        }
    }
}

/// Validates one registration (osm_core) and builds the `Apply` for its region actor. The host is
/// the signer: the receiving host actor's account id.
fn apply_message(input: &ReceiveInput, program: AccountId, e: NewEntry, timestamp: u64) -> SyncMessage {
    assert!(e.region.len() <= 32, "P001 RegionNotInSet");
    let valid = validate_entry(&EntryInput {
        region: &e.region,
        parent: (!e.parent.is_empty()).then_some(e.parent.as_str()),
        level: level_of(e.level),
        cid: &e.cid,
        source_url: &e.source_url,
        checksum_hex: &e.checksum,
        version: e.version,
        size_bytes: e.size_bytes,
        timestamp_ms: timestamp,
    })
    .unwrap_or_else(|err| panic!("{} {:?}", err.code(), err));
    let host = *input.receiver.account_id.value();
    let (to, seed) = region_actor(program, &e.region);
    let entry = Entry {
        cid: e.cid,
        source_url: e.source_url,
        checksum: valid.checksum,
        version: valid.version,
        size_bytes: e.size_bytes,
        hosted: true,
        timestamp,
        registered_by: host,
        hosts: vec![host],
    };
    SyncMessage::new(to, &Msg::Apply { region: e.region, parent: e.parent, level: e.level, entry }).with_pda_seeds([seed])
}

/// Asks the clock program to check that `timestamp` is within the window around block time.
fn assert_clock(r: Response, timestamp: u64) -> Response {
    r.send_sync(
        Actor::new(clock_core::CLOCK_01_PROGRAM_ACCOUNT_ID, clock_core::clock_account_id()),
        &clock_core::Message::AssertTimestamp {
            at_least: timestamp.saturating_sub(TIMESTAMP_WINDOW_MS),
            at_most: timestamp.saturating_add(TIMESTAMP_WINDOW_MS),
        },
    )
}

/// The host account behind an internal message: authorized through this program and sent by one
/// of this program's actors. A transaction sent straight to a region actor has no sender.
fn own_sender(input: &ReceiveInput) -> [u8; 32] {
    assert!(input.is_authorized && input.from_own_program(), "S104 region actors accept messages only from this program's host actors");
    *input.from.expect("checked by from_own_program").account_id.value()
}
