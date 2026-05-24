# AuditBatch

AuditBatch groups records for batch RPC demonstrations.

*`acme/example/v2/types.proto`*

```protobuf
message AuditBatch {
  repeated AuditRecord records = 1;
  string batch_id = 2;
}
```

