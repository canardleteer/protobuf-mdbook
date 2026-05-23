//! Crate-local CI and example generation for `protoc-gen-mdbook`.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

const CRATE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
const WORKSPACE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

/// Repo-local mdBook output (`--mdbook_out`). Guided `book-*` tasks always target this tree.
const API_BOOK_DIR: &str = "api-book";

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// buf-lint, fmt-check, clippy, test, build-plugin, book-init --markdown-only, book-links
    Ci,
    /// `cargo fmt` on this workspace plus `buf format -w` on `examples/proto/`.
    Fmt,
    /// `cargo fmt --check` plus `buf format --diff` on `examples/proto/`.
    FmtCheck,
    Clippy,
    Test,
    BuildPlugin,
    /// Scaffold or regenerate `./api-book` from `examples/proto/`.
    /// Uses plugin path defaults (`markdown_root=src/packages`, `summary_path=src/SUMMARY.md`).
    BookInit {
        #[arg(long, default_value = "package")]
        layout: String,
        #[arg(long)]
        summary: bool,
        /// Markdown only (no mdBook scaffold); wipes `./api-book` first. CI uses this.
        #[arg(long)]
        markdown_only: bool,
    },
    /// Refresh `./api-book` package markdown without `init` (preserves book.toml, theme, README).
    /// Passes `book=` so paths are loaded from `book.toml` via mdbook-core.
    BookRefresh {
        #[arg(long, default_value = "package")]
        layout: String,
        #[arg(long)]
        summary: bool,
    },
    /// Resolve in-page links and mdBook heading anchors in `./api-book/`.
    BookLinks,
    /// Run `mdbook build` on `./api-book/` (requires `book.toml` from `book-init`).
    BookBuild,
    RumdlCheck,
    RumdlFmt,
    /// Build linux/amd64 scratch image and run runtime smoke checks.
    Docker,
    /// Verify vendored highlight.js protobuf grammar matches meta.json and upstream.
    CheckHighlightjsVendor,
    /// `buf lint` on `examples/proto/` (requires `buf` on PATH; uses `buf.lock`).
    BufLint,
    /// `buf format -w` on `examples/proto/` (also run via `cargo xtask fmt`).
    BufFormat,
    /// `buf format --diff` on `examples/proto/` (also run via `cargo xtask fmt-check`).
    BufFormatCheck,
    /// Verify active Rust toolchain matches `rust-toolchain.toml`.
    CheckToolchain {
        /// Exit with failure when the active toolchain diverges from the pin.
        #[arg(long)]
        strict: bool,
    },
    /// LLVM source coverage via `cargo llvm-cov` (requires `cargo install cargo-llvm-cov --locked`).
    Coverage {
        /// Generate HTML and open in a browser.
        #[arg(long)]
        open: bool,
        /// Emit `lcov.info` for CI upload.
        #[arg(long)]
        lcov: bool,
        #[arg(long, default_value = "lcov.info")]
        output_path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Ci => {
            run("check-toolchain", || check_toolchain(true))?;
            run("buf-lint", buf_lint)?;
            run("fmt-check", fmt_check)?;
            run("clippy", clippy)?;
            run("test", test)?;
            run("build-plugin", build_plugin)?;
            run("check-highlightjs-vendor", check_highlightjs_vendor)?;
            run("book-init", || book_init("package", false, true))?;
            run("book-links", book_links)?;
            Ok(())
        }
        Cmd::Fmt => fmt(),
        Cmd::FmtCheck => fmt_check(),
        Cmd::Clippy => clippy(),
        Cmd::Test => test(),
        Cmd::BuildPlugin => build_plugin(),
        Cmd::BookInit {
            layout,
            summary,
            markdown_only,
        } => book_init(&layout, summary, markdown_only),
        Cmd::BookRefresh { layout, summary } => book_refresh(&layout, summary),
        Cmd::BookLinks => book_links(),
        Cmd::BookBuild => book_build(),
        Cmd::RumdlCheck => rumdl_check(),
        Cmd::RumdlFmt => rumdl_fmt(),
        Cmd::Docker => docker(),
        Cmd::CheckHighlightjsVendor => check_highlightjs_vendor(),
        Cmd::BufLint => buf_lint(),
        Cmd::BufFormat => buf_format_write(),
        Cmd::BufFormatCheck => buf_format_check(),
        Cmd::CheckToolchain { strict } => check_toolchain(strict),
        Cmd::Coverage {
            open,
            lcov,
            output_path,
        } => coverage(open, lcov, &output_path),
    }
}

