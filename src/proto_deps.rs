//! Export BSR deps from a Buf module for protoc `-I` (never protoc inputs).

use anyhow::{Context, Result, bail};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const VALIDATE_PROTO: &str = "buf/validate/validate.proto";
const EXPORT_LOCK: &str = ".proto-deps.export.lock";
const EXPORT_STAMP: &str = ".export.stamp";
/// Protovalidate export is ~200 KiB; reject truncated cache artifacts well below that.
const MIN_VALIDATE_BYTES: u64 = 100_000;
const LOCK_TIMEOUT: Duration = Duration::from_secs(120);
const LOCK_RETRY: Duration = Duration::from_millis(50);

/// Path to the exported Protovalidate schema under `export_dir`.
pub fn validate_proto_path(export_dir: &Path) -> PathBuf {
    export_dir.join(VALIDATE_PROTO)
}

fn export_lock_path(export_dir: &Path) -> PathBuf {
    export_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(EXPORT_LOCK)
}

fn export_stamp_path(export_dir: &Path) -> PathBuf {
    export_dir.join(EXPORT_STAMP)
}

fn staging_export_dir(export_dir: &Path) -> PathBuf {
    let parent = export_dir.parent().unwrap_or_else(|| Path::new("."));
    let name = export_dir
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "proto-deps".to_owned());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    parent.join(format!("{name}.staging-{nonce}"))
}

fn trash_export_dir(export_dir: &Path) -> PathBuf {
    export_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}.export-trash",
            export_dir
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| "proto-deps".to_owned())
        ))
}

struct ExportLock {
    lock_path: PathBuf,
    // Held for the lifetime of the guard; exclusive create prevents concurrent exports.
    _file: File,
}

impl Drop for ExportLock {
    fn drop(&mut self) {
        let _ = self._file.sync_all();
        let _ = fs::remove_file(&self.lock_path);
    }
}

/// Serialize `buf export` for a shared `export_dir` (parallel tests, CI cache restore).
fn acquire_export_lock(export_dir: &Path) -> Result<ExportLock> {
    if let Some(parent) = export_dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let lock_path = export_lock_path(export_dir);
    let deadline = Instant::now() + LOCK_TIMEOUT;
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(file) => {
                return Ok(ExportLock {
                    lock_path,
                    _file: file,
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                if Instant::now() >= deadline {
                    bail!(
                        "timed out waiting for proto-deps export lock at {}",
                        lock_path.display()
                    );
                }
                thread::sleep(LOCK_RETRY);
            }
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("open export lock {}", lock_path.display()));
            }
        }
    }
}

/// Byte length recorded after a successful export; paired with `validate.proto` on reuse.
fn read_export_stamp(export_dir: &Path) -> Option<u64> {
    fs::read_to_string(export_stamp_path(export_dir))
        .ok()
        .and_then(|stamp| stamp.trim().parse().ok())
}

fn write_export_stamp(export_dir: &Path, size: u64) -> Result<()> {
    fs::write(export_stamp_path(export_dir), format!("{size}\n"))
        .with_context(|| format!("write {}", export_stamp_path(export_dir).display()))
}

/// True when the file ends like a complete proto (guards against mid-write truncation).
fn validate_proto_tail(path: &Path) -> Result<bool> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let len = file.metadata()?.len();
    if len == 0 {
        return Ok(false);
    }
    let tail_len = len.min(64);
    file.seek(SeekFrom::End(-(tail_len as i64)))?;
    let mut tail = vec![0; tail_len as usize];
    file.read_exact(&mut tail)?;
    Ok(tail.ends_with(b"}\n") || tail.ends_with(b"}\r\n"))
}

/// Reuse only when size, stamp, and tail match — not merely when the path exists.
fn export_is_current(export_dir: &Path) -> Result<bool> {
    let validate = validate_proto_path(export_dir);
    let metadata = match fs::metadata(&validate) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(false),
    };
    let size = metadata.len();
    if size < MIN_VALIDATE_BYTES {
        return Ok(false);
    }
    if read_export_stamp(export_dir) != Some(size) {
        return Ok(false);
    }
    validate_proto_tail(&validate)
}

