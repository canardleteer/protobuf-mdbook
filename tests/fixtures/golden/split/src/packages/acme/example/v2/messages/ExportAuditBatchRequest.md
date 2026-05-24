# ExportAuditBatchRequest

ExportAuditBatchRequest exports a batch of audit rows (unary).

*`acme/example/v2/services.proto`*

```protobuf
message ExportAuditBatchRequest {
  ResourceIdentity identity = 1;
  TimeWindow window = 2;
  ListOptions options = 3;
}
```

