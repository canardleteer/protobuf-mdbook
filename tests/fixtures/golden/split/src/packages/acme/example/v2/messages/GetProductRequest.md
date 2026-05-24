# GetProductRequest

GetProductRequest fetches a single product by id.

*`acme/example/v2/catalog.proto`*

```protobuf
message GetProductRequest {
  string product_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.uuid = true
    ];
}
```

