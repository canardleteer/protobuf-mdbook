# InventoryAdjustment

InventoryAdjustment documents stock changes for admin RPCs.

*`acme/example/v2/catalog.proto`*

```protobuf
message InventoryAdjustment {
  string sku = 1;
  int64 delta = 2;
  string reason = 3;
  google.protobuf.Timestamp adjusted_at = 4;
}
```

