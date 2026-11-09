//! SDK errors, numbered as in docs/error-codes.md.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("S100 RpcUnreachable: cannot reach the sequencer at {url}: {cause}")]
    RpcUnreachable { url: String, cause: String },
    #[error("S170 RegistryDataInvalid: {account}: {why}")]
    RegistryDataInvalid { account: String, why: String },
    #[error("S110 StorageUnreachable: cannot reach the Storage node at {url}")]
    StorageUnreachable { url: String },
    #[error("S111 StorageUploadExhausted: upload failed after {attempts} attempts: {last_cause}")]
    StorageUploadExhausted { attempts: u32, last_cause: String },
    #[error("S112 CidNotFound: no Storage node served {cid}")]
    CidNotFound { cid: String },
    #[error("S113 DownloadedHashMismatch: {path} does not match the registered checksum")]
    DownloadedHashMismatch { path: PathBuf },
    #[error("S120 GeofabrikUnreachable: {cause}")]
    GeofabrikUnreachable { cause: String },
    #[error("S122 SnapshotGone: Geofabrik no longer serves {file}")]
    SnapshotGone { file: String },
    #[error(
        "S130 ImportChecksumMismatch: MD5 {got} does not match Geofabrik's {want} for {snapshot}"
    )]
    ImportChecksumMismatch {
        got: String,
        want: String,
        snapshot: String,
    },
    #[error("S132 NotAPbf: {path} is not an OSM PBF file")]
    NotAPbf { path: PathBuf },
    #[error("S152 DataDirUnsafe: {path} has unsafe permissions")]
    DataDirUnsafe { path: PathBuf },
    #[error("S160 InvalidInput: {field}: {why}")]
    InvalidInput { field: &'static str, why: String },
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl SdkError {
    /// Stable code, for example `S130`.
    pub fn code(&self) -> &'static str {
        match self {
            SdkError::RpcUnreachable { .. } => "S100",
            SdkError::RegistryDataInvalid { .. } => "S170",
            SdkError::StorageUnreachable { .. } => "S110",
            SdkError::StorageUploadExhausted { .. } => "S111",
            SdkError::CidNotFound { .. } => "S112",
            SdkError::DownloadedHashMismatch { .. } => "S113",
            SdkError::GeofabrikUnreachable { .. } => "S120",
            SdkError::SnapshotGone { .. } => "S122",
            SdkError::ImportChecksumMismatch { .. } => "S130",
            SdkError::NotAPbf { .. } => "S132",
            SdkError::DataDirUnsafe { .. } => "S152",
            SdkError::InvalidInput { .. } => "S160",
            SdkError::Io { .. } => "I/O",
        }
    }

    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        SdkError::Io {
            path: path.into(),
            source,
        }
    }
}
