# ApplyInventoryAdjustmentsResponse

ApplyInventoryAdjustmentsResponse summarizes applied rows.

*`acme/example/v2/catalog.proto`*

```protobuf
message ApplyInventoryAdjustmentsResponse {
  uint32 applied = 1;
  repeated ErrorDetail failures = 2;
}
```

