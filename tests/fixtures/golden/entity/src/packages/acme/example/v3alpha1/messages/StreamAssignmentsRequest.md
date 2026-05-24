# StreamAssignmentsRequest

StreamAssignmentsRequest opens a server stream of assignment events.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message StreamAssignmentsRequest {
  string experiment_id = 1 [(buf.validate.field).string.uuid = true];
  acme.example.v2.TimeWindow window = 2;
}
```

