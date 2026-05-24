# PipelineStepInput

PipelineStepInput is one unit of work inside a pipeline run.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message PipelineStepInput {
  string step_name = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 64
    ];
  acme.example.v2.PayloadEnvelope input = 2;
  RolloutStage rollout = 3;
}
```

