# ProductSku

ProductSku identifies a sellable item in documentation tables.

*`acme/example/v2/catalog.proto`*

```protobuf
message ProductSku {
  string sku = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 64
    ];
  string title = 2;
  string description = 3;
  Money price = 4;
  ProductStatus status = 5;
  repeated Label labels = 6;
}
```

