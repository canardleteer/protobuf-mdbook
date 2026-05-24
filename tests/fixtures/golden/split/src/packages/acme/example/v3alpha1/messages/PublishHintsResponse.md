# PublishHintsResponse

PublishHintsResponse aggregates uploaded hint parts.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message PublishHintsResponse {
  string batch_id = 1;
  uint32 parts_received = 2;
  acme.example.v2.PageResult page = 3;
}
```

