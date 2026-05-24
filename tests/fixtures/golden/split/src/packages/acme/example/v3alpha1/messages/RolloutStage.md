# RolloutStage

RolloutStage documents progressive delivery for pipeline stories.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message RolloutStage {
  ReleaseChannel channel = 1;
  google.protobuf.Duration bake_time = 2;
  uint32 max_parallel = 3 [(buf.validate.field).uint32.lte = 1000];
}
```