fn validate_export_dir(export_dir: &Path, proto_root: &Path) -> Result<u64> {
    let validate = validate_proto_path(export_dir);
    if !validate.is_file() {
        bail!(
            "buf export missing {}; run `buf dep update` in {}",
            validate.display(),
            proto_root.display()
        );
    }
    let size = fs::metadata(&validate)
        .with_context(|| format!("stat {}", validate.display()))?
        .len();
    if size < MIN_VALIDATE_BYTES {
        bail!(
            "buf export produced unexpectedly small {}; expected at least {MIN_VALIDATE_BYTES} bytes",
            validate.display()
        );
    }
    if !validate_proto_tail(&validate)? {
        bail!(
            "buf export produced incomplete {}; file does not end with a closing brace",
            validate.display()
        );
    }
    Ok(size)
}

fn run_buf_export(proto_root: &Path, output_dir: &Path) -> Result<()> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir).with_context(|| {
            format!(
                "clear proto-deps staging export at {}",
                output_dir.display()
            )
        })?;
    }
    if let Some(parent) = output_dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let status = Command::new("buf")
        .current_dir(proto_root)
        .args(["export", ".", "--output"])
        .arg(output_dir)
        .status()
        .context("buf export")?;
    if !status.success() {
        bail!("buf export failed");
    }
    Ok(())
}

/// Build in a staging dir, then rename into place so concurrent protoc readers keep their tree.
fn publish_export(staging: &Path, export_dir: &Path) -> Result<()> {
    let trash = trash_export_dir(export_dir);
    let _ = fs::remove_dir_all(&trash);
    if export_dir.exists() {
        fs::rename(export_dir, &trash).with_context(|| {
            format!(
                "rotate proto-deps export {} -> {}",
                export_dir.display(),
                trash.display()
            )
        })?;
    }
    fs::rename(staging, export_dir).with_context(|| {
        format!(
            "publish proto-deps export {} -> {}",
            staging.display(),
            export_dir.display()
        )
    })?;
    let _ = fs::remove_dir_all(&trash);
    Ok(())
}

/// Export `proto_root` and its `buf.yaml` deps into `export_dir` for protoc import paths.
///
/// When `refresh` is false, reuses an existing export when `buf/validate/validate.proto`
/// passes integrity checks (size, stamp, trailing syntax).
/// When `refresh` is true, replaces `export_dir` via staging (xtask default).
pub fn ensure_proto_deps_export(
    proto_root: &Path,
    export_dir: &Path,
    refresh: bool,
) -> Result<PathBuf> {
    let _lock = acquire_export_lock(export_dir)?;

    if !refresh && export_is_current(export_dir)? {
        return Ok(export_dir.to_path_buf());
    }

    let staging = staging_export_dir(export_dir);
    run_buf_export(proto_root, &staging)?;
    let size = validate_export_dir(&staging, proto_root)?;
    write_export_stamp(&staging, size)?;
    publish_export(&staging, export_dir)?;
    Ok(export_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn cached_validate_body() -> String {
        let suffix = "}\n";
        let padding_len = MIN_VALIDATE_BYTES as usize - suffix.len();
        format!("{}{}", "x".repeat(padding_len), suffix)
    }

    fn write_cached_export(export_dir: &Path, body: &str) {
        let validate = validate_proto_path(export_dir);
        fs::create_dir_all(validate.parent().expect("parent")).expect("mkdir");
        fs::write(&validate, body).expect("write validate.proto");
        write_export_stamp(export_dir, body.len() as u64).expect("write stamp");
    }

    #[test]
    fn ensure_proto_deps_export_reuses_existing_validate_proto() {
        let export_dir = tempfile::tempdir().expect("tempdir");
        let body = cached_validate_body();
        write_cached_export(export_dir.path(), &body);

        let got = ensure_proto_deps_export(Path::new("/unused"), export_dir.path(), false)
            .expect("cache hit");
        assert_eq!(got, export_dir.path());
        assert_eq!(
            fs::read_to_string(validate_proto_path(export_dir.path())).expect("read"),
            body
        );
    }

    #[test]
    fn ensure_proto_deps_export_rejects_truncated_without_stamp() {
        let export_dir = tempfile::tempdir().expect("tempdir");
        let validate = validate_proto_path(export_dir.path());
        fs::create_dir_all(validate.parent().expect("parent")).expect("mkdir");
        fs::write(&validate, "// truncated\n").expect("write validate.proto");

        assert!(!export_is_current(export_dir.path()).expect("check export"));
    }

    #[test]
    fn ensure_proto_deps_export_rejects_stale_stamp() {
        let export_dir = tempfile::tempdir().expect("tempdir");
        let body = cached_validate_body();
        write_cached_export(export_dir.path(), &body);
        fs::write(validate_proto_path(export_dir.path()), "// changed\n").expect("rewrite");

        assert!(!export_is_current(export_dir.path()).expect("check export"));
    }
}
