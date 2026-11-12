# Cycle counts (P-P1, H8)

Executor cycles of the registry program for each on-chain operation, measured with
`programs/osm_registry/cycle_bench_osm`. It uses the method of LEZ's `tools/cycle_bench`: the
program ELF runs in the RISC Zero executor with the same plan and apply inputs the sequencer
gives it (`Program::write_plan_inputs` / `write_apply_inputs`, LEZ v0.3 libraries).

- Program: `programs/osm_registry`, ImageID `f329edb220bdaade9d69250bb36969b95b672b4ea9d44c7a0092557762305e85` (the binary deployed on the public v0.3 testnet).
- Measured: 2026-11-09, repository HEAD `e88f27b`, macOS arm64, risc0-zkvm 3.0.5 executor, `RISC0_DEV_MODE=0`.
- Command: `cargo run --release -p cycle_bench_osm -- methods/guest/target/riscv32im-risc0-zkvm-elf/docker/osm_registry.bin` (in `programs/osm_registry`).
- User cycles exclude continuation overhead and power-of-two padding; padded cycles are the sum of 2^po2 over segments (what proving would size to).

| Operation | Regions | Plan user cycles | Apply user cycles (all effects) | Total user cycles | Total padded cycles |
|-----------|---------|------------------|---------------------------------|-------------------|---------------------|
| `register` | 1 | 35,563 | 14,407 | 49,970 | 196,608 |
| `register_batch` | 1 | 38,300 | 14,407 | 52,707 | 196,608 |
| `register_batch` | 5 | 146,927 | 72,403 | 219,330 | 589,824 |
| `register_batch` | 10 | 281,020 | 144,009 | 425,029 | 1,179,648 |
| `register_batch` | 16 | 442,383 | 230,823 | 673,206 | 2,097,152 |

Batching costs about the same per region as single registration (plan grows by about 27k user
cycles per region, apply by about 14.4k), so its saving is in transaction count and fees, not cycles.
16 is the program's `MAX_BATCH`.

## Fees seen on a real sequencer

On the local LEZ v0.3.1 standalone sequencer (2026-11-09): a `register` cost 414,960 lepta; a program
deploy (header + 5 segments) cost 3,629,536 lepta. The public v0.3 testnet charged the same 3,629,536
lepta for the deploy. The sequencer reserves `(gas_limit + 100,000) x 64` = 38,400,000 lepta up front
per transaction at the SPEL client's default gas limit.

## Not measured yet

- Apply into an account that already holds 8 entries (ring rotation); the cases above write into
  empty accounts.
- Proving time (`--prove` path). The public testnet executes public transactions without proofs.
- `register_batch` on LEZ 0.4 with the current guest (CI job `e2e-v04` prints it).

## LEZ 0.4 actor registry

Measured with `programs/osm_registry_v04/client/src/bin/cycle-bench-v04.rs`, the executor path
of LEZ v0.4.0-rc1's `tools/cycle_bench`: the whole delivery runs through `WholeExecution`, and
each transition (our host actor, the clock program, our region actor) runs in the RISC Zero
executor with `Program::write_receive_input`. One row per transition; a caller pays for all.

- Program: the 2026-11-09 guest build, ImageID `d1213b2811def8ad89db14db2d34c6523ffc6b0b2723f0eca95897e720881c1a`
  (built before `RegisterBatch` existed, so it has no batch numbers).
- Measured: 2026-11-12, macOS arm64, risc0-zkvm 3.0.5 executor. Raw output:
  `probes/data/cycles-v04-d1213b28.json`.

| Operation | Host actor | Clock | Region actor | Total user cycles | Total padded cycles |
|-----------|------------|-------|--------------|-------------------|---------------------|
| `Register` | 25,033 | 8,532 | 17,271 | 50,836 | 262,144 |
| `Unhost` | 12,531 | — | 16,084 | 28,615 | 131,072 |

So one registration costs about the same on 0.4 as on v0.3 (50,836 vs 49,970 user cycles); on
0.4 part of it moves to the clock program, which now checks the timestamp.
