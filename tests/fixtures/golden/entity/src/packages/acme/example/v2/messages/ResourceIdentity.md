# ResourceIdentity

ResourceIdentity combines tenant, labels, and metadata.

*`acme/example/v2/types.proto`*

```protobuf
message ResourceIdentity {
  TenantRef tenant = 1;
  LabelSet labels = 2;
  SharedMetadata metadata = 3;
  string resource_name = 4;
}
```

