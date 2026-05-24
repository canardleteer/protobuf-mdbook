# WatchInventoryRequest

WatchInventoryRequest subscribes to inventory change events.

*`acme/example/v2/services.proto`*

```protobuf
message WatchInventoryRequest {
  string warehouse_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 128
    ];
  TimeWindow window = 2;
  repeated FilterExpression filters = 3;
}
```

