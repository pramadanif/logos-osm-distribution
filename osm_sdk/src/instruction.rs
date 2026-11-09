//! Borsh encoding of the registry program's instructions, identical to what the SPEL-generated
//! client sends (variant index in IDL order, then the arguments). Used by writers that sign
//! elsewhere, e.g. a Basecamp module handing the bytes to `lez_core`.

use borsh::BorshSerialize;

/// Variant order = `idl/osm_registry.json` instruction order.
#[derive(BorshSerialize, Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Register {
        region: String,
        parent: String,
        level: u8,
        cid: String,
        source_url: String,
        checksum: String,
        version: u32,
        size_bytes: u64,
        timestamp: u64,
    },
    RegisterBatch {
        regions: Vec<String>,
        parents: Vec<String>,
        levels: Vec<u8>,
        cids: Vec<String>,
        source_urls: Vec<String>,
        checksums: Vec<String>,
        versions: Vec<u64>,
        sizes: Vec<u64>,
        timestamp: u64,
    },
    Unhost {
        region: String,
        cid: String,
    },
}

impl Instruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("borsh serialization of owned data cannot fail")
    }
}

/// Lowercase hex of a base58 account id, the form `lez_core` takes.
pub fn account_hex(b58: &str) -> Result<String, crate::SdkError> {
    let bytes = crate::registry::decode_account_id(b58.trim_start_matches("Public/"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
