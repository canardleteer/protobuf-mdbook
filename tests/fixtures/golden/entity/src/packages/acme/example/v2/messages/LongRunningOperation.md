# LongRunningOperation

LongRunningOperation is returned by admin-style RPCs in v1.

*`acme/example/v2/types.proto`*

```protobuf
message LongRunningOperation {
  string name = 1;
  OperationStatus status = 2;
  google.protobuf.Timestamp start_time = 3;
  google.protobuf.Timestamp end_time = 4;
  RetryPolicy retry_policy = 5;
  ErrorDetail error = 6;
  double percent_complete = 7;
}
```

