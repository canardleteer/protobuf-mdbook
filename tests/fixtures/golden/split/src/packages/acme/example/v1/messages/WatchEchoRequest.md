# WatchEchoRequest

WatchEchoRequest subscribes to echo events for a topic.

*`acme/example/v1/echo.proto`*

```protobuf
message WatchEchoRequest {
  string topic = 1;
  acme.example.v2.TimeWindow window = 2;
  repeated acme.example.v2.FilterExpression filters = 3;
}
```

