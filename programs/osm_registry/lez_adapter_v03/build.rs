//! Copies the SPEL-generated client into OUT_DIR so it can be included as a module, and adds the
//! `BorshSerialize` derive that `spel-client-gen --target rust+ffi` (vpavlin/spel @ e43b59be)
//! leaves off `OsmRegistryInstruction` in the client file while the FFI file has it.
//! It also drops `fetch_region_account`, which reads `account.data` as bytes (the pre-v0.3 model;
//! since v0.3 data is per-program shards). Reads go through `osm_sdk::registry` instead.
//! Both are logged in docs/BUGS_FILED.md. The build fails loudly if the generated text changes shape.
use std::{env, fs, path::PathBuf};

fn main() {
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../osm_registry_ffi/generated/osm_registry_client.rs");
    println!("cargo:rerun-if-changed={}", src.display());
    let text = fs::read_to_string(&src).expect("generated client present; run `make ffi-gen`");
    let from = "#[derive(Clone, Debug, Serialize, Deserialize)]\npub enum OsmRegistryInstruction";
    let to = "#[derive(Clone, Debug, Serialize, Deserialize, borsh::BorshSerialize)]\npub enum OsmRegistryInstruction";
    assert!(
        text.contains(from),
        "generated client changed shape; revisit lez_adapter_v03/build.rs"
    );
    let fetch = "    /// Fetch and deserialize the `region_account` account state.";
    let start = text
        .find(fetch)
        .expect("generated client changed shape: fetch_region_account");
    let end = text[start..]
        .find("\n    }\n")
        .expect("end of fetch_region_account")
        + start
        + "\n    }\n".len();
    let text = format!("{}{}", &text[..start], &text[end..]);
    let text: String = text
        .replace(from, to)
        .lines()
        .filter(|l| !l.starts_with("//!"))
        .map(|l| format!("{l}\n"))
        .collect();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("osm_registry_client.rs");
    fs::write(out, text).unwrap();
}