fn run(name: &str, f: impl FnOnce() -> Result<()>) -> Result<()> {
    eprintln!("xtask: {name}");
    f()
}

fn cargo(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(WORKSPACE_ROOT)
        .status()
        .with_context(|| format!("cargo {}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        bail!("cargo {} failed ({status})", args.join(" "));
    }
}

fn fmt() -> Result<()> {
    cargo_fmt(&[])?;
    buf_format_write()
}

fn fmt_check() -> Result<()> {
    cargo_fmt(&["--check"])?;
    buf_format_check()
}

fn cargo_fmt(extra: &[&str]) -> Result<()> {
    let mut args = vec![
        "fmt",
        "-p",
        "protoc-gen-mdbook",
        "-p",
        "protoc-gen-mdbook-xtask",
        "--",
    ];
    args.extend_from_slice(extra);
    cargo(&args)
}

fn clippy() -> Result<()> {
    cargo(&[
        "clippy",
        "--locked",
        "-p",
        "protoc-gen-mdbook",
        "-p",
        "protoc-gen-mdbook-xtask",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])
}

fn test() -> Result<()> {
    cargo(&["test", "--locked", "-p", "protoc-gen-mdbook"])
}

fn build_plugin() -> Result<()> {
    cargo(&["build", "--locked", "--release", "-p", "protoc-gen-mdbook"])
}

fn plugin_path() -> Result<PathBuf> {
    let mut path = Path::new(WORKSPACE_ROOT).join("target/release/protoc-gen-mdbook");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    let path = path
        .canonicalize()
        .with_context(|| format!("locate plugin binary at {}", path.display()))?;
    Ok(path)
}

fn examples_proto() -> PathBuf {
    Path::new(CRATE_DIR).join("examples/proto")
}

/// Exported BSR deps (`buf export`) for protoc `-I` (never protoc inputs).
fn proto_deps_export() -> PathBuf {
    Path::new(WORKSPACE_ROOT).join("target/proto-deps")
}

fn buf_command() -> Result<()> {
    let status = Command::new("buf")
        .arg("--version")
        .status()
        .context("spawn buf (--version)")?;
    if status.success() {
        Ok(())
    } else {
        bail!(
            "buf CLI not found or failed; install with \
             `cargo install buf-toolchain --locked --version 1.69.0` \
             (see Dockerfile buf-anchor)"
        );
    }
}

/// Export `examples/proto` and its `buf.yaml` deps for protoc import paths.
fn ensure_proto_deps_export() -> Result<PathBuf> {
    buf_command()?;
    protoc_gen_mdbook::proto_deps::ensure_proto_deps_export(
        &examples_proto(),
        &proto_deps_export(),
        true,
    )
}

fn buf_lint() -> Result<()> {
    buf_command()?;
    let proto_root = examples_proto();
    let status = Command::new("buf")
        .current_dir(&proto_root)
        .arg("lint")
        .status()
        .context("buf lint")?;
    if status.success() {
        Ok(())
    } else {
        bail!("buf lint failed ({status})");
    }
}

fn buf_format_write() -> Result<()> {
    buf_command()?;
    let proto_root = examples_proto();
    let status = Command::new("buf")
        .current_dir(&proto_root)
        .args(["format", "-w"])
        .status()
        .context("buf format -w")?;
    if status.success() {
        Ok(())
    } else {
        bail!("buf format -w failed ({status})");
    }
}

fn buf_format_check() -> Result<()> {
    buf_command()?;
    let proto_root = examples_proto();
    let status = Command::new("buf")
        .current_dir(&proto_root)
        .args(["format", "--diff"])
        .status()
        .context("buf format --diff")?;
    if status.success() {
        Ok(())
    } else {
        bail!("buf format --diff failed ({status}); run `cargo xtask fmt`");
    }
}

fn api_book() -> PathBuf {
    Path::new(WORKSPACE_ROOT).join(API_BOOK_DIR)
}

/// `--mdbook_opt` for guided `./api-book` runs.
fn mdbook_opt(layout: &str, summary: bool, init: bool, with_book: bool) -> String {
    let mut opt = if init {
        format!("init,layout={layout}")
    } else {
        format!("layout={layout}")
    };
    if summary {
        opt.push_str(",summary");
    }
    if with_book {
        let book = api_book();
        let book_s = book.to_string_lossy();
        opt.push_str(&format!(",book={book_s},mdbook_out={book_s}"));
    }
    opt
}

/// Every `.proto` under `examples/proto/` (recursive), relative to that directory.
/// Does not scan `target/proto-deps/` (BSR export for protoc `-I` only).
fn collect_example_protos(proto_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_example_protos_rec(proto_root, proto_root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_example_protos_rec(
    proto_root: &Path,
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_example_protos_rec(proto_root, &path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("proto") {
            files.push(
                path.strip_prefix(proto_root)
                    .context("strip prefix")?
                    .into(),
            );
        }
    }
    Ok(())
}

fn run_protoc_on_examples(out_dir: &Path, mdbook_opt: &str) -> Result<()> {
    build_plugin()?;
    let protoc = protoc_bin()?;
    let plugin = plugin_path()?;
    let proto_root = examples_proto();
    let inputs = collect_example_protos(&proto_root)?;
    if inputs.is_empty() {
        bail!("no .proto files under {}", proto_root.display());
    }
    eprintln!(
        "xtask: protoc {} proto file(s) → {} (opt={mdbook_opt})",
        inputs.len(),
        out_dir.display()
    );

    if out_dir.exists() && mdbook_opt.contains("init") {
        std::fs::remove_dir_all(out_dir).context("clear output before init")?;
    }
    std::fs::create_dir_all(out_dir)?;

    let deps = ensure_proto_deps_export()?;
    let mut cmd = Command::new(protoc);
    cmd.current_dir(&proto_root)
        .arg("-I")
        .arg(".")
        .arg("-I")
        .arg(&deps)
        .arg(format!("--plugin=protoc-gen-mdbook={}", plugin.display()))
        .arg(format!("--mdbook_out={}", out_dir.display()))
        .arg(format!("--mdbook_opt={mdbook_opt}"));
    for rel in &inputs {
        cmd.arg(rel);
    }
    let status = cmd.status().context("protoc")?;
    if !status.success() {
        bail!("protoc failed");
    }
    Ok(())
}

fn book_init(layout: &str, summary: bool, markdown_only: bool) -> Result<()> {
    let out = api_book();
    if markdown_only && out.exists() {
        std::fs::remove_dir_all(&out).context("clear api-book before markdown-only init")?;
    }
    let init = !markdown_only;
    run_protoc_on_examples(&out, &mdbook_opt(layout, summary, init, false))
}

fn book_refresh(layout: &str, summary: bool) -> Result<()> {
    let out = api_book();
    if !out.join("book.toml").is_file() {
        bail!(
            "{} is missing book.toml; run `cargo xtask book-init` first",
            out.display()
        );
    }
    run_protoc_on_examples(&out, &mdbook_opt(layout, summary, false, true))
}

fn protoc_bin() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("PROTOC_BIN")))
}

