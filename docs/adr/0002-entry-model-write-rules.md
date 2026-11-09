# ADR 0002: Entry model and write rules

Status: Accepted.

## Entry (per region account)

    RegionAccount {
        version: u8,                 // layout version
        region: String,              // prize leaf id, checked against osm_core::PREDEFINED
        parent: Option<String>,      // must equal the table
        level: Level,                // must equal the table
        history: Vec<Entry>,         // ring, newest last, len <= 8
    }
    Entry {
        cid: String,                 // Logos Storage manifest CID, charset + length checked
        source_url: String,          // dated Geofabrik URL, <path>-YYMMDD.osm.pbf, prefix checked
        checksum: [u8; 16],          // MD5 of the snapshot as published by Geofabrik
        version: u32,                // YYMMDD of the snapshot, parsed from source_url, must match
        size_bytes: u64,             // capped at 64 GiB
        hosted: bool,                // true while hosts is non-empty
        timestamp: u64,              // unix seconds, see "Timestamp" below (§2 guard 4)
        registered_by: AccountId,    // = transaction signer, set by the program
        hosts: Vec<AccountId>,       // deduplicated, len <= 16, first = registered_by
    }

The prize fields `region, parent, level, cid, source_url, checksum, version, hosted, timestamp`
are all present. `size_bytes` and `hosts` / `registered_by` are additions.

## Write rules

1. `register(region, entry)` with a CID already in the ring for that region: add the signer to
   that entry's `hosts` (mirror add). Idempotent: a signer already listed is a no-op success.
2. With a new CID: append a new entry; when the ring is full, drop the oldest entry. Never
   overwrite an entry in place.
3. `registered_by` and every added host are the transaction signer. Clients cannot supply them.
4. `version` must equal the date in `source_url`. Same source URL with a different checksum or
   CID is allowed (anyone can lie about bytes on-chain; nobody can stop it), but it is a new
   entry with the liar as `registered_by`.
5. Removal: a host may remove itself from `hosts` (`unhost`). Nothing else is ever deleted
   except by ring rotation.

## Timestamp (§2 guard 4, H37)

The client supplies `timestamp`. The program rejects values below 2026-11-01T00:00:00Z
(`1_761_955_200`) and declares the transaction valid only inside a window of plus or minus 600 s
around that value, using the LEZ validity window (`timestamp_window`, v0.4.0-rc1
`lee/state_machine/core/src/program/mod.rs:489`). The sequencer then refuses the transaction if
block time is outside the window, so a stored timestamp is within 10 minutes of real block time.
If v0.3 has no such window, the v0.3 backend stores the client value bounded only by the floor
and documents it as client-supplied. Both are verified in SC-B.5; until then this is a design,
not a fact. Ordering for P-R3: by `timestamp`, then ring position.

## Griefing and the SDK rule

The program cannot check that a CID's bytes match the MD5. So the SDK picks, per region, the
newest entry whose CID downloads and hashes to `checksum`, and `verify-coverage.sh` reports every
entry's status. Ring rotation means a flooder with 8 junk entries in one block can push ours out
of the ring; the cost is 8 paid transactions per region, and our re-register restores it. This
trade-off goes in docs/limitations.md (W4). A per-instance curator key was rejected: it creates
the capturable authority ADR 0003 avoids.
