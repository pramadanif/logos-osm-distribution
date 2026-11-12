//! Executor cycle counts of the LEZ 0.4 actor registry (P-P1, H8), measured the way LEZ
//! v0.4.0-rc1's `tools/cycle_bench` does it: the whole delivery runs through
//! `WholeExecution` (host actor -> clock -> region actors), and every transition that runs our
//! guest or the clock guest is executed in the RISC Zero executor with the input the sequencer
//! would give it (`Program::write_receive_input`). One row per transition, because a caller pays
//! for all of them.
//!
//!     cycle-bench-v04 <osm_registry_v04.bin> > cycles-v04.json
//!
//! `RISC0_DEV_MODE` has no effect on executor cycle counts.
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Instant;

use anyhow::{anyhow, bail, Result};
use borsh::BorshSerialize;
use clock_core::{ClockAccountData, CLOCK_01_PROGRAM_ACCOUNT_ID, CLOCK_10_PROGRAM_ACCOUNT_ID, CLOCK_50_PROGRAM_ACCOUNT_ID};
use lee::program::Program;
use lee_core::account::{AccountId, Actor, ActorState};
use lee_core::execution_state::{ExecutionEnvironment, PublicExecutionContext, TransactionEntry, TransitionView, WholeExecution};
use lee_core::native_token::{self, NATIVE_TOKEN_PROGRAM_ID};
use lee_core::program::{PdaSeed, ReceiveInput, Transition};
use lee_core::{from_frame, RootCall, Timestamp};
use risc0_zkvm::{default_executor, ExecutorEnv};
use sha2::{Digest, Sha256};

/// Wire format of the program's `Msg` (variant order is the wire format).
#[derive(BorshSerialize)]
#[allow(dead_code)]
enum Msg {
    Register { region: String, parent: String, level: u8, cid: String, source_url: String, checksum: String, version: u32, size_bytes: u64, timestamp: u64 },
    Unhost { region: String, cid: String },
    Apply,
    Remove,
    RegisterBatch { entries: Vec<NewEntry>, timestamp: u64 },
}

#[derive(BorshSerialize, Clone)]
struct NewEntry {
    region: String,
    parent: String,
    level: u8,
    cid: String,
    source_url: String,
    checksum: String,
    version: u32,
    size_bytes: u64,
}

const CID: &str = "zDvZRwzmA5LrY5Be279q4gpJqxk6v6XrS59rHkKakguRwwFPEMfH";
const TS: u64 = 1_794_198_446_642;
/// Where the bench pretends the registry is deployed; any id works, it only routes deliveries.
const PROGRAM: [u8; 32] = [7; 32];
const HOST: [u8; 32] = [9; 32];

fn program_id() -> AccountId {
    AccountId::new(PROGRAM)
}

fn region_actor(region: &str) -> Actor {
    let mut both = [0u8; 64];
    both[..10].copy_from_slice(b"osm-region");
    both[32..32 + region.len()].copy_from_slice(region.as_bytes());
    let seed = PdaSeed::new(Sha256::digest(both).into());
    Actor::new(AccountId::for_public_pda(&program_id(), &seed), program_id())
}

fn entry(region: &str) -> NewEntry {
    let def = osm_core::lookup(region).expect("bench regions are in the set");
    NewEntry {
        region: region.into(),
        parent: def.parent.unwrap_or("").into(),
        level: u8::from(def.level == osm_core::Level::Subregion),
        cid: CID.into(),
        source_url: osm_sdk::snapshot::snapshot_url(def, 261101).expect("dated url"),
        checksum: "0123456789abcdef0123456789abcdef".into(),
        version: 261101,
        size_bytes: 1_000_000,
    }
}

struct Meter<'a> {
    label: String,
    registry: &'a Program,
    clock: Program,
    states: HashMap<Actor, ActorState>,
    rows: Vec<serde_json::Value>,
}

