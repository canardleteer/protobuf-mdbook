# PipelineStatus

PipelineStatus tracks synthetic workflow execution.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
enum PipelineStatus {
  PIPELINE_STATUS_UNSPECIFIED = 0;
  PIPELINE_STATUS_QUEUED = 1;
  PIPELINE_STATUS_RUNNING = 2;
  PIPELINE_STATUS_SUCCEEDED = 3;
  PIPELINE_STATUS_FAILED = 4;
  PIPELINE_STATUS_CANCELLED = 5;
}
```

