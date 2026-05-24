# Product

Product bundles many SKUs for list RPC examples.

*`acme/example/v2/catalog.proto`*

```protobuf
message Product {
  string product_id = 1;
  string display_name = 2;
  repeated ProductSku skus = 3;
  TenantRef owner = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp updated_at = 6;
  Address warehouse = 7;
}
```

