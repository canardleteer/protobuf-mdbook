# SyncCatalogRequest

SyncCatalogRequest is one frame in a bidirectional catalog sync session.

*`acme/example/v2/services.proto`*

```protobuf
message SyncCatalogRequest {
  uint64 sequence = 1;
  string catalog_revision = 2;
  PayloadEnvelope payload = 3;
  bool fin = 4;
}
```

