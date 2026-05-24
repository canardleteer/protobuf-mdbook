# EchoBidiStreamRequest

EchoBidiStreamRequest is one frame in a bidirectional echo session.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoBidiStreamRequest {
  string message = 1;
  uint64 sequence = 2;
  bool fin = 3;
}
```

