# EchoBidiStreamResponse

EchoBidiStreamResponse mirrors a bidirectional frame back to the client.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoBidiStreamResponse {
  string message = 1;
  uint64 sequence = 2;
  bool fin = 3;
  acme.example.v2.StreamCursor cursor = 4;
}
```

