# Money

Money represents a decimal amount with currency code.

*`acme/example/v2/catalog.proto`*

```protobuf
message Money {
  int64 units = 1;
  int32 nanos = 2;
  string currency_code = 3 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.len = 3
    ];
}
```

