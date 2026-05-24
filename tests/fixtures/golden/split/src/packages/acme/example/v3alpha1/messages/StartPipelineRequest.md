# StartPipelineRequest

StartPipelineRequest kicks off a pipeline from staged inputs.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message StartPipelineRequest {
  string pipeline_name = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.pattern = "^[a-z][a-z0-9-]*$"
    ];
  repeated PipelineStage stages = 2 [(buf.validate.field).repeated.min_items = 1];
  acme.example.v2.ResourceIdentity actor = 3;
}
```

