# EchoServerStreamRequest

EchoServerStreamRequest opens a server-streaming RPC.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoServerStreamRequest {
  string message = 1;
  uint32 chunk_count = 2;
  google.protobuf.Duration inter_chunk_delay = 3;
  acme.example.v2.SharedMetadata metadata = 4;
}
```

