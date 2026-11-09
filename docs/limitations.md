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
- **Nothing is deployed yet.** This file grows with the build.
