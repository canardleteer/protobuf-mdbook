# PipelineStepResult

PipelineStepResult reports completion for a single step.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message PipelineStepResult {
  string step_name = 1;
  PipelineStatus status = 2;
  acme.example.v2.ErrorDetail error = 3;
  acme.example.v2.StreamCursor cursor = 4;
  google.protobuf.Timestamp finished_at = 5;
}
```

