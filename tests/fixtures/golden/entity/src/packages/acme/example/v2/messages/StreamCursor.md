# StreamCursor

StreamCursor supports resumable stream examples.

*`acme/example/v2/types.proto`*

```protobuf
message StreamCursor {
  string stream_id = 1;
  uint64 sequence = 2;
  google.protobuf.Timestamp emitted_at = 3;
}
```

