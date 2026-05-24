# PayloadEnvelope

PayloadEnvelope wraps opaque bytes with metadata.

*`acme/example/v2/types.proto`*

```protobuf
message PayloadEnvelope {
  PayloadHeader header = 1;
  bytes body = 2;
  SharedMetadata metadata = 3;
}
```

