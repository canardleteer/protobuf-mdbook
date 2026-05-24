# SubjectRef

SubjectRef identifies a user or device for experiment assignment.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message SubjectRef {
  oneof identity {
      string user_id = 1 [(buf.validate.field).string.uuid = true];
      string device_id = 2 [(buf.validate.field).string.min_len = 8];
    }
  acme.example.v2.TenantRef tenant = 3;
}
```

