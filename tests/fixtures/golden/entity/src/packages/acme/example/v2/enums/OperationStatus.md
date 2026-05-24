# OperationStatus

OperationStatus tracks synthetic long-running work.

*`acme/example/v2/types.proto`*

```protobuf
enum OperationStatus {
  OPERATION_STATUS_UNSPECIFIED = 0;
  OPERATION_STATUS_PENDING = 1;
  OPERATION_STATUS_RUNNING = 2;
  OPERATION_STATUS_SUCCEEDED = 3;
  OPERATION_STATUS_FAILED = 4;
  OPERATION_STATUS_CANCELLED = 5;
}
```

