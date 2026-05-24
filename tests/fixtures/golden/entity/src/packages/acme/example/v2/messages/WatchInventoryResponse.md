# WatchInventoryResponse

WatchInventoryResponse is one event on the WatchInventory server stream.

*`acme/example/v2/services.proto`*

```protobuf
message WatchInventoryResponse {
  string event_id = 1;
  InventoryAdjustment adjustment = 2;
  google.protobuf.Timestamp observed_at = 3;
  StreamCursor cursor = 4;
}
```

