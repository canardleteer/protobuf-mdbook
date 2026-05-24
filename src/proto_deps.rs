//! Export BSR deps from a Buf module for protoc `-I` (never protoc inputs).

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

const VALIDATE_PROTO: &str = "buf/validate/validate.proto";

/// Path to the exported Protovalidate schema under `export_dir`.
pub fn validate_proto_path(export_dir: &Path) -> PathBuf {
    export_dir.join(VALIDATE_PROTO)
}

/// Export `proto_root` and its `buf.yaml` deps into `export_dir` for protoc import paths.
///
/// When `refresh` is false, reuses an existing export if `buf/validate/validate.proto` is present.
/// When `refresh` is true, clears `export_dir` first (xtask default).
pub fn ensure_proto_deps_export(
    proto_root: &Path,
    export_dir: &Path,
    refresh: bool,
) -> Result<PathBuf> {
    let validate = validate_proto_path(export_dir);
    if !refresh && validate.is_file() {
        return Ok(export_dir.to_path_buf());
    }

    if export_dir.exists() {
        std::fs::remove_dir_all(export_dir)
            .with_context(|| format!("clear proto-deps export at {}", export_dir.display()))?;
    }
    if let Some(parent) = export_dir.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let status = Command::new("buf")
        .current_dir(proto_root)
        .args(["export", ".", "--output"])
        .arg(export_dir)
        .status()
        .context("buf export")?;
    if !status.success() {
        bail!("buf export failed");
    }
    if !validate.is_file() {
        bail!(
            "buf export missing {}; run `buf dep update` in {}",
            validate.display(),
            proto_root.display()
        );
    }
    Ok(export_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn ensure_proto_deps_export_reuses_existing_validate_proto() {
        let export_dir = tempfile::tempdir().expect("tempdir");
        let validate = validate_proto_path(export_dir.path());
        fs::create_dir_all(validate.parent().expect("parent")).expect("mkdir");
        fs::write(&validate, "// stub\n").expect("write validate.proto");

        let got = ensure_proto_deps_export(Path::new("/unused"), export_dir.path(), false)
            .expect("cache hit");
        assert_eq!(got, export_dir.path());
        assert!(validate.is_file());
        assert_eq!(fs::read_to_string(&validate).expect("read"), "// stub\n");
    }
}
