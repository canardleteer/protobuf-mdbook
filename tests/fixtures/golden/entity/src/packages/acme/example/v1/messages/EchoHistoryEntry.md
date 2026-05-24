# EchoHistoryEntry

EchoHistoryEntry is one row in a fictional audit trail.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoHistoryEntry {
  google.protobuf.Timestamp at = 1;
  string rpc_name = 2;
  EchoUnaryRequest unary_request = 3;
  acme.example.v2.ErrorDetail error = 4;
}
```

