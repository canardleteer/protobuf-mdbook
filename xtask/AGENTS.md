# xtask instructions

The crate is a root workspace member (`protobuf-mdbook-xtask`).
`.cargo/config.toml` maps `cargo xtask` to
`cargo run --package protobuf-mdbook-xtask --`.

These commands are synchronous. Do not add `tokio` or `tracing` as ceremony.

## Command policy

- `check` registers `toolchain`, `buf-lint`, `fmt`, `check`, `clippy`, `test`,
  `build-plugin`, `highlight-rust`, `book-init`, and `book-links` in that order.
  It is fail-fast, runs all ten by default, supports mutually exclusive `--only`
  and `--exclude`, and defaults feature-aware Cargo commands to `--all-features`.
  `fmt` is `cargo fmt --all -- --check` plus `buf format --diff` on
  `examples/proto/`. `book-init` is markdown-only into `./api-book`. `ci` is a
  visible alias over exactly the same implementation.
- Keep CI and workflow declarations aligned with `check` when behavior changes.
  The xtask must remain provider-agnostic and must not inspect those
  declarations itself.
- `image` (visible alias `docker`) builds `Dockerfile` from the repository root
  as `protobuf-mdbook:local` for `linux/amd64`. `auto` tries Docker before
  Buildah. Smoke is `--version` with no network, plus image inspect for
  `nobody`, the `/protoc-gen-mdbook` entrypoint, and `linux/amd64`. Temporary
  containers are always removed. Local images are retained. Images are never
  pushed.
- `coverage` supports `llvm-cov` and `tarpaulin`. Reports are
  `target/coverage/llvm-cov/html/index.html` and
  `target/coverage/tarpaulin/tarpaulin-report.html`. Both `coverage --open` and
  `coverage-open` generate a fresh report first.
- `profile` and `profile-open` are omitted. There is no established local
  profiling workload, and adding a synthetic one would be artificial.
- `mcp-test` is omitted. This repository does not expose a stdio MCP server.

Repo-specific write and guided commands stay as extra handles: `fmt`,
`fmt-check`, `book-init`, `book-refresh`, `book-links`, `book-build`,
`buf-lint`, `buf-format`, `buf-format-check`, `check-toolchain`,
`check-highlight-rust`, `update-highlight-golden`, `update-golden`,
`rumdl-check`, and `rumdl-fmt`.

## Tool guidance

Probe only tools selected by the command. A failed launch is an unusable tool,
not an absent one. Recommend these exact Cargo installs when applicable:

```text
cargo install --locked cargo-llvm-cov
cargo install --locked cargo-tarpaulin
cargo install buf-toolchain --locked --version 1.73.0-rc.1
```

Use `rustup component add rustfmt` or `rustup component add clippy` for missing
Rust components. Link to official Docker or Buildah installation instructions.
Do not guess a platform package-manager command.

## Implementation and validation

Represent subprocesses as a program plus OS argument vector. Keep `main.rs`
thin, use Clap derive types, and retain parser, selection, feature, alias,
command-construction, and failure-path tests.

Prefer adding Cargo-aware cross-platform orchestration here over adding a
Python wrapper. This repository has no retained Python orchestration.

From the repository root:

```shell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo xtask check --only fmt,check
```

The image and coverage commands depend on selected local tools and should be
exercised when their implementations change.
