# ListEchoHistoryResponse

ListEchoHistoryResponse returns history rows.

*`acme/example/v1/echo.proto`*

```protobuf
message ListEchoHistoryResponse {
  repeated EchoHistoryEntry entries = 1;
  acme.example.v2.PageResult page = 2;
}
```

