/// Generate IDL JSON for the osm_registry program.
///
/// Usage:
///   cargo run --bin generate_idl > osm_registry-idl.json

spel_framework::generate_idl!("../methods/guest/src/bin/osm_registry.rs");
