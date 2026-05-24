# CancelPipelineRequest

CancelPipelineRequest stops a run by id.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message CancelPipelineRequest {
  string run_id = 1 [(buf.validate.field).string.uuid = true];
  string reason = 2 [(buf.validate.field).string.max_len = 512];
}
```

