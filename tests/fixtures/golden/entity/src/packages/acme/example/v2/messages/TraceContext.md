# TraceContext

TraceContext duplicates some metadata for nested RPC examples.

*`acme/example/v2/types.proto`*

```protobuf
message TraceContext {
  string trace_id = 1;
  string span_id = 2;
  google.protobuf.Duration sampling_delay = 3;
}
```

