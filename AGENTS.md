# Contributor guide — `protobuf-mdbook` & `protoc-gen-mdbook`

## Toolchain

- Rust is pinned in `rust-toolchain.toml` (exact stable channel, not rolling
  `stable`). Bump the pin there when advancing the compiler.
- CI installs the pin via `dtolnay/rust-toolchain@1.95.0` (matches `rust-toolchain.toml`); `cargo xtask ci`
  runs `check-toolchain --strict` first. Locally: `cargo xtask check-toolchain`
  warns on drift; `--strict` fails.

## mdBook version (single source of truth)

- **Pin:** root [`Cargo.toml`](Cargo.toml) `[workspace.dependencies]` entries
  `mdbook-core`, `mdbook-driver`, and `mdbook-preprocessor` (keep versions aligned).
- **Runtime truth:** `protobuf-mdbook --version`, `protoc-gen-mdbook --version`, and
  `mdbook-protobuf-highlight --version` / `-V` print the compiled pin via
  [`mdbook_core::MDBOOK_VERSION`](https://docs.rs/mdbook-core) (exposed as
  `protobuf_mdbook::mdbook_version()` in the library).
- **Do not** duplicate the mdBook version number in README, AGENTS, or comments —
  refer readers to `Cargo.toml` and `--version` instead.
- **Bump workflow:** edit `Cargo.toml` only, run `cargo xtask ci`. Integration tests
  assert `--version` includes `mdbook_version()`; no manual doc version edits required.

## mdBook public API (prefer over reimplementation)

- **Prefer** pinned public crates (`mdbook-core`, `mdbook-driver`, `mdbook-summary`) over
  duplicating mdBook behavior. Use `mdbook-html` only when a helper is actually public at the pin.
- **Init:** `mdbook_driver::MDBook::init` + `BookBuilder` in a temp dir ([`src/init.rs`](src/init.rs)) —
  do not hand-copy theme trees.
- **Refresh paths:** `mdbook_core::config::Config::from_disk` + `book.src` ([`src/book_config.rs`](src/book_config.rs)).
- **SUMMARY:** build `mdbook_summary::Summary` / `Link`, render to markdown, then `parse_summary` —
  **warn on stderr** if parse fails (still emit); unit/integration tests assert parse success on fixtures.
- **Do not** call `MDBook::load` / `mdbook build` inside the plugin; `xtask` runs `mdbook build` for validation.
- **No public API** for init README template or highlight `book.toml` / `theme/protobuf-highlight.css`
  wiring — string edits after `BookBuilder` are acceptable (documented in [`src/init.rs`](src/init.rs)).

| Area | Verdict | Action |
|------|---------|--------|
| Init scaffold | OK — BookBuilder | None |
| `Config::from_disk` | OK | None |
| SUMMARY | `mdbook-summary` tree + emitter | `parse_summary` warn-only at runtime |
| `heading_slug` | `mdbook-html::utils::id_from_content` is crate-private | Local shim + parity test on bump |
| Init README / highlight theme | No API | `book.toml` preprocessor + `theme/protobuf-highlight.css` via [`src/highlight/book_toml.rs`](src/highlight/book_toml.rs) |
| Proto API pages | Plugin domain | Hand-written render |

## Companion proto markdown

- **Default on:** copy hand-written `.md` beside included `.proto` trees into `{markdown_root}/`
  (flat names: `dir.with.dots.<stem>.md`). Opt out: `no_proto_markdown`.
- **Never synthesize** module README bodies — copy bytes only. Init book-level [`README.md`](src/init.rs)
  remains the only generated onboarding doc.
- **Layout rule:** discovery and SUMMARY nesting follow **filesystem directory layout**, not protobuf
  `import` graph.
- **SUMMARY:** best-effort nav with **minimal subchaptering** (`SUMMARY_MAX_DEPTH` in [`src/summary/nav_tree.rs`](src/summary/nav_tree.rs)) —
  collapse pass-through dirs, flatten when too deep. Link titles: bare at corpus root (`acme/`) and
  for nested subchapters; `{module.path} - {title}` for section companions (dot path from output
  filename); generated package pages use the package name only. Authors own final `SUMMARY.md`.