impl ExecutionEnvironment for Meter<'_> {
    type Error = anyhow::Error;

    fn handle_message(&mut self, input: &ReceiveInput, _: &TransitionView<'_>) -> Result<Transition> {
        let pid = input.receiver.program_account_id;
        if pid == NATIVE_TOKEN_PROGRAM_ID {
            return Ok(native_token::handle_message(input)?);
        }
        let (name, program) = if pid == program_id() {
            ("osm_registry_v04", self.registry)
        } else if pid == clock_core::clock_account_id() {
            ("clock", &self.clock)
        } else {
            bail!("no guest registered at {pid}")
        };
        let mut b = ExecutorEnv::builder();
        Program::write_receive_input(input, &mut b)?;
        let t = Instant::now();
        let info = default_executor().execute(b.build()?, program.elf())?;
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let padded: u64 = info.segments.iter().map(|s| 1u64 << s.po2).sum();
        self.rows.push(serde_json::json!({
            "case": self.label, "program": name,
            "receiver": if input.receiver.account_id == AccountId::new(HOST) { "host actor".to_string() }
                        else if pid == program_id() { "region actor".to_string() } else { "clock".to_string() },
            "user_cycles": info.cycles(), "padded_cycles": padded, "segments": info.segments.len(), "exec_ms": ms,
        }));
        let payload = from_frame(&info.journal.bytes).ok_or_else(|| anyhow!("malformed journal frame"))?;
        Ok(borsh::from_slice::<Transition>(payload)?)
    }

    fn public_actor_state(&mut self, actor: Actor) -> Result<ActorState> {
        Ok(self.states.get(&actor).cloned().unwrap_or_default())
    }

    fn admits(&mut self, _: AccountId) -> Result<bool> {
        Ok(true)
    }
}

/// Runs one transaction: `msg` to the host actor (signed), with the clock accounts and every
/// region actor declared. `pre` holds region states that already exist.
fn case(registry: &Program, label: String, regions: &[&str], msg: &Msg, pre: &HashMap<Actor, ActorState>) -> Result<(Vec<serde_json::Value>, HashMap<Actor, ActorState>)> {
    let host = Actor::new(AccountId::new(HOST), program_id());
    let clock_state: ActorState = ClockAccountData { block_id: 100, timestamp: Timestamp::from(TS) }.to_bytes().into();
    let clocks = [CLOCK_01_PROGRAM_ACCOUNT_ID, CLOCK_10_PROGRAM_ACCOUNT_ID, CLOCK_50_PROGRAM_ACCOUNT_ID]
        .map(|id| Actor::new(id, clock_core::clock_account_id()));
    let mut actors: std::collections::BTreeSet<Actor> = [host].into_iter().collect();
    actors.extend(regions.iter().map(|r| region_actor(r)));
    actors.extend(clocks);
    let mut states = pre.clone();
    for c in clocks {
        states.insert(c, clock_state.clone());
    }
    let ctx = PublicExecutionContext { actors, authorized_accounts: vec![host.account_id].into_iter().collect(), ..PublicExecutionContext::default() };
    let mut meter = Meter { label, registry, clock: programs::clock(), states, rows: Vec::new() };
    let outcome = WholeExecution::new(ctx, TransactionEntry::Sync(RootCall { to: host, message: borsh::to_vec(msg)? }), Vec::new(), None, &[])?
        .execute(&mut meter)?;
    // Keep the region states the run produced, so a later case (unhost) can start from them.
    let mut after = pre.clone();
    for r in regions {
        let a = region_actor(r);
        if let Some(data) = outcome.public.accounts.get(&a.account_id) {
            after.insert(a, data.actor_state(program_id()).clone());
        }
    }
    Ok((meter.rows, after))
}

fn main() -> Result<()> {
    let path = std::env::args().nth(1).ok_or_else(|| anyhow!("usage: cycle-bench-v04 <osm_registry_v04.bin>"))?;
    let registry = Program::new(Cow::Owned(std::fs::read(&path)?))?;
    let image_id: String = registry.id().iter().flat_map(|w| w.to_le_bytes()).map(|b| format!("{b:02x}")).collect();
    let regions: Vec<&str> = osm_core::PREDEFINED.iter().filter(|d| d.level == osm_core::Level::Country).map(|d| d.region).take(16).collect();
    let mut rows = Vec::new();
    let empty = HashMap::new();

    let e = entry(regions[0]);
    let msg = Msg::Register { region: e.region, parent: e.parent, level: e.level, cid: e.cid, source_url: e.source_url, checksum: e.checksum, version: e.version, size_bytes: e.size_bytes, timestamp: TS };
    let (r, after) = case(&registry, "register".into(), &regions[..1], &msg, &empty)?;
    rows.extend(r);

    let (r, _) = case(&registry, "unhost".into(), &regions[..1], &Msg::Unhost { region: regions[0].into(), cid: CID.into() }, &after)?;
    rows.extend(r);

    for n in [1usize, 5, 10, 16] {
        let msg = Msg::RegisterBatch { entries: regions[..n].iter().map(|r| entry(r)).collect(), timestamp: TS };
        // A guest built before RegisterBatch existed refuses the message; record that instead
        // of stopping, so register/unhost numbers are still reported for it.
        match case(&registry, format!("register_batch_{n}"), &regions[..n], &msg, &empty) {
            Ok((r, _)) => rows.extend(r),
            Err(e) => rows.push(serde_json::json!({"case": format!("register_batch_{n}"), "error": e.to_string()})),
        }
    }
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({"image_id": image_id, "rows": rows}))?);
    Ok(())
}
