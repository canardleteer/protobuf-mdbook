# StreamAuditRecordsResponse

StreamAuditRecordsResponse is one audit row on the stream.

*`acme/example/v2/services.proto`*

```protobuf
message StreamAuditRecordsResponse {
  AuditRecord record = 1;
  StreamCursor cursor = 2;
}
```

