# BatchRequest

BatchRequest aggregates many items for BatchEcho RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message BatchRequest {
  repeated BatchItem items = 1 [(buf.validate.field).repeated.min_items = 1];
  ResourceIdentity identity = 2;
  bool partial_failure_allowed = 3;
}
```

