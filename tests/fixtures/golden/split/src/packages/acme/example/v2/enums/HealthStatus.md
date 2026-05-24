# HealthStatus

HealthStatus is reported by gateway health RPCs.

*`acme/example/v2/types.proto`*

```protobuf
enum HealthStatus {
  HEALTH_STATUS_UNSPECIFIED = 0;
  HEALTH_STATUS_SERVING = 1;
  HEALTH_STATUS_NOT_SERVING = 2;
  HEALTH_STATUS_DEGRADED = 3;
}
```

