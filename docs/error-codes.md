# Error codes

Typed codes shared by the program, SDK, FFI, CLI and UI. Each code has one plain message for the
UI (H34). The CLI prints the code and message and exits with the listed status. Codes are stable
once released in the alpha (SC-A.4); new codes are added, existing ones never change meaning.

## Program (on-chain, returned as the failure reason)

| Code | Name | UI message |
|------|------|------------|
| P001 | `RegionNotInSet` | This region is not in the LP-0018 predefined set. |
| P002 | `PositionMismatch` | Parent or level does not match the predefined set for this region. |
| P003 | `BadCid` | The content id is not a valid Logos Storage CID. |
| P004 | `BadSourceUrl` | The source must be a dated Geofabrik file for this region. |
| P005 | `VersionMismatch` | The version does not match the date in the source file name. |
| P006 | `BadChecksum` | The checksum must be a 32-character MD5 hex string. |
| P007 | `SizeOutOfRange` | The file size is zero or larger than the 64 GiB limit. |
| P008 | `TimestampOutOfRange` | The registration time is outside the accepted window. |
| P009 | `HostsFull` | This copy already has the maximum of 16 hosts. |
| P010 | `BatchTooLarge` | Too many regions in one batch. Split it into smaller batches. |
| P011 | `DuplicateInBatch` | The same region appears twice in one batch. |
| P012 | `AccountMismatch` | The region account does not belong to this region. |

## SDK / CLI (exit status in brackets)

| Code | Name | UI message |
|------|------|------------|
| S100 | `RpcUnreachable` [3] | Cannot reach the sequencer at {url}. Check the network setting. |
| S101 | `WrongNetwork` [3] | The sequencer at {url} is not the {profile} network. |
| S102 | `ProgramNotFound` [3] | The registry program {id} is not deployed on this network. Check the profile. |
| S103 | `TxNotIncluded` [4] | The transaction was sent but is not in a block after {secs} s. Nothing was registered. |
| S104 | `TxRejected` [4] | The network rejected the transaction: {reason}. |
| S105 | `InsufficientFunds` [4] | The paying account has {have}; about {need} is needed. |
| S106 | `WalletLocked` [4] | The wallet is locked. Enter the wallet password. |
| S110 | `StorageUnreachable` [5] | Cannot reach the Storage node at {url}. Check the Storage setting. |
| S111 | `StorageUploadExhausted` [5] | Upload failed after {n} attempts: {cause}. |
| S112 | `CidNotFound` [5] | No Storage node served {cid} within {secs} s. |
| S113 | `DownloadedHashMismatch` [5] | The downloaded file does not match the registered checksum. It was deleted. |
| S120 | `GeofabrikUnreachable` [6] | Cannot reach Geofabrik. Hosted regions are still available. |
| S121 | `GeofabrikDisabled` [6] | Geofabrik is turned off in settings, and this region is not hosted. |
| S122 | `SnapshotGone` [6] | Geofabrik no longer serves the snapshot {file}. |
| S130 | `ImportChecksumMismatch` [7] | This file's MD5 {got} does not match Geofabrik's {want} for {snapshot}. Nothing was stored. |
| S131 | `ImportUnknownSnapshot` [7] | Cannot tell which Geofabrik snapshot this file is. Give the snapshot date. |
| S132 | `NotAPbf` [7] | This file is not an OSM PBF file. |
| S140 | `RegionNotHosted` [8] | {region} is not hosted on Logos Storage yet. |
| S141 | `NoVerifiedEntry` [8] | {region} has entries, but none downloaded and matched its checksum. |
| S150 | `DiskFull` [9] | Not enough free disk: {need} needed, {have} free. |
| S151 | `CacheLimit` [9] | The map cache limit ({limit}) is too small for this file. |
| S152 | `DataDirUnsafe` [9] | The data folder {path} has unsafe permissions. |
| S160 | `InvalidInput` [2] | {field}: {why}. Nothing was changed. |

Exit status 0 = success, 1 = internal bug, 2 = invalid input. Input is rejected, never trimmed or
clamped (E64).
