# TenantRef

TenantRef identifies a fictional tenant for multi-tenant stories.

*`acme/example/v2/types.proto`*

```protobuf
message TenantRef {
  string tenant_id = 1 [(buf.validate.field).string.uuid = true];
  string slug = 2 [
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 64,
      (buf.validate.field).string.pattern = "^[a-z0-9-]+$"
    ];
  SharedKind tier = 3;
}
```