fn book_links() -> Result<()> {
    let out_dir = api_book();
    let markdown_root = if out_dir.join("book.toml").is_file() {
        protoc_gen_mdbook::book_config::markdown_root_dir(&out_dir)
            .with_context(|| format!("load paths from {}", out_dir.join("book.toml").display()))?
    } else {
        out_dir.join("src/packages")
    };
    if !markdown_root.is_dir() {
        bail!(
            "{} is missing generated markdown under {}; run `cargo xtask book-init --markdown-only` or `book-init` first",
            out_dir.display(),
            markdown_root
                .strip_prefix(&out_dir)
                .unwrap_or(&markdown_root)
                .display()
        );
    }
    protoc_gen_mdbook::link_check::assert_tree(&out_dir).context("markdown link check")
}

fn book_build() -> Result<()> {
    let out_dir = api_book();
    if !out_dir.join("book.toml").is_file() {
        bail!(
            "{} is missing book.toml; run `cargo xtask book-init` first",
            out_dir.display()
        );
    }
    let status = Command::new("mdbook")
        .args(["build"])
        .current_dir(&out_dir)
        .status()
        .context("mdbook build")?;
    if status.success() {
        Ok(())
    } else {
        bail!("mdbook build failed ({status})");
    }
}

fn rumdl_check() -> Result<()> {
    let readme = Path::new(CRATE_DIR).join("README.md");
    let status = Command::new("rumdl")
        .args(["check", readme.to_str().expect("utf8")])
        .status()
        .context("rumdl check")?;
    if status.success() {
        Ok(())
    } else {
        bail!("rumdl check failed");
    }
}

