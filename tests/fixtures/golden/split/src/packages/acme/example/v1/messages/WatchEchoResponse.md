# WatchEchoResponse

WatchEchoResponse is pushed on the WatchEcho server stream.

*`acme/example/v1/echo.proto`*

```protobuf
message WatchEchoResponse {
  string event_id = 1;
  EchoUnaryResponse payload = 2;
  google.protobuf.Timestamp observed_at = 3;
}
```

