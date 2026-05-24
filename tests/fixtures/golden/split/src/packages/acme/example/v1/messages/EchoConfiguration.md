# EchoConfiguration

EchoConfiguration captures static config referenced in comments only.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoConfiguration {
  uint32 default_chunk_size = 1;
  google.protobuf.Duration default_timeout = 2;
  acme.example.v2.RetryPolicy retry_policy = 3;
}
```

