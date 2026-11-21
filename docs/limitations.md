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
- **LEZ 0.4 only on a local rc sequencer.** The actor program passes the probes on a local
  v0.4.0-rc1 sequencer; testnet 0.4 is not live. SPEL has no 0.4 support, so the 0.4 program is
  hand-written against LEZ's actor API and there is no 0.4 IDL (docs/status/lez-04-port.md). v0.3
  evidence never counts for the prize.
- **Basecamp writes are v0.3 only.** The distribution module signs through `lez_core` 0.5.1, which
  is built on LEZ v0.3.1. On 0.4 the module reads (discover, resolve, fetch, status) but cannot
  write until a 0.4 wallet module exists; the CLI writes on 0.4 with `osm-chain-v04`.
- **Basecamp GUI not run yet.** The modules, app and templates build and package on Linux and
  macOS, and the core modules load and answer in `logosctl`; the app has not been opened in
  Basecamp, so its screens, two-profile isolation and Qt number formatting are untested.
- **The C++ parts are not run under ASan/UBSan.** Only the C ABI is (CI `ffi-sanitizers`); the app
  backend and the C++ template need a Qt build under the sanitizers.
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
- **Android is not built or tested.** The prize lists Android from testnet 0.4 as recommended,
  not required; CI covers Linux, macOS and Windows for the Rust crates only (W9).
- **LEZ 0.4 wallet passwords.** The 0.4 wallet encrypts its storage; scripts pass the password in
  `LEE_WALLET_PASSWORD` (read from a 0600 file), because the wallet otherwise prompts on the
  terminal (docs/BUGS_FILED.md #9).
