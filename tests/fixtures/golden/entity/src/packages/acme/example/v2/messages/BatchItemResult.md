# BatchItemResult

BatchItemResult reports per-item outcomes.

*`acme/example/v2/types.proto`*

```protobuf
message BatchItemResult {
  BatchKey key = 1;
  bool ok = 2;
  ErrorDetail error = 3;
  PayloadEnvelope response_envelope = 4;
}
```

