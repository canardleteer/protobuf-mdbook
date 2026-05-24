# EchoServerStreamResponse

EchoServerStreamResponse is one chunk in a server stream.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoServerStreamResponse {
  string message = 1;
  acme.example.v2.StreamCursor cursor = 2;
  uint32 index = 3;
}
```

