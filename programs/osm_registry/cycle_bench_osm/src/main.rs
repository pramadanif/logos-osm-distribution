//! Executor cycle counts for each registry instruction (P-P1, H8), measured the way LEZ's
//! `tools/cycle_bench` does it: run the program ELF in the RISC Zero executor with the same
//! plan and apply inputs the sequencer gives it, and report user cycles per phase.
//!
//!     cargo run --release -p cycle_bench_osm -- <osm_registry.bin> > cycles.json
use std::collections::HashMap;
use std::time::Instant;

use anyhow::{bail, Result};
use borsh::BorshSerialize;
use nssa::program::Program;
use nssa_core::{
    account::{AccountId, ProgramShardSelector, ShardData},
    from_frame,
    program::{AccountMeta, ApplyInput, GuestOutput, PlanInput},
};
use risc0_zkvm::{default_executor, ExecutorEnv};

/// Same layout as `OsmRegistryInstruction` in the program (borsh, variant order of the IDL).
#[derive(BorshSerialize)]
enum Instruction {
    Register { region: String, parent: String, level: u8, cid: String, source_url: String, checksum: String, version: u32, size_bytes: u64, timestamp: u64 },
    RegisterBatch { regions: Vec<String>, parents: Vec<String>, levels: Vec<u8>, cids: Vec<String>, source_urls: Vec<String>, checksums: Vec<String>, versions: Vec<u64>, sizes: Vec<u64>, timestamp: u64 },
    #[allow(dead_code)]
    Unhost { region: String, cid: String },
}

const CID: &str = "zDvZRwzmA5LrY5Be279q4gpJqxk6v6XrS59rHkKakguRwwFPEMfH";
const TS: u64 = 1_794_198_446_642;

fn pda(program: &AccountId, region: &str) -> AccountId {
    // SPEL seeds ["osm-region", arg region]: sha256(pad32("osm-region") || pad32(region)).
    use risc0_zkvm::sha::{Impl, Sha256 as _};
    let pad = |s: &str| { let mut b = [0u8; 32]; b[..s.len()].copy_from_slice(s.as_bytes()); b };
    let mut both = [0u8; 64];
    both[..32].copy_from_slice(&pad("osm-region"));
    both[32..].copy_from_slice(&pad(region));
    let seed: [u8; 32] = Impl::hash_bytes(&both).as_bytes().try_into().unwrap();
    AccountId::for_public_pda(program, &nssa_core::program::PdaSeed::new(seed))
}

/// Returns (journal, user cycles, padded segment cycles = sum of 2^po2, segments, wall ms).
fn exec(program: &Program, write: impl Fn(&mut risc0_zkvm::ExecutorEnvBuilder) -> Result<()>) -> Result<(Vec<u8>, u64, u64, u64, f64)> {
    let mut b = ExecutorEnv::builder();
    write(&mut b)?;
    let t = Instant::now();
    let info = default_executor().execute(b.build()?, program.elf())?;
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let user: u64 = info.segments.iter().map(|s| u64::from(s.cycles)).sum();
    let padded: u64 = info.segments.iter().map(|s| 1u64 << s.po2).sum();
    Ok((info.journal.bytes, user, padded, info.segments.len() as u64, ms))
}

fn out(journal: &[u8]) -> Result<GuestOutput> {
    Ok(borsh::from_slice(from_frame(journal).ok_or_else(|| anyhow::anyhow!("bad frame"))?)?)
}

