# ExperimentArm

ExperimentArm is one variant in an A/B style experiment.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message ExperimentArm {
  string arm_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.pattern = "^arm_[a-z0-9]+$"
    ];
  string label = 2 [(buf.validate.field).string.max_len = 64];
  uint32 weight_basis_points = 3 [
      (buf.validate.field).uint32.gte = 0,
      (buf.validate.field).uint32.lte = 10000
    ];
  map<string, string> parameters = 4;
}
```

