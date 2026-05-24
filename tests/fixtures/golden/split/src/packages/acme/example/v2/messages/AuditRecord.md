# AuditRecord

AuditRecord captures a single synthetic audit line.

*`acme/example/v2/types.proto`*

```protobuf
message AuditRecord {
  string actor = 1;
  string action = 2;
  google.protobuf.Timestamp occurred_at = 3;
  google.protobuf.Struct details = 4;
}
```