fn rumdl_fmt() -> Result<()> {
    let readme = Path::new(CRATE_DIR).join("README.md");
    let status = Command::new("rumdl")
        .args(["fmt", readme.to_str().expect("utf8")])
        .status()
        .context("rumdl fmt")?;
    if status.success() {
        Ok(())
    } else {
        bail!("rumdl fmt failed");
    }
}

const DOCKER_IMAGE: &str = "protoc-gen-mdbook:local";

fn docker() -> Result<()> {
    docker_build()?;
    docker_smoke_version()?;
    docker_smoke_config()?;
    Ok(())
}

fn docker_build() -> Result<()> {
    eprintln!("xtask: docker build --platform linux/amd64 -t {DOCKER_IMAGE}");
    let status = Command::new("docker")
        .args([
            "build",
            "--platform",
            "linux/amd64",
            "-f",
            &format!("{CRATE_DIR}/Dockerfile"),
            "-t",
            DOCKER_IMAGE,
            WORKSPACE_ROOT,
        ])
        .status()
        .context("docker build")?;
    if status.success() {
        Ok(())
    } else {
        bail!("docker build failed ({status})");
    }
}

/// Runtime image must answer `--version`.
fn docker_smoke_version() -> Result<()> {
    eprintln!("xtask: docker run --version smoke");
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--entrypoint",
            "/protoc-gen-mdbook",
            DOCKER_IMAGE,
            "--version",
        ])
        .output()
        .context("docker run --version")?;
    if !output.status.success() {
        bail!(
            "docker --version failed ({:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("protoc-gen-mdbook") {
        bail!("docker --version stdout missing plugin name: {stdout}");
    }
    let pin = protoc_gen_mdbook::mdbook_version();
    if !stdout.contains(pin) {
        bail!("docker --version stdout missing mdbook pin {pin}: {stdout}");
    }
    eprintln!("xtask: docker --version ok: {}", stdout.trim());
    Ok(())
}

