# ApplyInventoryAdjustmentsRequest

ApplyInventoryAdjustmentsRequest batches adjustments.

*`acme/example/v2/catalog.proto`*

```protobuf
message ApplyInventoryAdjustmentsRequest {
  repeated InventoryAdjustment adjustments = 1;
  ResourceIdentity actor = 2;
}
```

