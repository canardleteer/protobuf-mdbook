# PublishEventsRequest

PublishEventsRequest is one client-streaming upload chunk.

*`acme/example/v1/gateway.proto`*

```protobuf
message PublishEventsRequest {
  string event_type = 1;
  acme.example.v2.PayloadEnvelope envelope = 2;
  google.protobuf.Timestamp client_time = 3;
}
```

