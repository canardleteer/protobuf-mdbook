# ListFeatureFlagsResponse

ListFeatureFlagsResponse returns a page of flags.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message ListFeatureFlagsResponse {
  repeated FeatureFlag flags = 1;
  acme.example.v2.PageResult page = 2;
}
```

