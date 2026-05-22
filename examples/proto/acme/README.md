# Acme APIs

Overview of the Acme example protobuf modules used by `protoc-gen-mdbook`
fixtures.

## Example output from protoc-gen-mdbook

- **Mermaid diagrams** — only in `acme.example.v1` (flowchart and sequence
  diagrams in proto comments; no Mermaid in v2 or v3alpha1).
- **Protovalidate with CEL highlight rendering** — message-level CEL rules in
  `acme.example.v2` (`NumericRange`) and `acme.example.v3alpha1`
  (`ExperimentSpec`, `PipelineRun`); field-level Protovalidate rules across v2
  and v3alpha1. v1 has no Protovalidate imports.
- **CommonMark in comments** — tables, emphasis, and cross-package type links
  (notably in `acme.example.v1`).
- **Companion markdown** — hand-written `.md` beside protos (for example module
  README and `DEV-NOTES`) copied flat into the book SUMMARY.
