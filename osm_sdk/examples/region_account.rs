//! Prints the region account address for a program and region (same derivation as the SPEL
//! seeds `["osm-region", arg region]`).
//!
//!     cargo run -p osm_sdk --example region_account -- <program-id> <region>
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let [_, program, region] = &a[..] else {
        eprintln!("usage: region_account <program-id> <region>");
        std::process::exit(2);
    };
    match osm_sdk::registry::region_account_id(program, region) {
        Ok(id) => println!("{id}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
