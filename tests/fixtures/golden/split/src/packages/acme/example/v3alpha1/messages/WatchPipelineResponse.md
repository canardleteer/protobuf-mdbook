# WatchPipelineResponse

WatchPipelineResponse is one event on the WatchPipeline server stream.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message WatchPipelineResponse {
  string event_id = 1;
  PipelineStepResult step = 2;
  google.protobuf.Timestamp observed_at = 3;
  acme.example.v2.StreamCursor cursor = 4;
}
```

