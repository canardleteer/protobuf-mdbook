# PublishEventsResponse

PublishEventsResponse acknowledges a published event stream.

*`acme/example/v1/gateway.proto`*

```protobuf
message PublishEventsResponse {
  string event_id = 1;
  acme.example.v2.StreamCursor cursor = 2;
}
```

