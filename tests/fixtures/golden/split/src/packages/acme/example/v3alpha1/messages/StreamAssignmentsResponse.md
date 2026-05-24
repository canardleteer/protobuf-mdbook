# StreamAssignmentsResponse

StreamAssignmentsResponse is one assignment event on the stream.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message StreamAssignmentsResponse {
  ExperimentAssignment assignment = 1;
  acme.example.v2.StreamCursor cursor = 2;
  google.protobuf.Timestamp observed_at = 3;
}
```

