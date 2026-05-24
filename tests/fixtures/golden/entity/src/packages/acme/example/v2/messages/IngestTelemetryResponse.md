# IngestTelemetryResponse

IngestTelemetryResponse acknowledges ingested points.

*`acme/example/v2/services.proto`*

```protobuf
message IngestTelemetryResponse {
  uint64 accepted = 1;
  uint64 rejected = 2;
}
```

