# RelayOpen

RelayOpen carries handshake metadata for RelayConnect.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayOpen {
  string session_name = 1;
  acme.example.v2.ResourceIdentity identity = 2;
  acme.example.v2.QuotaLimits requested_limits = 3;
  repeated acme.example.v2.Label labels = 4;
}
```

