# EchoUnaryResponse

EchoUnaryResponse returns the echoed text.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoUnaryResponse {
  string message = 1;
  google.protobuf.Timestamp echoed_at = 2;
  acme.example.v2.SharedMetadata metadata = 3;
}
```

