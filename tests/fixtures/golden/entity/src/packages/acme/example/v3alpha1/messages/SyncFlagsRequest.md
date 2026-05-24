# SyncFlagsRequest

SyncFlagsRequest is one frame in a bidirectional flag sync session.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message SyncFlagsRequest {
  uint64 sequence = 1;
  FeatureFlag flag = 2;
  bool fin = 3;
}
```

