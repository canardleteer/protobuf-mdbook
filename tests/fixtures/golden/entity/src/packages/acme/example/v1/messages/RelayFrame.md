# RelayFrame

RelayFrame is one unit in a bidirectional relay stream.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayFrame {
  uint64 sequence = 1;
  bytes payload = 2;
  acme.example.v2.SharedMetadata metadata = 3;
  bool fin = 4;
}
```

