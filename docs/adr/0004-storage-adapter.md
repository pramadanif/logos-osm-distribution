# ADR 0004: Storage adapter

Status: Accepted.

## Context (read from source, 2026-11-09)

- Storage node REST API, `logos-storage-nim` v0.5.3 `openapi.yaml`, base
  `http://localhost:8080/api/storage/v1`: `POST /data` (streaming upload, returns CID,
  `advertise` query default true), `GET /data/{cid}` (local only, 404 if absent),
  `GET /data/{cid}/network/stream` (fetch from the network, streamed),
  `GET /data/{cid}/exists`, `GET|POST /data/{cid}/advertise`, `GET /space`, `GET /debug/info`.
- Basecamp `storage_module` v3.0.2 (`src/storage_module_plugin.h`): `uploadUrl(filePath,
  chunkSize, advertise)`, `uploadInit/uploadChunk/uploadFinalize/uploadCancel`,
  `downloadToUrl(cid, filePath, local, chunkSize, isPrivate, advertise)`, `downloadChunks`,
  `downloadCancel`, `exists(cid)`, all returning `StdLogosResult`.

## Decision

- One Rust trait in `osm_sdk`: `StorageBackend { upload_file, download_to_file, exists,
  advertise }`, all streaming by path, never by byte buffer (H27).
- Two backends: `RestStorage` (CLI, CI, VPS node) and `ModuleStorage` (inside Basecamp, calls
  `storage_module` through the Logos core module bridge). Only `osm_sdk` knows either.
- Retry policy wraps every upload: exponential back-off with full jitter, base 500 ms, factor 2,
  cap 30 s, max attempts configurable (default 6). Retries only on transient classes (connect
  error, timeout, 5xx); 4xx fails at once. After the last attempt the error is
  `StorageUploadExhausted { attempts, last_cause }` (P-R1, H13).
- Download integrity: the CID is the integrity check (content addressing, prize text). The SDK
  also hashes the downloaded file with MD5 and compares to the registry `checksum`, because that
  is what the evaluator does and it catches wrong-CID entries (ADR 0002).
