# ComponentHealth

ComponentHealth describes one sub-system in aggregate health.

*`acme/example/v2/types.proto`*

```protobuf
message ComponentHealth {
  string component = 1;
  HealthStatus status = 2;
  string detail = 3;
}
```

