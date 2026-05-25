# protobuf-mdbook

> [!WARNING]
> Clanker generated code, running an auto-release pipeline.
>
> Decide if that degree of automation is appropriate for your requirements.

- [Example Output: `examples/proto`](https://canardleteer.github.io/protobuf-mdbook/)
- [Developer Documentation](#development)

## Features

- `.proto` → `mdBook` documentation
- `buf` and `protoc` support
- Build-time syntax highlighting
- Markdown generation layout flexibility
- Companion Markdown file support

## Tools

**protobuf-mdbook** and **protoc-gen-mdbook** share one generator: they turn
protobuf schemas and comments into **mdBook** documentation (or Markdown-only
trees). **`mdbook-protobuf-highlight`** is a separate mdBook preprocessor that
highlights `protobuf` / `cel` fences at **`mdbook build`** time.

| Binary | Role | Docs |
|--------|------|------|
| **`protobuf-mdbook`** | Standalone CLI — writes files to disk (`-o` / `--output`) | [Standalone CLI](#standalone-cli-protobuf-mdbook) |
| **`protoc-gen-mdbook`** | **protoc** plugin — reads `CodeGeneratorRequest` on stdin, writes `CodeGeneratorResponse` on stdout | [Protoc plugin](#protoc-plugin-protoc-gen-mdbook) |
| **`mdbook-protobuf-highlight`** | **mdBook** preprocessor — highlights `protobuf` / `cel` fences at build time; `install` patches `book.toml` | [Syntax highlighting](#syntax-highlighting) |

The pinned **[mdBook](https://rust-lang.github.io/mdBook/)** release is declared
in root [`Cargo.toml`](Cargo.toml) (`mdbook-core` / `mdbook-driver` /
`mdbook-preprocessor`). All three binaries report the exact pin at runtime and
share the crate version:

```bash
protobuf-mdbook --version           # or -V (includes mdbook pin)
protoc-gen-mdbook --version         # or -V (includes mdbook pin)
mdbook-protobuf-highlight --version # or -V (includes mdbook pin)
```

## Install

### Users

Install three commands from crates.io (one crate, three binaries):

```bash
cargo install protobuf-mdbook
```

Installs:

- `protobuf-mdbook` — standalone CLI (`buf` or `protoc` input).
- `protoc-gen-mdbook` — `protoc` plugin.
- `mdbook-protobuf-highlight` — mdBook preprocessor for build-time
  protobuf / CEL highlighting.

That puts all three on your PATH (typically `~/.cargo/bin`). Install one binary
only with `--bin protoc-gen-mdbook`, `--bin protobuf-mdbook`, or
`--bin mdbook-protobuf-highlight`.

#### `protobuf-mdbook`

- **`buf` compiler:** **`protobuf-mdbook`** by default uses
  [buf](https://github.com/bufbuild/buf), which is easy to install with
  [`buf-toolchain`](https://github.com/canardleteer/buf-rs):
  - `cargo install buf-toolchain`
    ([repo](https://github.com/canardleteer/buf-rs)).
- **`protoc` compiler:** available as well, but you're on your own for
  installing it.

#### `protoc-gen-mdbook`

`protoc` adaptation of `protobuf-mdbook`.

#### `mdbook-protobuf-highlight`

mdBook invokes this preprocessor during **`mdbook build`** / **`mdbook serve`**
(stdin/stdout book JSON). It does not read `.proto` files — generation stays in
**`protobuf-mdbook`** / **`protoc-gen-mdbook`**.

- **`init`** wires `[preprocessor.protobuf-highlight]` in `book.toml` by
  default.
- Existing books: **`mdbook-protobuf-highlight install [book-root]`** then
  **`mdbook build`**.
- Requires **`mdbook-protobuf-highlight`** on PATH (same `cargo install` as the
  other binaries).

See [Syntax highlighting](#syntax-highlighting) for toggles, theme CSS, and
maintainer workflow.

## Standalone CLI (`protobuf-mdbook`)

Use **`protobuf-mdbook`** when you want the same output without wiring a
`protoc` plugin.

### Buf module (default)

Point at a directory containing `buf.yaml`; BSR deps resolve automatically:

```bash
# scaffold mdBook once (under the hood, this calls `mdbook init` via API integration)
protobuf-mdbook -o ./api-book --init examples/proto

# update only the generated markdown, without re-initializing the mdBook scaffolding.
protobuf-mdbook -o ./api-book examples/proto
```

### Loose `.proto` tree (`--compiler protoc`)

If there is no `buf.yaml`, opt into `protoc` (PATH, then vendored `protoc`
bundled in this crate):

```bash
protobuf-mdbook --compiler protoc -o ./out -I tests/fixtures tests/fixtures/doc_rich.proto
```

### Prebuilt descriptor set (no compiler)

**`protobuf-mdbook --descriptor-set`** reads a binary
`google.protobuf.FileDescriptorSet` (`.binpb`, `.fds`, or any path — the
extension is just a convention).

#### `buf`

Via: `buf build -o ./descriptors.binpb`.

Generator options are listed in [Generator options](#generator-options) below.
Run **`protobuf-mdbook --version`** for the pinned mdBook release.

#### `protoc`

Produce one with **`protoc --descriptor_set_out`** (include source info so
comments and fenced blocks survive):

```bash
# Compile a binpb
protoc -I tests/fixtures \
  --descriptor_set_out=./descriptors.binpb \
  --include_imports --include_source_info \
  tests/fixtures/doc_rich.proto

# generate mdbook ready documents
protobuf-mdbook -o ./out --descriptor-set ./descriptors.binpb
```

## Protoc plugin (`protoc-gen-mdbook`)

This walkthrough uses the example protos in [`examples/proto/`](examples/proto/)
and writes a local book at `./api-book` (gitignored).

```bash
# Build the plugin and confirm versions (mdbook pin is in --version output).
cargo build
cargo run -p protobuf-mdbook --bin protoc-gen-mdbook -- --version

PLUGIN=target/debug/protoc-gen-mdbook   # or target/release/...

# ONE-TIME: scaffold a local mdBook at ./api-book from this repo's example protos.
# Do not repeat `init` on a book you have already customized — it replaces the scaffold.
(cd examples/proto && buf export . --output ../../target/proto-deps)
protoc -I examples/proto -I target/proto-deps \
  --plugin=protoc-gen-mdbook="$PLUGIN" \
  --mdbook_out=./api-book \
  --mdbook_opt=init,layout=package \
  examples/proto/acme/example/{v1,v2,v3alpha1}/*.proto

cd api-book
# Read ./README.md (generated beside book.toml): next steps, mermaid, rumdl, lychee.
# Edit book.toml, SUMMARY, themes, and preprocessors to taste.
# If you relocate API pages or SUMMARY, note the paths — see ONGOING below.
mdbook build    # static HTML smoke test
mdbook serve    # live preview while editing
cd ..

# ONGOING: regenerate API markdown only.
# Pass `book=` to load `[book] src` from book.toml (via mdbook-core); paths default to
# `{src}/packages` and `{src}/SUMMARY.md`. Does not touch book.toml, SUMMARY, theme,
# or api-book/README.md unless you pass summary or init.
protoc -I examples/proto -I target/proto-deps \
  --plugin=protoc-gen-mdbook="$PLUGIN" \
  --mdbook_out=./api-book \
  --mdbook_opt=layout=package,book=./api-book,mdbook_out=./api-book \
  examples/proto/acme/example/{v1,v2,v3alpha1}/*.proto

# Override inferred paths explicitly when needed (omit summary unless regenerating nav):
#
#   --mdbook_opt=layout=package,book=./api-book,mdbook_out=./api-book,markdown_root=content/api
```

Use an **[mdBook](https://rust-lang.github.io/mdBook/)** CLI whose major.minor
matches either binary’s **`--version`** output when building or serving locally.
List every `.proto` you want documented (your shell expands globs like
`{v1,v2,v3alpha1}/*.proto` —
[`protoc`](https://protobuf.dev/reference/cpp/api-docs/#command-line-interface)
itself does not), or use a discovery script / **`cargo xtask book-refresh`** for
this repo’s examples.

The same **`init`** / ongoing refresh flow works with **`protobuf-mdbook`** —
see [Standalone CLI](#standalone-cli-protobuf-mdbook) — using **`-o` /
`--output`** and native flags such as **`--init`**, **`--layout`**, and
**`--book`** instead of **`--mdbook_out`** / **`--mdbook_opt`**.

## Output modes

| Mode | Option | Emits |
|------|--------|-------|
| Markdown only (default) | *(none)* | `{markdown_root}/**/*.md` only (default `markdown_root=src/packages`) |
| Markdown + nav file | `summary` | Default output plus `summary_path` (default `src/SUMMARY.md`) |
| mdBook project | `init` | Full mdBook tree, package-only SUMMARY, and `README.md` beside `book.toml` |

### Where files land

**Protoc plugin:** protoc writes each generated path relative to
**`--mdbook_out`** only. Each file is `{book_root}/…` under that root (default
`book_root=.`).

**Standalone CLI:** **`protobuf-mdbook`** writes the same relative paths under
**`-o` / `--output`** (equivalent to **`--mdbook_out`**).

With default `[book] src`, package pages are `{src}/packages/<package>.md` (e.g.
`api-book/src/packages/acme.example.v1.md`).

Pass **`book=`** (book root directory or path to `book.toml`) to load
`[book] src` via **[mdbook-core](https://docs.rs/mdbook-core)** and infer
`markdown_root={src}/packages` and `summary_path={src}/SUMMARY.md`. Explicit
`markdown_root=` / `summary_path=` still override. Pair with **`mdbook_out=`**
in options when validating that the output root matches the book root (plugin:
**`--mdbook_opt`**; CLI: **`-o` / `--output`** with **`--book`**).

**Default refresh** (no `init`, no `summary`) updates only package markdown
under `markdown_root`; it does not rewrite `book.toml`, theme, init `README.md`,
or SUMMARY unless you opt in.

With `init`, generated package pages live under `{markdown_root}/`. `init`’s
placeholder `chapter_1.md` is not included. The default theme is copied into the
output tree.

By default, `init` also wires the **mdbook-protobuf-highlight** preprocessor in
`book.toml` (see [Syntax highlighting](#syntax-highlighting)).

Entity bodies use **protobuf source-style fenced blocks** (with file paths), not
field/enum tables. Message-level
[Protovalidate](https://buf.build/bufbuild/protovalidate) CEL rules are split
into adjacent `cel` fenced blocks when present in source.

## Generator options

Both binaries share the same semantics; spelling differs by surface:

- **`protobuf-mdbook`:** native clap flags with hyphens (`--layout entity`,
  `--no-proto-highlight`); repeatable path flags where noted (`-I` /
  `--proto-path`).
- **`protoc-gen-mdbook`:** comma-separated on **`--mdbook_opt=…`** (or
  `CodeGeneratorRequest.parameter`); underscore tokens (`layout=entity`,
  `no_proto_highlight`).

| `protobuf-mdbook` | Protoc (`--mdbook_opt`) | Applies to | Purpose |
|-------------------|-------------------------|------------|---------|
| `--init` | `init` | init | mdBook scaffold + package `SUMMARY` + `README.md` |
| `--summary` | `summary` | default | Optional SUMMARY without mdBook tree |
| `--layout package` \| `entity` \| `split` | `layout=package` \| `entity` \| `split` | both | Doc rollup (default `package`) |
| `--book-root <path>` | `book_root=<path>` | both | Subdirectory under output root (default `.`) |
| `--book <path>` | `book=<path>` | refresh | Book root or `book.toml`; loads `[book] src` via mdbook-core |
| `-o` / `--output <path>` | `mdbook_out=<path>` | refresh | Validate output root matches `book=` (warn if divergent) |
| `--markdown-root <path>` | `markdown_root=<path>` | both | API markdown directory (default `src/packages`, or `{src}/packages` with `book=`) |
| `--summary-path <path>` | `summary_path=<path>` | both | SUMMARY when `summary`/`init` (default `src/SUMMARY.md`, or `{src}/SUMMARY.md` with `book=`) |
| `-I` / `--proto-path <dir>` (repeatable) | `proto_path=<dir>` \| `dir:a:dir:b` | both | Search path(s) for `.proto` sources |
| `--title <text>` | `title=<text>` | init only | `book.toml` title (default **Protobuf documentation** if omitted) |
| `--ignore git` \| `none` | `ignore=git` \| `ignore=none` | init only | Whether init emits `.gitignore` (default `git`) |
| `--no-proto-highlight` | `no_proto_highlight` | init only | Set `protobuf = false` in preprocessor config (default: on) |
| `--no-cel-highlight` | `no_cel_highlight` | init only | Set `cel = false` in preprocessor config (default: on; independent of protobuf) |
| `--no-proto-markdown` | `no_proto_markdown` | both | Disable copying companion `.md` beside protos and companion entries in SUMMARY |
| `--escape-tags` \| `--escape-tags entities` | `escape_tags` \| `escape_tags=backticks` \| `escape_tags=entities` | both | Rewrite HTML-like `<…>` in leading-comment prose so mdBook does not treat them as HTML tags (bare flag: inline code; `entities` uses `&lt;…&gt;`) |
| — | `markdown_only` | — | Deprecated (ignored); default output is already markdown-only |

`force` is always implied (non-interactive protoc).

## Documentation layout (`layout=`)

Paths are under `{book_root}/{markdown_root}/` (defaults: `book_root=.`,
`markdown_root=src/packages`).

| Value | Output |
|-------|--------|
| `package` (default) | `<package>.md` — one page per package |
| `entity` | `<pkg>/messages\|enums\|services/<Name>.md` |
| `split` | Package `index.md` plus entity pages as in `entity` |

(`<pkg>` uses dots → slashes, e.g. `acme/example/v1`.)

Comments come from `SourceCodeInfo` and are copied **verbatim** (no
`@exclude` or protoc-gen-doc directives). Field and RPC types link to other
documented entities when those types are in `file_to_generate`.

## Syntax highlighting

Generated API pages use ` ```protobuf ` fences; Protovalidate message-level CEL
rules also emit adjacent ` ```cel ` blocks at generation time. At
**`mdbook build`** time, **`mdbook-protobuf-highlight`** converts those
fences into pre-highlighted HTML (`<pre class="protobuf-mdbook …">`) compatible with
mdBook’s bundled
[`highlight.css`](https://rust-lang.github.io/mdBook/format/theme/syntax-highlighting.html).

### Init (default on)

With `init`, the generator wires `[preprocessor.protobuf-highlight]` in
`book.toml` (`command = "mdbook-protobuf-highlight"`). Requires
`mdbook-protobuf-highlight` on PATH when you run `mdbook build` / `mdbook serve`
(included in `cargo install protobuf-mdbook`).

| Init flag | Effect |
|-----------|--------|
| *(default)* | `protobuf = true`, `cel = true` in preprocessor config |
| `no_proto_highlight` | `protobuf = false`, `cel = true` |
| `no_cel_highlight` | `protobuf = true`, `cel = false` |
| both flags | No preprocessor section (highlighting disabled at init) |

### Standalone install

For books not created via `init`:

```bash
mdbook-protobuf-highlight install [book-root]
mdbook build
```

Toggle languages in `book.toml`:

```toml
[preprocessor.protobuf-highlight]
command = "mdbook-protobuf-highlight"
protobuf = true
cel = true
```

Custom themes need no `index.hbs` patches — highlighting is build-time HTML, not
client-side Highlight.js registration.

### Protobuf and CEL behavior

- **Protobuf / proto fences:** highlight proto body; split message-level
  `(buf.validate.message).cel` into separate highlighted CEL blocks (same rules
  as generation-time `cel_fence`).
- **CEL fences:** highlight directly (hand-written companion markdown).
- **Field-level** `[(buf.validate.field).cel …]` stays inside protobuf blocks.

Grammar rules are ported to Rust in [`src/highlight/`](src/highlight/) from
reference files in [`assets/highlightjs/`](assets/highlightjs/) (protobuf:
BSD-3-Clause upstream HLJS 10.1.1; CEL: repo-authored). CI proves output via
golden HTML fixtures (`cargo xtask check-highlight-rust`).

### Maintainer workflow

After intentional grammar changes:

```bash
cargo xtask update-highlight-golden
cargo xtask check-highlight-rust
```

### Syntax grammar theme customization

Highlighting emits `<pre class="protobuf-mdbook language-protobuf">` or
`language-cel` with inner `hljs-*` spans. Init installs
`theme/protobuf-highlight.css` (layout only); token colors come from mdBook’s
`highlight.css`. Scope overrides with `pre.language-protobuf` /
`pre.language-cel` (block) or `pre.language-cel .hljs-string`(tokens).

New token classes require editing [`src/highlight/`](src/highlight/).

```toml
[output.html]
additional-css = [
  "theme/protobuf-highlight.css",
  "theme/my-protobuf-theme.css",
]
```

```css
/* theme/my-protobuf-theme.css — proto block typography */
pre.language-protobuf {
  font-family: var(--mono-font);
  color: var(--fg);
}

/* CEL-only token colors */
pre.language-cel .hljs-title { color: #9cdcfe; }
pre.language-cel .hljs-string { color: #ce9178; }
```

### Limitations

**CEL extraction** uses brace-depth scanning, not a full protobuf/CEL lexer. If
an `expression:` string literal contains `};` sequences, the split may truncate
or mis-bound the block. Prefer simple expressions in generated docs, or
hand-written `cel` fenced blocks in proto module READMEs for complex rules.

## Companion markdown

By default, hand-written `.md` files on the **ancestor path** of each included
`.proto` (from the import root through the proto’s directory) are copied into
`{markdown_root}/` using flat names: `dir.segments.<stem>.md` (for example
`acme/example/v1/README.md` → `{markdown_root}/acme.example.v1.README.md`).
Content is copied verbatim; the generator does not synthesize module README
bodies. Opt out with `no_proto_markdown`.

SUMMARY/chapter order follows **directory layout**, not protobuf `import`
relationships. Generated `SUMMARY.md` is a **starting point** — edit nav after
generation runs. With `summary` or `init`, companions are wired using **minimal
subchaptering** (pass-through directories without their own `.md` are collapsed;
very deep trees may flatten). Section companions use `{module.path} - {title}`
(dot-separated paths from output filenames); nested subchapters and generated
package pages use bare titles.

```text
proto/                              # protoc -I root or protobuf-mdbook input root
└── acme/
    ├── README.md                   # → src/packages/acme.README.md
    └── example/                    # intermediate — OK to add .md here
        ├── README.md               # → src/packages/acme.example.README.md
        ├── v1/
        │   ├── README.md           # → src/packages/acme.example.v1.README.md
        │   ├── MOVING-TO-V2.md     # → src/packages/acme.example.v1.MOVING-TO-V2.md
        │   └── *.proto             # → src/packages/acme.example.v1.md
        └── v2/
            ├── README.md           # → src/packages/acme.example.v2.README.md
            └── *.proto             # → src/packages/acme.example.v2.md
```

Use `proto_path=<dir>` (or `dir:a:dir:b`) when `.proto` paths need extra search
roots for discovery. Only `.md` on ancestor chains of **included** protos are
copied.

## Development

Build from source (Rust toolchain: see
[`rust-toolchain.toml`](rust-toolchain.toml)):

```bash
cargo build --release
```

Binaries: `target/release/protobuf-mdbook`, `target/release/protoc-gen-mdbook`,
and `target/release/mdbook-protobuf-highlight`.

### xtasks

From the **repository root**:

| Command | Purpose |
|---------|---------|
| `cargo xtask ci` | check-toolchain, `buf-lint`, `fmt-check`, clippy, test, build-plugin, `check-highlight-rust`, `book-init --markdown-only`, `book-links` |
| `cargo xtask fmt` | `cargo fmt` + `buf format -w` on `examples/proto/` |
| `cargo xtask fmt-check` | `cargo fmt --check` + `buf format --diff` on `examples/proto/` |
| `cargo xtask buf-lint` | `buf lint` on `examples/proto/` (needs [Buf CLI](https://buf.build/docs/cli/installation/); see [`buf.lock`](examples/proto/buf.lock)) |
| `cargo xtask buf-format` | `buf format -w` on `examples/proto/` only |
| `cargo xtask buf-format-check` | `buf format --diff` on `examples/proto/` only |
| `cargo xtask check-toolchain` | Warn if active rustc/components diverge from `rust-toolchain.toml` (`--strict` to fail) |
| `cargo xtask book-init` | Full mdBook scaffold at `./api-book` (wipes first; run once locally) |
| `cargo xtask book-init --markdown-only` | Markdown only → `./api-book` (wipes first; what CI uses for link checks) |
| `cargo xtask book-refresh` | Refresh `./api-book` markdown; passes `book=` to load paths from `book.toml` |
| `cargo xtask book-links` | Resolve in-page links and mdBook heading anchors in `./api-book/` |
| `cargo xtask book-build` | `mdbook build` on `./api-book/` |
| `cargo xtask coverage --open` | LLVM HTML coverage (needs [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov), `llvm-tools-preview`) |
| `cargo xtask rumdl-fmt` | Format this crate `README.md` (needs `rumdl` on PATH) |
| `cargo xtask rumdl-check` | Lint root `README.md` with rumdl |
| `cargo xtask docker` | Build linux/amd64 scratch image + runtime smoke tests |
| `cargo xtask check-highlight-rust` | Verify Rust highlighter output vs golden HTML in `tests/fixtures/highlight/` |
| `cargo xtask update-highlight-golden` | Refresh highlight golden HTML after grammar changes |

**Guided `book-init` / `book-refresh`:** default **`--generator protoc`**
(`protoc` + **`protoc-gen-mdbook`**; matches CI). Use **`--generator cli`** for
**`protobuf-mdbook`** on `examples/proto/` (needs Buf on PATH):

```bash
cargo xtask book-init --generator cli
cargo xtask book-refresh --generator cli
```

**Local preview:** `cargo xtask book-init`, then `cd api-book && mdbook serve`
(or `cargo xtask book-build`).

### Example protos

Example protos: [`examples/proto/`](examples/proto/) (Buf module; Protovalidate
via BSR dep in [`buf.yaml`](examples/proto/buf.yaml)). **`--generator protoc`**
/ tests run `buf export` to `target/proto-deps/` for `protoc -I` — do not pass
exported `.proto` files as protoc inputs. Generated book: `./api-book/`
(gitignored).

**Canonical protoc inputs:** [`src/examples.rs`](src/examples.rs) defines
`protobuf_mdbook::examples::EXAMPLE_PROTO_INPUTS` — the eight `acme/example/…`
files used by **`cargo xtask book-*`**, integration tests, and link-check runs.
That list matches the shell globs in the protoc walkthrough above and **excludes**
`buf/validate/validate.proto` (import-only via `buf export`). When you add a
new fixture `.proto` under `acme/`, append its path relative to `examples/proto/`
to `EXAMPLE_PROTO_INPUTS`. **`protobuf-mdbook examples/proto`** (Buf module root)
still discovers module protos via Buf; only the explicit protoc / xtask / test paths
use the shared list.

### Contributing

Contributor details: [`AGENTS.md`](AGENTS.md).

## Docker (linux/amd64)

Container **runtime** images are `scratch` + the static **`protoc-gen-mdbook`**
binary only (non-root `nobody` user). The [`Dockerfile`](Dockerfile) uses a
**`buf-anchor`** stage (`cargo install buf-toolchain --locked --version
1.69.0`), copies `buf` into the musl builder, runs `buf --version`, then
compiles the plugin.

From the repository root:

```bash
cargo xtask docker
```

That builds `protobuf-mdbook:local` for `linux/amd64` and smoke-tests
`--version`, image user, entrypoint, and platform. Plain build:

```bash
docker build --platform linux/amd64 -t protobuf-mdbook:local -f Dockerfile .
docker run --rm --entrypoint /protoc-gen-mdbook protobuf-mdbook:local --version
```

Proto comments may include fenced blocks such as `mermaid`; those pass through
into generated Markdown unchanged. Diagram rendering and `book.toml`
preprocessor setup are left to you (see
[mdbook-mermaid](https://github.com/badboy/mdbook-mermaid)).

## Similar projects

Other tools that generate mdBook documentation from protobuf:

- [**mdbook-protobuf**](https://github.com/zakhenry/mdbook-protobuf) — mdBook
  **preprocessor** that builds reference docs from a `FileDescriptorSet` on disk
  (configure `proto_descriptor` in `book.toml`; runs during `mdbook build` /
  `mdbook serve`).
- [**protoc-gen-mdbook**](https://github.com/matze/protoc-gen-mdbook) (matze) —
  earlier **protoc plugin** with a similar name; generates mdBook pages from
  `.proto` files via `protoc --mdbook_out`.
