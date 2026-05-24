# SyncFlagsResponse

SyncFlagsResponse mirrors a bidirectional flag sync frame.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message SyncFlagsResponse {
  uint64 sequence = 1;
  acme.example.v2.StreamCursor cursor = 2;
  acme.example.v2.ErrorDetail error = 3;
  bool fin = 4;
}
```

