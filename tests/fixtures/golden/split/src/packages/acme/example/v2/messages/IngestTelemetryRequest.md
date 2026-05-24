# IngestTelemetryRequest

IngestTelemetryRequest is one client-streaming telemetry point.

*`acme/example/v2/services.proto`*

```protobuf
message IngestTelemetryRequest {
  string metric_name = 1;
  double value = 2;
  google.protobuf.Timestamp observed_at = 3;
  LabelSet labels = 4;
}
```

