# ListEchoHistoryRequest

ListEchoHistoryRequest lists EchoHistoryEntry rows.

*`acme/example/v1/echo.proto`*

```protobuf
message ListEchoHistoryRequest {
  acme.example.v2.ListOptions options = 1;
  acme.example.v2.ResourceIdentity identity = 2;
}
```

