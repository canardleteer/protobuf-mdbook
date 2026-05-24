# ListProductsResponse

ListProductsResponse returns a page of products.

*`acme/example/v2/catalog.proto`*

```protobuf
message ListProductsResponse {
  repeated Product products = 1;
  PageResult page = 2;
}
```

