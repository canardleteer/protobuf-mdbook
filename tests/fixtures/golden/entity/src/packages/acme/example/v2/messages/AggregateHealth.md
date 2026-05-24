# AggregateHealth

AggregateHealth rolls up component statuses.

*`acme/example/v2/types.proto`*

```protobuf
message AggregateHealth {
  HealthStatus overall = 1;
  repeated ComponentHealth components = 2;
  google.protobuf.Timestamp evaluated_at = 3;
}
```