/// Scratch runtime: non-root user, static entrypoint only.
fn docker_smoke_config() -> Result<()> {
    eprintln!("xtask: docker inspect config smoke");
    let user = docker_inspect_format("{{.Config.User}}")?;
    if user != "nobody" {
        bail!("expected Config.User=nobody, got {user:?}");
    }
    let entrypoint = docker_inspect_format("{{json .Config.Entrypoint}}")?;
    if !entrypoint.contains("protoc-gen-mdbook") {
        bail!("unexpected Entrypoint: {entrypoint}");
    }
    let os = docker_inspect_format("{{.Os}}")?;
    let arch = docker_inspect_format("{{.Architecture}}")?;
    if os != "linux" || arch != "amd64" {
        bail!("expected linux/amd64 image, got {os}/{arch}");
    }
    Ok(())
}

fn sha256_hex(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(sha256_hex_bytes(&bytes))
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Compare vendored Highlight.js grammars against `assets/highlightjs/*.meta.json`.
fn check_highlightjs_vendor() -> Result<()> {
    let highlight_dir = Path::new(CRATE_DIR).join("assets/highlightjs");
    let mut meta_paths: Vec<PathBuf> = std::fs::read_dir(&highlight_dir)
        .with_context(|| format!("read_dir {}", highlight_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|x| x.to_str()) == Some("json")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".meta.json"))
        })
        .collect();
    meta_paths.sort();

    if meta_paths.is_empty() {
        bail!("no *.meta.json under {}", highlight_dir.display());
    }

    let compiled = protoc_gen_mdbook::mdbook_version();
    let mut mdbook_pin_checked = false;

    for meta_path in &meta_paths {
        let meta: serde_json::Value =
            serde_json::from_slice(&std::fs::read(meta_path).context("read meta.json")?)
                .context("parse meta.json")?;
        let grammar = meta["grammar"]
            .as_str()
            .or_else(|| {
                meta_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.strip_suffix(".meta").unwrap_or(s))
            })
            .context("grammar name in meta.json")?;

        if !mdbook_pin_checked {
            let mdbook_pin = meta["mdbook_pin"]
                .as_str()
                .context("mdbook_pin in meta.json")?;
            if !compiled.contains(mdbook_pin) {
                bail!(
                    "mdbook pin mismatch: meta.json mdbook_pin={mdbook_pin:?}, \
                     protoc_gen_mdbook::mdbook_version()={compiled:?}"
                );
            }
            mdbook_pin_checked = true;
        }

        let vendored_file = meta["vendored_file"]
            .as_str()
            .context("vendored_file in meta.json")?;
        let vendored_path = highlight_dir.join(vendored_file);
        let vendored_sha = sha256_hex(&vendored_path)?;
        let expected_vendored = meta["vendored_sha256"]
            .as_str()
            .context("vendored_sha256 in meta.json")?;
        if vendored_sha != expected_vendored {
            bail!(
                "vendored_sha256 mismatch for {grammar}: file={vendored_sha}, \
                 meta.json={expected_vendored}; update {} after editing {vendored_file}",
                meta_path.file_name().unwrap().to_string_lossy()
            );
        }

        let upstream_url = meta
            .get("upstream_file_url")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());
        let expected_upstream = meta
            .get("upstream_sha256")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());
        match (upstream_url, expected_upstream) {
            (None, None) => {
                eprintln!("xtask: {grammar} ok (local grammar, vendored hash)");
            }
            (Some(url), Some(expected)) => {
                eprintln!("xtask: fetching upstream {url} ({grammar})");
                let output = Command::new("curl")
                    .args(["-fsSL", url])
                    .output()
                    .with_context(|| format!("curl upstream {grammar}"))?;
                if !output.status.success() {
                    bail!(
                        "curl failed for {grammar} ({:?}): {}",
                        output.status.code(),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                let upstream_sha = sha256_hex_bytes(&output.stdout);
                if upstream_sha != expected {
                    bail!(
                        "upstream_sha256 drift for {grammar}: fetched={upstream_sha}, \
                         meta.json={expected}; review and re-vendor if needed"
                    );
                }
                eprintln!("xtask: {grammar} ok (vendored + upstream hashes)");
            }
            _ => bail!(
                "invalid upstream fields in {}: upstream_file_url and upstream_sha256 \
                 must both be set or both absent/null",
                meta_path.display()
            ),
        }
    }

    eprintln!(
        "xtask: check-highlightjs-vendor ok ({} grammar(s), mdbook pin)",
        meta_paths.len()
    );
    Ok(())
}

