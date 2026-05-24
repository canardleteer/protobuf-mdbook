# ListFeatureFlagsRequest

ListFeatureFlagsRequest paginates feature flags for a tenant.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message ListFeatureFlagsRequest {
  acme.example.v2.ListOptions options = 1;
  acme.example.v2.TenantRef tenant = 2;
  ReleaseChannel channel_filter = 3;
}
```

