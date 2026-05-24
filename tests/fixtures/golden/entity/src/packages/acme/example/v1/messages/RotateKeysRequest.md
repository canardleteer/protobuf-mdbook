# RotateKeysRequest

RotateKeysRequest triggers a long-running operation example.

*`acme/example/v1/gateway.proto`*

```protobuf
message RotateKeysRequest {
  acme.example.v2.TenantRef tenant = 1;
  bool dry_run = 2;
}
```