fn docker_inspect_format(format: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["image", "inspect", "--format", format, DOCKER_IMAGE])
        .output()
        .with_context(|| format!("docker image inspect --format {format}"))?;
    if !output.status.success() {
        bail!(
            "docker image inspect failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[derive(Debug)]
struct ToolchainPin {
    channel: String,
    components: Vec<String>,
}

fn read_toolchain_pin() -> Result<ToolchainPin> {
    let path = Path::new(WORKSPACE_ROOT).join("rust-toolchain.toml");
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut channel = None;
    let mut components = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("channel = ") {
            channel = Some(trim_toml_string(v));
        } else if let Some(v) = line.strip_prefix("components = ") {
            components = parse_toml_string_array(v);
        }
    }
    let channel = channel.context("rust-toolchain.toml missing channel")?;
    Ok(ToolchainPin {
        channel,
        components,
    })
}

fn trim_toml_string(raw: &str) -> String {
    raw.trim().trim_matches('"').trim_matches('\'').to_string()
}

fn parse_toml_string_array(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| trim_toml_string(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn command_output(bin: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("{bin} {}", args.join(" ")))?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        bail!(
            "{bin} {} failed ({:?}): {}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

fn check_toolchain(strict: bool) -> Result<()> {
    let pin = read_toolchain_pin()?;
    let rustc_v = command_output("rustc", &["-V"])?;
    let active = rustc_v
        .split_whitespace()
        .nth(1)
        .context("parse rustc -V")?;

    let mut divergent = false;
    if !rustc_v.contains(&pin.channel) {
        divergent = true;
        eprintln!(
            "xtask: warning: active rustc ({active}) does not match rust-toolchain.toml channel ({})",
            pin.channel
        );
    }

    let installed = command_output("rustup", &["component", "list", "--installed"])?;
    for component in &pin.components {
        if !installed.lines().any(|line| line.starts_with(component)) {
            divergent = true;
            eprintln!(
                "xtask: warning: rustup component {component:?} from rust-toolchain.toml is not installed"
            );
        }
    }

    if divergent {
        if strict {
            bail!(
                "toolchain diverges from rust-toolchain.toml (channel={}, components={:?}); \
                 run `rustup toolchain install` in the repo root",
                pin.channel,
                pin.components
            );
        }
        eprintln!("xtask: check-toolchain: divergent (non-strict mode; use --strict to fail)");
    } else {
        eprintln!(
            "xtask: check-toolchain ok (channel={}, components={:?})",
            pin.channel, pin.components
        );
    }
    Ok(())
}

fn ensure_cargo_llvm_cov() -> Result<()> {
    let status = Command::new("cargo")
        .args(["llvm-cov", "--version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("cargo llvm-cov --version")?;
    if status.success() {
        return Ok(());
    }
    bail!(
        "cargo llvm-cov not found; install with: cargo install cargo-llvm-cov --locked\n\
         also run: rustup component add llvm-tools-preview"
    );
}

fn coverage(open: bool, lcov: bool, output_path: &Path) -> Result<()> {
    ensure_cargo_llvm_cov()?;
    let mut args = vec![
        "llvm-cov",
        "--locked",
        "-p",
        "protoc-gen-mdbook",
        "--all-targets",
    ];
    if open {
        args.push("--open");
    } else if lcov {
        args.extend(["--lcov", "--output-path"]);
        args.push(output_path.to_str().context("lcov output path utf8")?);
    } else {
        args.push("--html");
    }
    let result = cargo(&args);
    if result.is_err() {
        eprintln!(
            "hint: if llvm-cov failed to find tools, run: rustup component add llvm-tools-preview"
        );
    }
    result
}
