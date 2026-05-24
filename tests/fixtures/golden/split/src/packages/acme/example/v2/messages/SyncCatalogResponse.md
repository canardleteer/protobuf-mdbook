# SyncCatalogResponse

SyncCatalogResponse mirrors a bidirectional catalog sync frame.

*`acme/example/v2/services.proto`*

```protobuf
message SyncCatalogResponse {
  uint64 sequence = 1;
  StreamCursor cursor = 2;
  ErrorDetail error = 3;
  bool fin = 4;
}
```

