# Acme APIs

Overview of the Acme example protobuf modules used by the
[`protobuf-mdbook`](https://github.com/canardleteer/protobuf-mdbook/)
fixtures.

## Example output from protobuf-mdbook

- Mermaid diagrams appear only in `acme.example.v1` (flowchart and sequence
  diagrams in proto comments; no Mermaid in v2 or v3alpha1).
- Protovalidate with CEL highlight rendering covers message-level CEL rules in
  `acme.example.v2` (`NumericRange`) and `acme.example.v3alpha1`
  (`ExperimentSpec`, `PipelineRun`), plus field-level Protovalidate rules across v2
  and v3alpha1. v1 has no Protovalidate imports.
- CommonMark in comments includes tables, emphasis, and cross-package type links
  (notably in `acme.example.v1`).
- Companion markdown copies hand-written `.md` beside protos (for example module
  README and `DEV-NOTES`) flat into the book SUMMARY.