/// Runs plan, then apply for every effect; returns one JSON row per phase.
fn case(program: &Program, self_id: AccountId, label: &str, accounts: Vec<AccountMeta>, ix: &Instruction, pre: &HashMap<ProgramShardSelector, ShardData>) -> Result<Vec<serde_json::Value>> {
    let input = PlanInput { self_account_id: self_id, caller_account_id: None, accounts, instruction_data: borsh::to_vec(ix)? };
    let mut rows = Vec::new();
    let (journal, user, padded, segs, ms) = exec(program, |env| Ok(Program::write_plan_inputs(&input, env)?))?;
    rows.push(serde_json::json!({"instruction": label, "phase": "plan", "user_cycles": user, "padded_cycles": padded, "segments": segs, "exec_ms": ms}));
    let GuestOutput::Plan(plan) = out(&journal)? else { bail!("{label}: plan returned an apply journal") };
    let (mut apply_user, mut apply_padded, mut effects) = (0u64, 0u64, 0u32);
    for effect in plan.effects {
        let a = ApplyInput { self_account_id: self_id, selector: effect.selector, pre_data: pre.get(&effect.selector).cloned().unwrap_or_default(), effect_data: effect.data };
        let (j, u, pd, _s, _ms) = exec(program, |env| Ok(Program::write_apply_inputs(&a, env)?))?;
        let GuestOutput::Apply(o) = out(&j)? else { bail!("{label}: apply returned a plan journal") };
        if o.post_data.is_none() { bail!("{label}: apply wrote nothing") }
        apply_user += u;
        apply_padded += pd;
        effects += 1;
    }
    rows.push(serde_json::json!({"instruction": label, "phase": "apply", "effects": effects, "user_cycles": apply_user, "padded_cycles": apply_padded}));
    Ok(rows)
}

fn main() -> Result<()> {
    let path = std::env::args().nth(1).ok_or_else(|| anyhow::anyhow!("usage: cycle_bench_osm <osm_registry.bin>"))?;
    let program = Program::new(std::fs::read(&path)?.into())?;
    let self_id = AccountId::new([0x42; 32]);
    let signer = AccountId::new([0x07; 32]);
    let regions: Vec<(&str, &str, u8, &str)> = vec![
        ("kenya", "", 0, "africa/kenya"), ("egypt", "", 0, "africa/egypt"), ("nigeria", "", 0, "africa/nigeria"),
        ("morocco", "", 0, "africa/morocco"), ("ethiopia", "", 0, "africa/ethiopia"), ("peru", "", 0, "south-america/peru"),
        ("chile", "", 0, "south-america/chile"), ("colombia", "", 0, "south-america/colombia"), ("argentina", "", 0, "south-america/argentina"),
        ("brazil", "", 0, "south-america/brazil"), ("japan", "", 0, "asia/japan"), ("vietnam", "", 0, "asia/vietnam"),
        ("thailand", "", 0, "asia/thailand"), ("philippines", "", 0, "asia/philippines"), ("pakistan", "", 0, "asia/pakistan"),
        ("bangladesh", "", 0, "asia/bangladesh"),
    ];
    let meta = |r: &str| AccountMeta::new(pda(&self_id, r), false, self_id);
    let mut rows = Vec::new();
    let empty = HashMap::new();
    let (r, p, l, path0) = regions[0];
    rows.extend(case(&program, self_id, "register", vec![meta(r), AccountMeta::new(signer, true, self_id)],
        &Instruction::Register { region: r.into(), parent: p.into(), level: l, cid: CID.into(),
            source_url: format!("https://download.geofabrik.de/{path0}-261101.osm.pbf"), checksum: "0123456789abcdef0123456789abcdef".into(),
            version: 261101, size_bytes: 1, timestamp: TS }, &empty)?);
    for n in [1usize, 5, 10, 16] {
        let sel = &regions[..n];
        let mut accounts = vec![AccountMeta::new(signer, true, self_id)];
        accounts.extend(sel.iter().map(|x| meta(x.0)));
        let ix = Instruction::RegisterBatch {
            regions: sel.iter().map(|x| x.0.to_string()).collect(), parents: sel.iter().map(|x| x.1.to_string()).collect(),
            levels: sel.iter().map(|x| x.2).collect(), cids: vec![CID.into(); n],
            source_urls: sel.iter().map(|x| format!("https://download.geofabrik.de/{}-261101.osm.pbf", x.3)).collect(),
            checksums: vec!["0123456789abcdef0123456789abcdef".into(); n], versions: vec![261101; n], sizes: vec![1; n], timestamp: TS };
        rows.extend(case(&program, self_id, Box::leak(format!("register_batch n={n}").into_boxed_str()), accounts, &ix, &empty)?);
    }
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({"program": path, "rows": rows}))?);
    Ok(())
}
