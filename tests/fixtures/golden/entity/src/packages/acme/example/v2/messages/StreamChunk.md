# StreamChunk

StreamChunk is a unit payload in streaming RPC fixtures.

*`acme/example/v2/types.proto`*

```protobuf
message StreamChunk {
  StreamCursor cursor = 1;
  PayloadEnvelope payload = 2;
  bool terminal = 3;
}
```

