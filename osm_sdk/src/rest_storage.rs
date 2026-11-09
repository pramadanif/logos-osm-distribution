//! Logos Storage node REST client (`logos-storage-nim` v0.5.3 `openapi.yaml`, base
//! `http://<host>:8080/api/storage/v1`). Streams files; never holds a whole file in memory.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::time::Duration;

use crate::storage::{StorageBackend, StorageFault};

/// Longest CID response body accepted from the node, in bytes.
const MAX_CID_BODY: u64 = 256;

pub struct RestStorage {
    base: String,
    agent: ureq::Agent,
}

impl RestStorage {
    /// `base` is the API root, for example `http://127.0.0.1:8080/api/storage/v1`.
    /// `read_timeout` bounds a stall between bytes, not the whole transfer, so multi-GB files work.
    pub fn new(base: &str, read_timeout: Duration) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(read_timeout)
            .build();
        RestStorage {
            base: base.trim_end_matches('/').to_string(),
            agent,
        }
    }

    fn url(&self, cid: &str, tail: &str) -> Result<String, StorageFault> {
        osm_core::entry::validate_cid(cid)
            .map_err(|_| StorageFault::Permanent(format!("invalid CID {cid:?}")))?;
        Ok(format!("{}/data/{cid}{tail}", self.base))
    }
}

fn classify(e: ureq::Error) -> StorageFault {
    match e {
        ureq::Error::Status(code, _) if code >= 500 => {
            StorageFault::Transient(format!("HTTP {code}"))
        }
        ureq::Error::Status(code, _) => StorageFault::Permanent(format!("HTTP {code}")),
        ureq::Error::Transport(t) => StorageFault::Transient(t.to_string()),
    }
}

fn io_fault(what: &str, e: io::Error) -> StorageFault {
    StorageFault::Transient(format!("{what}: {e}"))
}

impl StorageBackend for RestStorage {
    fn upload_file(&mut self, path: &Path) -> Result<String, StorageFault> {
        let open_err = |e: io::Error| StorageFault::Permanent(format!("{}: {e}", path.display()));
        let f = File::open(path).map_err(open_err)?;
        // Explicit length: the node's support for chunked uploads is not documented.
        let len = f.metadata().map_err(open_err)?.len();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.is_empty() || name.contains(['"', '\r', '\n']) {
            return Err(StorageFault::Permanent(format!(
                "unsafe file name {name:?}"
            )));
        }
        let resp = self
            .agent
            .post(&format!("{}/data", self.base))
            .set("content-type", "application/octet-stream")
            .set("content-length", &len.to_string())
            .set(
                "content-disposition",
                &format!("attachment; filename=\"{name}\""),
            )
            .send(f)
            .map_err(classify)?;
        let mut body = String::new();
        resp.into_reader()
            .take(MAX_CID_BODY + 1)
            .read_to_string(&mut body)
            .map_err(|e| io_fault("reading CID", e))?;
        if body.len() as u64 > MAX_CID_BODY {
            return Err(StorageFault::Permanent("CID response too long".into()));
        }
        Ok(body.trim().to_string())
    }

    /// Fetches through the network and writes `dest` atomically (`<dest>.part`, then rename).
    fn download_to_file(&mut self, cid: &str, dest: &Path) -> Result<(), StorageFault> {
        let url = self.url(cid, "/network/stream")?;
        let resp = self.agent.get(&url).call().map_err(|e| match e {
            ureq::Error::Status(404, _) => StorageFault::Permanent(format!("{cid} not found")),
            other => classify(other),
        })?;
        let part = dest.with_extension("part");
        let res = (|| -> io::Result<()> {
            let mut out = File::create(&part)?;
            io::copy(&mut resp.into_reader(), &mut out)?;
            out.sync_all()?;
            fs::rename(&part, dest)
        })();
        res.map_err(|e| {
            let _ = fs::remove_file(&part);
            io_fault("download", e)
        })
    }

    /// v0.5.3 answers `{"<cid>": <bool>}` (observed 2026-11-09), although `openapi.yaml` documents
    /// `{"has": <bool>}`. Both are accepted; any other shape is an error.
    fn exists(&mut self, cid: &str) -> Result<bool, StorageFault> {
        let url = self.url(cid, "/exists")?;
        let resp = self.agent.get(&url).call().map_err(classify)?;
        let mut body = Vec::new();
        resp.into_reader()
            .take(4096)
            .read_to_end(&mut body)
            .map_err(|e| io_fault("reading exists", e))?;
        let v: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| StorageFault::Permanent(format!("bad exists response: {e}")))?;
        v.get(cid)
            .or_else(|| v.get("has"))
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| StorageFault::Permanent(format!("bad exists response: {v}")))
    }
}
