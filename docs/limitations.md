# Limitations

Written as found, not at the end. Each line says what, why, and what a user should expect.

- **Two country ids are interpretations.** The prize names "Ireland" and "Malaysia" without ids.
  We use Geofabrik's `ireland-and-northern-ireland` and `malaysia-singapore-brunei`, the only
  extracts that contain them without overlapping another region in the set.
- **The registry cannot check bytes.** Anyone can register a CID whose bytes do not match the
  checksum. The SDK only uses entries whose download hashes to the checksum, and the verify script
  reports every entry. A flooder can push good entries out of a region's 8-entry ring by paying
  for 8 transactions; re-registering restores them (ADR 0002).
- **Hosting exposes the host's node address** to Storage peers, as Logos Storage serves data
  directly (ADR 0005).
- **LEZ v0.3 only.** The program, SDK write path and probes run on LEZ v0.3 (localnet and the public
  v0.3 testnet). LEZ 0.4 changes the programming model to actors; the port, and SPEL support for
  it, are not done (docs/status/lez-04-port.md). v0.3 evidence never counts for the prize.
- **No Basecamp app or module yet.** They need nix and logos-module-builder.
- **No public Storage node yet.** CIDs registered from this machine cannot be fetched by others
  until the node runs on a public server, so nothing is registered on the public v0.3 testnet.
- **Timestamps on v0.3 are client-supplied** but bound to block time: the program declares the
  transaction valid only within ±10 minutes of the value, in milliseconds.
- **Fee reserve.** Each write needs 38,400,000 lepta available up front (the sequencer reserves
  it); measured fees are far lower (register 414,960 lepta on the local v0.3.1 sequencer).
- **The write path shells out to `osm-chain`.** The CLI stays free of LEZ wallet libraries and
  runs the signer as a separate binary with an argument array.
- **SPEL-generated client patched at build time.** Two defects in the generated client are
  worked around in `lez_adapter_v03/build.rs` (docs/BUGS_FILED.md #1, #2).
