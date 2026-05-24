# BatchEchoRequest

BatchEchoRequest wraps a v2 batch payload for STANDARD RPC naming.

*`acme/example/v1/echo.proto`*

```protobuf
message BatchEchoRequest {
  acme.example.v2.BatchRequest batch = 1;
}
```

