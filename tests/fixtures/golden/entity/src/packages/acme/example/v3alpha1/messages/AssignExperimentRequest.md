# AssignExperimentRequest

AssignExperimentRequest assigns a subject to an experiment arm.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message AssignExperimentRequest {
  ExperimentSpec spec = 1;
  SubjectRef subject = 2;
  repeated string exclusion_keys = 3 [(buf.validate.field).repeated.max_items = 32];
}
```

