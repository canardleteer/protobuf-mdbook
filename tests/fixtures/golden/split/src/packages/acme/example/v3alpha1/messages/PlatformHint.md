# PlatformHint

PlatformHint links alpha metadata back into v2 envelopes.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message PlatformHint {
  string hint_id = 1;
  acme.example.v2.PayloadEnvelope envelope = 2;
  repeated acme.example.v2.Label labels = 3;
}
```