- **Refresh:** companion files under `markdown_root` are overwritten like package pages.

## Init path

Always scaffold mdBook projects through `mdbook_driver::MDBook::init` +
`BookBuilder` in a temp directory (`init.rs`), then merge generated markdown
into `CodeGeneratorResponse` files. Do not hand-copy theme trees.

- **`init` opt-in:** full mdBook tree, package-only SUMMARY (`summary_path=`, default `src/SUMMARY.md`), `README.md` beside `book.toml`.
- **Default:** markdown under `markdown_root=` only (default `src/packages/`); optional `summary` writes `summary_path=` without touching mdBook scaffold.
- Init strips mdBook’s default `src/SUMMARY.md` and `src/chapter_1.md` stubs (and a custom `summary_path=` if set) before merge.
- Default book title when `title=` is omitted: **Protobuf documentation** (not inferred from package names).
- **Syntax highlighting (default on `init`):** wires `[preprocessor.protobuf-highlight]`
  in `book.toml` for **`mdbook-protobuf-highlight`** (build-time HTML; see
  [`src/highlight/`](src/highlight/)). Opt out with `no_proto_highlight` and/or
  `no_cel_highlight`. User-facing details: root `README.md` **Syntax highlighting**.
- **CEL fences:** message-level `(buf.validate.message).cel` is split from `protobuf`
  fences into adjacent ` ```cel ` blocks at generation time ([`src/render/cel_fence.rs`](src/render/cel_fence.rs))
  and again at mdbook build for unsplit companion markdown ([`src/highlight/cel_split.rs`](src/highlight/cel_split.rs)).

## Highlight grammars (reference + CI)

- Reference JS grammars: [`assets/highlightjs/`](assets/highlightjs/) (`protobuf-10.js`,
  `cel-10.js`, `*.meta.json`, `NOTICE`) — used as the spec when porting rules to Rust.
- Runtime highlighter: [`src/highlight/`](src/highlight/) (`protobuf.rs`, `cel.rs`).
- **`check-highlight-rust`:** golden HTML parity in `tests/fixtures/highlight/` (part of
  `cargo xtask ci`). Refresh with `cargo xtask update-highlight-golden` after intentional
  grammar edits.

## Examples and output

- Authoritative example protos: `examples/proto/` (Buf module; BSR dep
  `buf.build/bufbuild/protovalidate` in `buf.yaml` / `buf.lock` — **never vendored**
  in-repo). `buf lint` / `buf format` resolve deps via Buf; protoc runs export on demand.
- Format locally: `cargo xtask fmt` (`cargo fmt` + `buf format -w`). CI uses `fmt-check`
  (`cargo fmt --check` + `buf format --diff`) and `buf lint` (Buf CLI on PATH; CI installs
  1.69.0 via `cargo install buf-toolchain --locked --version 1.69.0`). Shared helper
  `proto_deps::ensure_proto_deps_export` writes gitignored `target/proto-deps/` for protoc
  `-I` only — never pass exported files as inputs (`cargo xtask book-*` and link-check tests
  call it automatically).
- **Canonical protoc inputs:** [`src/examples.rs`](src/examples.rs) (`EXAMPLE_PROTO_INPUTS`) —
  shared by `cargo xtask book-*`, integration tests, and link-check; excludes exported
  `buf/validate/validate.proto`. Update the list when adding fixture protos under `acme/`.
- Generated book at `./api-book/` (gitignored): CI runs `cargo xtask book-init --markdown-only` then `book-links`; local preview uses `book-init` once, then `book-refresh`. Guided tasks accept `--generator protoc` (default, CI) or `--generator cli` (`protobuf-mdbook` + Buf on `examples/proto/`).

## Output conventions

Options for **`protoc-gen-mdbook`** are comma-separated on **`--mdbook_opt=`**
(or `CodeGeneratorRequest.parameter`). **`protobuf-mdbook`** exposes the same
semantics as native clap flags (hyphens); see README **Generator options**.

### Where files land (protoc contract)

protoc writes each `CodeGeneratorResponse` file path relative to **`--mdbook_out`** only.

| Option | Default | Role |
|--------|---------|------|
| `--mdbook_out` | *(protoc flag)* | Output root directory |
| `book_root=` | `.` | Optional subdirectory under `--mdbook_out` |
| `book=` | — | Book root or `book.toml`; loads `[book] src` via `mdbook_core::config::Config::from_disk` |
| `mdbook_out=` | — | Validate `--mdbook_out` matches `book=` (stderr warning if divergent) |
| `markdown_root=` | `src/packages` | API markdown directory (or `{src}/packages` when `book=` set) |
| `summary_path=` | `src/SUMMARY.md` | SUMMARY when `summary`/`init` (or `{src}/SUMMARY.md` when `book=` set) |

Explicit `markdown_root=` / `summary_path=` / `book_root=` override `book=` inference.
Implementation: [`src/book_config.rs`](src/book_config.rs). **Default refresh** (no `init`, no `summary`) overwrites only files under `markdown_root` — it does not touch `book.toml`, theme, init `README.md`, or SUMMARY unless you pass `summary`/`init`.

### Defaults (no flags)

| Setting | Default |
|---------|---------|
| Output mode | **Markdown only** — `{markdown_root}/**/*.md` |
| `layout=` | **`package`** — one page per protobuf package |
| SUMMARY | **Not emitted** |
| mdBook scaffold (`book.toml`, theme, init `README.md`) | **Not emitted** |

Example (ongoing refresh): `--mdbook_opt=layout=package,book=./api-book,mdbook_out=./api-book`

### Output modes (opt-in)

| Flag | Specifies | Emits |
|------|-----------|--------|
| *(none)* | default | Package markdown under `markdown_root` only |
| `summary` | nav without mdBook | Above + `summary_path` |
| `init` | one-time mdBook scaffold | Full mdBook tree + package-only SUMMARY + `README.md` beside `book.toml` |

- **`init`** implies `summary` (SUMMARY is always written) but **always package-only** links in SUMMARY — one line per package, no `Message …` / `Enum …` lines — even when `layout=entity` or `layout=split` (entity pages may still be generated; SUMMARY stays package-only).
- **`summary` without `init`:** SUMMARY shape follows `layout=` — package links only for `package`; per-entity lines for `entity` / `split`.
- Repeat **`init`** on a customized book overwrites scaffold files the plugin emits (`book.toml`, theme, init `README.md`, `summary_path`, package markdown). Ongoing runs should omit `init`.

Init-only options: `title=`, `no_proto_highlight`, `ignore=git` (default) / `ignore=none`. See README plugin-options table for full list.

### Layout (`layout=`)

Paths below are under `{book_root}/{markdown_root}/` (default `markdown_root=src/packages`).

| Value | Default? | Page paths |
|-------|----------|------------|
| `package` | **yes** | `<package>.md` |
| `entity` | no | `<pkg>/messages\|enums\|services/<Name>.md` |
| `split` | no | `<pkg>/index.md` plus entity pages as in `entity` |

(`<pkg>` uses dots → slashes, e.g. `acme/example/v1`.)

### Entity bodies (all layouts)

Prefer **synthesized `protobuf` fences** (with source file path), not field/enum/RPC tables. RPC signatures are bold prose lines, not headings or tables.

## Markdown formatting

### Generated output (plugin)

- Proto comments come from `SourceCodeInfo` and are emitted **verbatim** by default (see
  `push_markdown_doc` in `src/render/proto_syntax.rs`) — no reflow, no injected hard line breaks
  inside emphasis or links. Optional `escape_tags` rewrites HTML-like `<…>` in leading-comment
  prose for mdBook; see the README generator options table.
- RPC signature lines are generated as single-line Markdown, e.g.
  `**EchoUnary** ( [EchoUnaryRequest](#…) ) returns ( … )` — keep type links on one line; do not
  split `[text](#anchor)` or `*…*` / `**…**` across lines when changing render code.

### Hand-written docs (README, AGENTS, init README)

- Optional: [`rumdl`](https://github.com/rvben/rumdl) if installed — `cargo xtask rumdl-fmt` /
  `cargo xtask rumdl-check` (currently scoped to root `README.md`; config in `.rumdl.toml`).
- When editing Markdown by hand, avoid reflow that breaks inline constructs:
  - do not insert line breaks inside `*italic*` or `**bold**` spans;
  - do not insert line breaks inside `[label](url)` link targets or labels.

Generated `./api-book/` pages are not rumdl-formatted in CI; validate with `cargo xtask book-links`.

## Hand-off

From the repository root:

```bash
cargo xtask ci   # includes buf-lint and fmt-check (proto format via buf format --diff)
```

Human spot-check (full mdBook at repo root):

```bash
cargo xtask book-init      # once: full mdBook at ./api-book
cargo xtask book-refresh   # after proto edits; loads paths from book.toml via book=
# Optional: --generator cli  (protobuf-mdbook + buf on examples/proto/)
cd api-book && mdbook serve
```

Guided `book-*` xtasks target `./api-book`. Default `--generator protoc`; `--generator cli`
runs `protobuf-mdbook` instead. `book-refresh` passes `book=` /
`mdbook_out=` so paths align with `[book] src` in `book.toml`.

## Tests

- Unit tests in `lib` / `render` / `options` / `init` / `book_config`.
- `tests/protoc_invocation.rs` — fixture markdown-only vs `init` for **both** `protoc-gen-mdbook`
  and `protobuf-mdbook --compiler protoc`; `--version` pins.
- `tests/protobuf_mdbook.rs` — CLI-only paths: `--descriptor-set`, buf module root without
  per-file filter.
- `tests/link_check.rs` — layout/options variants for **both** backends (`protobuf-mdbook`
  uses default buf on `examples/proto`; skipped when `buf` is not on PATH).
- Shared helpers: [`tests/common/mod.rs`](tests/common/mod.rs) (`mirrored_backends`,
  `mirrored_fixture_backends`, `run_examples`, `run_fixture`).
- Golden output regression: `tests/output_regression.rs`; refresh baselines with
  `cargo xtask update-golden`.
- Required gate: `cargo xtask book-links` (part of `ci`, after `book-init --markdown-only`).
- When iterating on markdown output: `cargo xtask book-init --markdown-only`, `cargo xtask book-links`.
- Local coverage: `cargo xtask coverage --open` (requires `cargo install cargo-llvm-cov --locked`
  and `rustup component add llvm-tools-preview`; not part of `ci`).

## Protoc plugin contract

- **Stock `protoc`** is the ground truth for the plugin binary: integration tests spawn
  [`protoc-bin-vendored`](https://docs.rs/protoc-bin-vendored) and pass
  `--plugin=protoc-gen-mdbook=…` / `--mdbook_out=…`.
- **Standalone CLI:** `protobuf-mdbook` (`src/bin/protobuf-mdbook.rs`) resolves inputs
  via [`src/input.rs`](src/input.rs) — default **`buf build`** on a Buf module,
  `--compiler protoc` for loose proto trees, or `--descriptor-set` for prebuilt FDS.
  Shares `generate_from_input()` / `write_generated_files()` with the plugin.
- **Lib + bin layout:** `generate()` in `src/lib.rs` decodes plugin requests;
  `src/main.rs` is stdin/stdout only; `protobuf-mdbook` writes to `-o`.
- **Vendored `protoc` caveat:** the bundled binary can fail on exotic hosts
  (some NixOS / musl-only setups). The CLI falls back to vendored protoc only when
  `--compiler protoc` is used and `protoc` is not on PATH.

## CI

- Local and GitHub Actions both run `cargo xtask ci` (see `.github/workflows/rust-tests.yml`).
- Toolchain: `dtolnay/rust-toolchain@1.95.0` with `components: rustfmt, clippy` (matches `rust-toolchain.toml`); `ci` runs
  `check-toolchain --strict` before buf lint, fmt-check, and clippy/test.
- Buf CLI 1.69.0: `cargo install buf-toolchain --locked --version 1.69.0` on CI; `ci` runs `buf-lint` and `fmt-check` (includes
  `buf format --diff` on `examples/proto/`).
- Matrix covers Linux, macOS, and Windows with `shell: bash`.
- **Docker:** `cargo xtask docker` builds the scratch image (`Dockerfile`) and runs
  runtime smoke checks (`--version`, non-root user, entrypoint). CI runs the same
  via the `docker` job (Buildx + `cargo xtask docker`).
