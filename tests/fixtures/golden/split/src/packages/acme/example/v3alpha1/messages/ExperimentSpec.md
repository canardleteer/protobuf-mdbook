# ExperimentSpec

ExperimentSpec defines arms and eligibility for assignment RPCs.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message ExperimentSpec {
  
  string experiment_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.uuid = true
    ];
  string hypothesis = 2;
  repeated ExperimentArm arms = 3 [(buf.validate.field).repeated.min_items = 2];
  acme.example.v2.TimeWindow enrollment_window = 4;
  acme.example.v2.LabelSet audience = 5;
}
```

**Protovalidate (CEL)**

```cel
id: "experiment.arms_weights_positive"
      message: "each arm must have positive weight in basis points"
      expression: "this.arms.all(arm, arm.weight_basis_points > 0u)"
```

