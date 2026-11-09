//! Private, durable local data (H38): folder 0700, files 0600, atomic writes.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::SdkError;

/// Creates `path` (and parents) and makes it owner-only on Unix. An existing folder that is
/// readable by group or others is tightened, not trusted.
pub fn ensure_private_dir(path: &Path) -> Result<PathBuf, SdkError> {
    fs::create_dir_all(path).map_err(|e| SdkError::io(path, e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|e| SdkError::io(path, e))?;
        let mode = fs::metadata(path)
            .map_err(|e| SdkError::io(path, e))?
            .permissions()
            .mode();
        if mode & 0o077 != 0 {
            return Err(SdkError::DataDirUnsafe {
                path: path.to_path_buf(),
            });
        }
    }
    Ok(path.to_path_buf())
}

/// Writes `bytes` to `dest` atomically: temp file in the same folder, `validate` on the written
/// bytes, fsync, rename, folder fsync. A crash leaves either the old file or the new one (E75).
pub fn atomic_write(
    dest: &Path,
    bytes: &[u8],
    validate: impl Fn(&[u8]) -> bool,
) -> Result<(), SdkError> {
    if !validate(bytes) {
        return Err(SdkError::InvalidInput {
            field: "file contents",
            why: format!("refusing to replace {} with invalid data", dest.display()),
        });
    }
    let dir = dest.parent().ok_or_else(|| SdkError::InvalidInput {
        field: "path",
        why: format!("{} has no parent folder", dest.display()),
    })?;
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.tmp-{}", std::process::id()));
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let write = || -> std::io::Result<()> {
        let mut f = opts.open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, dest)?;
        #[cfg(unix)]
        fs::File::open(dir)?.sync_all()?;
        Ok(())
    };
    write().map_err(|e| {
        let _ = fs::remove_file(&tmp);
        SdkError::io(dest, e)
    })
}

/// Keeps `dir` under `cap_bytes` by deleting the oldest `*.osm.pbf` files first (by modification
/// time), never `keep`. Returns the deleted paths. `.part` files are left alone (resumable).
pub fn enforce_cache_cap(
    dir: &Path,
    cap_bytes: u64,
    keep: &Path,
) -> Result<Vec<PathBuf>, SdkError> {
    let mut files = Vec::new();
    for e in fs::read_dir(dir).map_err(|e| SdkError::io(dir, e))? {
        let e = e.map_err(|e| SdkError::io(dir, e))?;
        let p = e.path();
        if p.extension().is_some_and(|x| x == "pbf") && p.to_string_lossy().ends_with(".osm.pbf") {
            let m = e.metadata().map_err(|err| SdkError::io(&p, err))?;
            files.push((m.modified().ok(), m.len(), p));
        }
    }
    let mut total: u64 = files.iter().map(|f| f.1).sum();
    files.sort_by_key(|f| f.0);
    let mut deleted = Vec::new();
    for (_, len, p) in files {
        if total <= cap_bytes {
            break;
        }
        if p == keep {
            continue;
        }
        fs::remove_file(&p).map_err(|e| SdkError::io(&p, e))?;
        total = total.saturating_sub(len);
        deleted.push(p);
    }
    Ok(deleted)
}
