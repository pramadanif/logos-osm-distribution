# Building on the OSM registry and hosting maps

For two groups: developers who want map data in their own Logos module, and people who want to
help by hosting (mirroring) regions. Status: alpha on LEZ v0.3. The Basecamp modules and two templates are written but not yet
proven in Basecamp; the paths below that are proven use the SDK, the C ABI and the CLI.

## Use map data in your module

Your module needs three things: the registry program id, a sequencer RPC URL, and a Logos
Storage node. `networks.json` lists the current deployments.

**Rust:** depend on `osm_sdk` (pinned git commit, see [sdk.md](sdk.md)) and follow the worked
example there: `Registry::resolve(region)` gives the CID and Geofabrik MD5 of the newest hosted
snapshot; `fetch::fetch_region` downloads it by CID and checks the MD5.

**Basecamp module:** start from `examples/consumer-module` (a C++ core module calling the
read-only `pramadani_osm_sdk` module) or `examples/distribution-consumer-app` (a QML app calling
`pramadani_osm_distribution`). Copy the folder, rename the module, build your idea.

**C / C++ / Qt:** link `osm_ffi` and include `osm_ffi/include/osm_ffi.h`. Every call takes and
returns JSON, e.g.

```c
char *r = osm_resolve("{\"rpc\":\"https://testnet.lez.logos.co\","
                      "\"program\":\"5VZeCZKSzkQyYb2nPm4ShDAKpkwncvbQh58XSg4cCAcS\","
                      "\"region\":\"kenya\"}");
/* {"ok":true,"data":{"cid":"…","checksum":"…","version":261101,…}} or {"ok":false,"code":"S1xx",…} */
osm_free(r);
```

Call it off the UI thread; network calls have timeouts but can take seconds.

**What you can rely on**

- The consumer API (`discover`, `resolve`, `fetch`, `hosted_status`, `check_updates`) is frozen
  for the alpha and versioned (`osm_version()` returns `"api": 1`). A breaking change gets a new
  API number; the 0.4 port changes the backend, not this API.
- A file you get from `fetch` has the MD5 that was registered for it; anything else is an error.
- Your module never needs a wallet or Geofabrik to read hosted data.

**Rules for the LP-0018 adoption count** (prize text): your own idea, your own forge account with
real history, commits spread over time, at least one public repository. Nobody from this project
writes or pays for your code.

## Host a region (mirror)

Hosting means: keep the file in your Logos Storage node, and register yourself on-chain as a
host of its CID. Logos Storage CIDs are content-addressed, so the same file uploaded to your
node gets the same CID, and registering that CID adds you to its host list instead of creating a
new entry (ADR 0002).

1. Run a Logos Storage node that others can reach (public IP or port forwarding). Note: hosting
   shares your node's network address with peers ([security.md](security.md)).
2. Get a funded LEZ account. On the v0.3 testnet: the `lez-faucet` Basecamp app gives 0.1 LEZ
   per day; each write needs 38,400,000 lepta available for the fee reserve.
3. Host. Either download the snapshot from Geofabrik and host it:

       osm host --region kenya --cache-dir ./cache --storage http://127.0.0.1:8080/api/storage/v1 \
                --rpc https://testnet.lez.logos.co --program 5VZeCZKSzkQyYb2nPm4ShDAKpkwncvbQh58XSg4cCAcS \
                --host <your account> --chain-bin osm-chain

   or mirror what is already hosted, without touching Geofabrik:

       osm download --region kenya --dir ./maps --storage <any reachable node> --rpc … --program …
       osm import ./maps/kenya-261101.osm.pbf --region kenya --snapshot 261101 --storage <your node> …

   `import` checks the file against Geofabrik's published MD5 once, uploads it to your node (same
   CID) and registers you as a host.
4. Keep the node running. Stop hosting with `osm unhost --region kenya --cid <cid> …`.

## FAQ

**Can someone overwrite my entry?** No. Entries are append-only; another signer with the same CID
is added as a host, a different CID becomes a separate entry. Someone who pays for 8 transactions
can rotate a region's history; re-registering restores yours (limitations.md).

**What if a registered file is wrong?** Downloads check the MD5 and skip entries that do not
verify; the coverage verifier reports them.

**Does anything depend on a server run by this project?** No. The sequencer, Logos Storage and
Geofabrik (only as the snapshot source) are the only network services ([ADR 0006](adr/0006-external-services.md)).

**Which regions?** Exactly the 72 of the prize's closed set: `osm regions`.
