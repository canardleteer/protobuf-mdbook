# QuotaLimits

QuotaLimits documents soft limits referenced from gateway RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message QuotaLimits {
  uint32 max_requests_per_minute = 1;
  uint32 max_stream_duration_seconds = 2;
  uint64 max_payload_bytes = 3;
}
```

