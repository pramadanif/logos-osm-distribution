//! LP-0018 OSM registry SDK.
//!
//! Network access sits behind traits so every rule here is testable offline. LEZ access will
//! live in `lez-adapter`; this crate never talks to the chain directly.

pub mod datadir;
pub mod discovery;
pub mod error;
pub mod hash;
pub mod rest_storage;
pub mod retry;
pub mod snapshot;
pub mod storage;

pub use error::SdkError;
