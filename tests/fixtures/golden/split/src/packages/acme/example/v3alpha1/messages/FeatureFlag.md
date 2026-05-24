# FeatureFlag

FeatureFlag describes a toggle evaluated by fictional clients.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message FeatureFlag {
// Stable identifier (slug).
  string key = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 128,
      (buf.validate.field).string.pattern = "^[a-z][a-z0-9_.-]*$"
    ];
  string display_name = 2 [(buf.validate.field).string.max_len = 256];
  string description = 3;
  bool default_enabled = 4;
  ReleaseChannel channel = 5;
  acme.example.v2.SharedMetadata metadata = 6;
  google.protobuf.Timestamp updated_at = 7;
}
```

