# BatchItem

BatchItem pairs a key with an envelope for batch APIs.

*`acme/example/v2/types.proto`*

```protobuf
message BatchItem {
  BatchKey key = 1;
  PayloadEnvelope envelope = 2;
}
```

