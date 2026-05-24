# WatchPipelineRequest

WatchPipelineRequest subscribes to pipeline step events.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message WatchPipelineRequest {
  string run_id = 1 [(buf.validate.field).string.uuid = true];
  acme.example.v2.TimeWindow window = 2;
  repeated acme.example.v2.FilterExpression filters = 3;
}
```

