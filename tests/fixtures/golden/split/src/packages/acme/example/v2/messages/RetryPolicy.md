# RetryPolicy

RetryPolicy is embedded in long-running operation messages.

*`acme/example/v2/types.proto`*

```protobuf
message RetryPolicy {
  uint32 max_attempts = 1;
  google.protobuf.Duration initial_backoff = 2;
  google.protobuf.Duration max_backoff = 3;
  double backoff_multiplier = 4;
}
```

