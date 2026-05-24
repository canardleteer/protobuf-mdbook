# RelayAck

RelayAck confirms a relay session is ready.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayAck {
  string session_id = 1;
  google.protobuf.Timestamp opened_at = 2;
  acme.example.v2.AggregateHealth health = 3;
}
```

