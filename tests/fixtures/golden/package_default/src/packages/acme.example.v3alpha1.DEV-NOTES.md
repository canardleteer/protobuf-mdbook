# Developer notes — v3alpha1 fixtures

Internal notes for maintainers of `protobuf-mdbook` example protos. **Lorem
ipsum** filler below is intentional stress text for mdBook preprocessors.

## Conventions

- Package id: `acme.example.v3alpha1` (alpha suffix in the protobuf package name).
- Cross-links: prefer `acme.example.v2` shared types; avoid new Mermaid here
  (diagrams remain in `acme.example.v1` only).
- Validation: annotate request-shaped messages with
  [Protovalidate](https://protovalidate.com/) rules; keep rules valid under `buf lint` (BSR dep in
  `examples/proto/buf.yaml`).

## protovalidate smoke targets

| Message | File | Rules exercised |
|---------|------|-----------------|
| `FeatureFlag` | `types.proto` | `string.min_len`, `string.pattern` |
| `AssignExperimentRequest` | `services.proto` | `repeated.min_items`, field `required` |
| `PipelineRun` | `pipeline.proto` | message-level `cel` |

## Escaping and inline HTML

Use `<kbd>Ctrl</kbd>+<kbd>S</kbd> in prose sparingly. Escaped asterisks: \*not
italic\*.

## Fenced protobuf (documentation only)

```protobuf
syntax = "proto3";
package acme.example.v3alpha1;
message Example { string id = 1; }
```

## Reference links

- [Protovalidate](https://protovalidate.com/)
- [mdBook preprocessors](https://rust-lang.github.io/mdBook/format/configuration.html)

---

*Last updated: fictional — rotate lorem when touching fixtures.*
