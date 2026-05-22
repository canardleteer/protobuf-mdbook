# Example protos (`examples/proto`)

Test fixtures for **protoc-gen-mdbook**. The book under `./api-book/` (and the
GitHub Pages site published from `main`) is generated from these `.proto` files
and companion markdown — not a real product API.

## Buf module

Buf module for fixture packages under `acme/example/{v1,v2,v3alpha1}/`.

- **`buf.yaml`** — `STANDARD` lint, `FILE` breaking, BSR dep
  [`buf.build/bufbuild/protovalidate`](https://buf.build/bufbuild/protovalidate) for
  `buf/validate/validate.proto` imports.
- **`buf.lock`** — pinned dep commits (regenerate with `buf dep update` here).

CI runs `buf lint` and `buf format --diff` (via `cargo xtask fmt-check`). Format locally with
`cargo xtask fmt` (`buf format -w` on this directory). For raw
`protoc`, export deps first (xtask does this automatically):

```bash
cd examples/proto
buf export . --output ../../target/proto-deps
protoc -I . -I ../../target/proto-deps …
```
