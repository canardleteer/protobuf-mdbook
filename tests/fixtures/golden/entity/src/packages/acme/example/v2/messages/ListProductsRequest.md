# ListProductsRequest

ListProductsRequest paginates catalog inventory.

*`acme/example/v2/catalog.proto`*

```protobuf
message ListProductsRequest {
  ListOptions options = 1;
  ProductStatus status_filter = 2;
  string search_query = 3 [(buf.validate.field).string.max_len = 256];
}
```

