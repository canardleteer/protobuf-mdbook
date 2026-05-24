# StreamAuditRecordsRequest

StreamAuditRecordsRequest opens a server stream of audit rows.

*`acme/example/v2/services.proto`*

```protobuf
message StreamAuditRecordsRequest {
  TenantRef tenant = 1;
  repeated FilterExpression filters = 2;
}
```

