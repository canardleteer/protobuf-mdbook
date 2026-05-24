# EchoClientStreamRequest

EchoClientStreamRequest is one chunk in a client stream upload.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoClientStreamRequest {
  string message = 1;
  uint32 part_index = 2;
  bool last_part = 3;
}
```

