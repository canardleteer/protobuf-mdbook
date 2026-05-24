# PublishHintsRequest

PublishHintsRequest is one client-streaming platform hint chunk.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message PublishHintsRequest {
  string batch_id = 1;
  PlatformHint hint = 2;
  uint32 part_index = 3;
  bool last_part = 4;
}
```

