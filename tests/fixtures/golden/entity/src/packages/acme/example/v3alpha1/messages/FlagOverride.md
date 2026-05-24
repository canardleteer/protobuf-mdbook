# FlagOverride

FlagOverride pins a flag value for one tenant.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message FlagOverride {
  acme.example.v2.TenantRef tenant = 1;
  string flag_key = 2 [(buf.validate.field).string.min_len = 1];
  bool enabled = 3;
  google.protobuf.Timestamp expires_at = 4;
}
```

