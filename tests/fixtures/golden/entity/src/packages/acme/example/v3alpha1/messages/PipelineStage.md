# PipelineStage

PipelineStage is a oneof-heavy stage definition for doc rendering.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message PipelineStage {
  oneof stage {
      PipelineStepInput step = 1;
      RolloutStage rollout_only = 2;
      acme.example.v2.AuditBatch audit_snapshot = 3;
    }
}
```

